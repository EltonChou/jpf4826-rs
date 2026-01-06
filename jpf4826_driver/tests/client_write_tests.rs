mod mock;

use jpf4826_driver::{Jpf4826Client, OperatingMode, PwmFrequency, WorkMode};
use mock::MockController;

// Helper to create a test client
async fn create_test_client() -> (Jpf4826Client, MockController) {
    let mock = MockController::new();
    let registers = mock.registers.clone();
    let client = Jpf4826Client::new_mock(registers, 1).await;
    (client, mock)
}

#[tokio::test]
async fn test_write_low_level() {
    let (mut client, mock) = create_test_client().await;

    use jpf4826_driver::registers::RegisterAddress;

    // Write to a register
    client
        .write(RegisterAddress::ModbusAddress, 5)
        .await
        .unwrap();

    // Verify it was written
    let value = mock.read_register(0x0002).unwrap();
    assert_eq!(value, 5);
}

#[tokio::test]
async fn test_reset() {
    let (mut client, mock) = create_test_client().await;

    client.reset().await.unwrap();

    // Verify reset command was written (0x00AA to register 0x0020)
    let value = mock.read_register(0x0020).unwrap();
    assert_eq!(value, 0x00AA);
}

#[tokio::test]
async fn test_set_mode_temperature() {
    let (mut client, mock) = create_test_client().await;

    client.set_mode(OperatingMode::Temperature).await.unwrap();

    // Verify 0xFFFF was written to register 0x0003
    let value = mock.read_register(0x0003).unwrap();
    assert_eq!(value, 0xFFFF);
}

#[tokio::test]
async fn test_set_mode_manual() {
    let (mut client, mock) = create_test_client().await;

    client.set_mode(OperatingMode::Manual).await.unwrap();

    // Manual mode sets register to 0 (actual speed set via set_fan_speed)
    let value = mock.read_register(0x0003).unwrap();
    assert_eq!(value, 0);
}

#[tokio::test]
async fn test_set_eco_shutdown() {
    let (mut client, mock) = create_test_client().await;

    client.set_eco(WorkMode::Shutdown).await.unwrap();

    let value = mock.read_register(0x0005).unwrap();
    assert_eq!(value, 0x0000);
}

#[tokio::test]
async fn test_set_eco_minimum_speed() {
    let (mut client, mock) = create_test_client().await;

    client.set_eco(WorkMode::MinimumSpeed).await.unwrap();

    let value = mock.read_register(0x0005).unwrap();
    assert_eq!(value, 0x0001);
}

#[tokio::test]
async fn test_set_fan_speed_valid() {
    let (mut client, mock) = create_test_client().await;

    // Set speed to 75%
    client.set_fan_speed(75).await.unwrap();

    let value = mock.read_register(0x0003).unwrap();
    assert_eq!(value, 75);
}

#[tokio::test]
async fn test_set_fan_speed_zero() {
    let (mut client, mock) = create_test_client().await;

    client.set_fan_speed(0).await.unwrap();

    let value = mock.read_register(0x0003).unwrap();
    assert_eq!(value, 0);
}

#[tokio::test]
async fn test_set_fan_speed_hundred() {
    let (mut client, mock) = create_test_client().await;

    client.set_fan_speed(100).await.unwrap();

    let value = mock.read_register(0x0003).unwrap();
    assert_eq!(value, 100);
}

#[tokio::test]
async fn test_set_fan_speed_invalid() {
    let (mut client, _mock) = create_test_client().await;

    // 101% should fail
    let result = client.set_fan_speed(101).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_set_fan_count() {
    let (mut client, mock) = create_test_client().await;

    client.set_fan_count(3).await.unwrap();

    let value = mock.read_register(0x0006).unwrap();
    assert_eq!(value, 3);
}

#[tokio::test]
async fn test_disable_fault_detection() {
    let (mut client, mock) = create_test_client().await;

    client.disable_fault_detection().await.unwrap();

    // Setting fan count to 0 disables fault detection
    let value = mock.read_register(0x0006).unwrap();
    assert_eq!(value, 0);
}

#[tokio::test]
async fn test_set_fan_count_invalid() {
    let (mut client, _mock) = create_test_client().await;

    // Fan count 5 should fail (valid: 0-4)
    let result = client.set_fan_count(5).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_set_addr_valid() {
    let (mut client, mock) = create_test_client().await;

    client.set_addr(10).await.unwrap();

    let value = mock.read_register(0x0002).unwrap();
    assert_eq!(value, 10);
}

#[tokio::test]
async fn test_set_addr_invalid_zero() {
    let (mut client, _mock) = create_test_client().await;

    // Address 0 is invalid
    let result = client.set_addr(0).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_set_addr_invalid_255() {
    let (mut client, _mock) = create_test_client().await;

    // Address 255 is invalid (max is 254)
    let result = client.set_addr(255).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_set_addr_updates_client_internal_address() {
    let (mut client, mock) = create_test_client().await;

    // Verify initial address
    assert_eq!(client.slave_addr(), 1, "Initial client address should be 1");

    // Change address to 42
    client.set_addr(42).await.unwrap();

    // Verify the controller's register was updated
    let register_value = mock.read_register(0x0002).unwrap();
    assert_eq!(
        register_value, 42,
        "Controller register should contain new address"
    );

    // Verify the client's internal address was synchronized
    assert_eq!(
        client.slave_addr(),
        42,
        "Client internal address should be updated to match controller"
    );

    // Change address again to verify it works multiple times
    client.set_addr(100).await.unwrap();
    assert_eq!(
        client.slave_addr(),
        100,
        "Client address should update correctly on subsequent calls"
    );
    assert_eq!(
        mock.read_register(0x0002).unwrap(),
        100,
        "Controller register should reflect second address change"
    );
}

#[tokio::test]
async fn test_set_pwm_frequency() {
    let (mut client, mock) = create_test_client().await;

    client
        .set_pwm_frequency(PwmFrequency::Hz5000)
        .await
        .unwrap();

    let value = mock.read_register(0x000B).unwrap();
    assert_eq!(value, 0x0003); // Hz5000 = 0x0003
}

#[tokio::test]
async fn test_set_temperature_threshold_valid() {
    let (mut client, mock) = create_test_client().await;

    // Start 25°C, Full 45°C
    client.set_temperature_threshold(25, 45).await.unwrap();

    // Check start temp (register 0x000C)
    let start = mock.read_register(0x000C).unwrap();
    assert_eq!(start, 65); // 25 + 40

    // Check full temp (register 0x000D)
    let full = mock.read_register(0x000D).unwrap();
    assert_eq!(full, 85); // 45 + 40
}

#[tokio::test]
async fn test_set_temperature_threshold_invalid_order() {
    let (mut client, _mock) = create_test_client().await;

    // High temp must be greater than low temp
    let result = client.set_temperature_threshold(50, 30).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_set_temperature_threshold_equal() {
    let (mut client, _mock) = create_test_client().await;

    // Equal temps should fail
    let result = client.set_temperature_threshold(40, 40).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_set_temperature_threshold_out_of_range() {
    let (mut client, _mock) = create_test_client().await;

    // -25°C is below minimum (-20°C)
    let result = client.set_temperature_threshold(-25, 50).await;
    assert!(result.is_err());

    // 125°C is above maximum (120°C)
    let result2 = client.set_temperature_threshold(20, 125).await;
    assert!(result2.is_err());
}
