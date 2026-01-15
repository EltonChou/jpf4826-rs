#![cfg(feature = "test-mock")]

mod mock;

use jpf4826_driver::{FanStatus, Jpf4826Client, TemperatureUnit};
use mock::MockController;

// Helper to create a test client
#[cfg(any(test, feature = "test-mock"))]
async fn create_test_client() -> (Jpf4826Client, MockController) {
    let mock = MockController::new();
    let registers = mock.registers.clone();
    let client = Jpf4826Client::new_mock(registers, 1).await;
    (client, mock)
}

#[tokio::test]
async fn test_read_temperature() {
    let (mut client, _mock) = create_test_client().await;

    let temp = client.temperature().await.unwrap();
    assert_eq!(temp.value, 31); // 71 - 40 = 31°C
    assert_eq!(temp.unit, TemperatureUnit::Celsius);
}

#[tokio::test]
async fn test_read_fan_speed() {
    let (mut client, _mock) = create_test_client().await;

    // Test all 4 fans
    for fan_index in 1..=4 {
        let rpm = client.fan_speed(fan_index).await.unwrap();
        assert_eq!(rpm, 1400);
    }
}

#[tokio::test]
async fn test_read_fan_speed_invalid_index() {
    let (mut client, _mock) = create_test_client().await;

    // Fan index 0 should fail
    let result = client.fan_speed(0).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().invalid_fan_index().is_some());

    // Fan index 5 should fail
    let result = client.fan_speed(5).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_read_fan_count() {
    let (mut client, _mock) = create_test_client().await;

    let count = client.fan_count().await.unwrap();
    assert_eq!(count, 4);
}

#[tokio::test]
async fn test_read_fan_status() {
    let (mut client, mock) = create_test_client().await;

    // Initially all fans running and normal
    let fans = client.fan_status().await.unwrap();
    assert_eq!(fans.len(), 4);

    for (i, fan) in fans.iter().enumerate() {
        assert_eq!(fan.index, (i + 1) as u8);
        assert_eq!(fan.status, FanStatus::Normal);
        assert_eq!(fan.rpm, 1400);
    }

    // Set Fan 3 to have a fault
    mock.set_fan_fault(3, true);
    let fans = client.fan_status().await.unwrap();
    assert_eq!(fans[2].status, FanStatus::Fault);
    assert_eq!(fans[0].status, FanStatus::Normal);
}

#[tokio::test]
async fn test_read_full_status() {
    let (mut client, _mock) = create_test_client().await;

    let status = client.status().await.unwrap();

    // Verify all fields
    assert_eq!(status.modbus_address, 1);
    assert_eq!(status.fan_count, 4);
    assert!(!status.eco_mode); // Work mode = 1 = MinimumSpeed, so eco_mode = false
    assert_eq!(status.temperature_current.value, 31);
    assert_eq!(status.temperature_low_threshold.value, 30);
    assert_eq!(status.temperature_high_threshold.value, 50);
    assert_eq!(status.fans.len(), 4);
}

#[tokio::test]
async fn test_read_low_level() {
    let (mut client, _mock) = create_test_client().await;

    use jpf4826_driver::registers::RegisterAddress;

    // Read single register
    let values = client
        .read(RegisterAddress::CurrentTemperature, 1)
        .await
        .unwrap();
    assert_eq!(values.len(), 1);
    assert_eq!(values[0], 71); // 31°C + 40

    // Read multiple consecutive registers
    let values = client
        .read(RegisterAddress::CurrentTemperature, 3)
        .await
        .unwrap();
    assert_eq!(values.len(), 3);
    assert_eq!(values[0], 71); // Temperature
    assert_eq!(values[1], 0x000F); // Fan status
    assert_eq!(values[2], 0x0001); // Modbus addr
}
