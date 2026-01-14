use jpf4826_driver::types::*;

#[test]
fn test_operating_mode_temperature() {
    let mode = OperatingMode::Temperature;
    assert_eq!(mode.to_register_value(), 0xFFFF);
}

#[test]
fn test_operating_mode_manual() {
    // Manual mode uses 0-100 values, not an enum value
    // The actual speed is set separately via set_fan_speed
    // This test verifies the enum exists
    let _mode = OperatingMode::Manual;
}

#[test]
fn test_work_mode_shutdown() {
    let mode = WorkMode::Shutdown;
    assert_eq!(mode.to_register_value(), 0x0000);
}

#[test]
fn test_work_mode_minimum_speed() {
    let mode = WorkMode::MinimumSpeed;
    assert_eq!(mode.to_register_value(), 0x0001);
}

#[test]
fn test_work_mode_from_register() {
    assert_eq!(
        WorkMode::from_register_value(0x0000),
        Some(WorkMode::Shutdown)
    );
    assert_eq!(
        WorkMode::from_register_value(0x0001),
        Some(WorkMode::MinimumSpeed)
    );
    assert_eq!(WorkMode::from_register_value(0x0002), None);
}

#[test]
fn test_fan_status_variants() {
    let _normal = FanStatus::Normal;
    let _fault = FanStatus::Fault;
}

#[test]
fn test_temperature_unit_variants() {
    let _celsius = TemperatureUnit::Celsius;
    let _fahrenheit = TemperatureUnit::Fahrenheit;
}

#[test]
fn test_pwm_frequency_all_values() {
    assert_eq!(PwmFrequency::Hz500.to_register_value(), 0x0000);
    assert_eq!(PwmFrequency::Hz1000.to_register_value(), 0x0001);
    assert_eq!(PwmFrequency::Hz2000.to_register_value(), 0x0002);
    assert_eq!(PwmFrequency::Hz5000.to_register_value(), 0x0003);
    assert_eq!(PwmFrequency::Hz10000.to_register_value(), 0x0004);
    assert_eq!(PwmFrequency::Hz25000.to_register_value(), 0x0005);
}

#[test]
fn test_pwm_frequency_from_register() {
    assert_eq!(
        PwmFrequency::from_register_value(0x0000),
        Some(PwmFrequency::Hz500)
    );
    assert_eq!(
        PwmFrequency::from_register_value(0x0001),
        Some(PwmFrequency::Hz1000)
    );
    assert_eq!(
        PwmFrequency::from_register_value(0x0002),
        Some(PwmFrequency::Hz2000)
    );
    assert_eq!(
        PwmFrequency::from_register_value(0x0003),
        Some(PwmFrequency::Hz5000)
    );
    assert_eq!(
        PwmFrequency::from_register_value(0x0004),
        Some(PwmFrequency::Hz10000)
    );
    assert_eq!(
        PwmFrequency::from_register_value(0x0005),
        Some(PwmFrequency::Hz25000)
    );
    assert_eq!(PwmFrequency::from_register_value(0x0006), None);
}

#[test]
fn test_pwm_frequency_to_hz() {
    assert_eq!(PwmFrequency::Hz500.to_hz(), 500);
    assert_eq!(PwmFrequency::Hz1000.to_hz(), 1000);
    assert_eq!(PwmFrequency::Hz2000.to_hz(), 2000);
    assert_eq!(PwmFrequency::Hz5000.to_hz(), 5000);
    assert_eq!(PwmFrequency::Hz10000.to_hz(), 10000);
    assert_eq!(PwmFrequency::Hz25000.to_hz(), 25000);
}

#[test]
fn test_pwm_frequency_from_hz() {
    assert_eq!(PwmFrequency::from_hz(500), Some(PwmFrequency::Hz500));
    assert_eq!(PwmFrequency::from_hz(1000), Some(PwmFrequency::Hz1000));
    assert_eq!(PwmFrequency::from_hz(2000), Some(PwmFrequency::Hz2000));
    assert_eq!(PwmFrequency::from_hz(5000), Some(PwmFrequency::Hz5000));
    assert_eq!(PwmFrequency::from_hz(10000), Some(PwmFrequency::Hz10000));
    assert_eq!(PwmFrequency::from_hz(25000), Some(PwmFrequency::Hz25000));
    assert_eq!(PwmFrequency::from_hz(3000), None);
}

#[test]
fn test_temperature_struct() {
    let temp = Temperature {
        value: 31,
        unit: TemperatureUnit::Celsius,
    };
    assert_eq!(temp.value, 31);
    assert!(matches!(temp.unit, TemperatureUnit::Celsius));
}

#[test]
fn test_fan_info_struct() {
    let fan = FanInfo {
        index: 1,
        status: FanStatus::Normal,
        rpm: 1400,
    };
    assert_eq!(fan.index, 1);
    assert!(matches!(fan.status, FanStatus::Normal));
    assert_eq!(fan.rpm, 1400);
}

#[test]
fn test_controller_status_struct_creation() {
    // Just test that we can create the struct
    let status = ControllerStatus {
        eco_mode: true,
        modbus_address: 1,
        pwm_frequency: PwmFrequency::Hz25000,
        fan_count: 4,
        temperature_current: Temperature {
            value: 26,
            unit: TemperatureUnit::Celsius,
        },
        temperature_low_threshold: Temperature {
            value: 27,
            unit: TemperatureUnit::Celsius,
        },
        temperature_high_threshold: Temperature {
            value: 40,
            unit: TemperatureUnit::Celsius,
        },
        fans: vec![],
    };

    assert!(status.eco_mode);
    assert_eq!(status.fan_count, 4);
}
