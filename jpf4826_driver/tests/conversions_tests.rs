use jpf4826_driver::conversions::*;
use jpf4826_driver::types::FanStatus;

// Temperature conversion tests

#[test]
fn test_celsius_to_register_value() {
    // 31°C should map to 71 (31 + 40)
    assert_eq!(celsius_to_register(31), 71);
    assert_eq!(celsius_to_register(0), 40);
    assert_eq!(celsius_to_register(-20), 20);
    assert_eq!(celsius_to_register(120), 160);
}

#[test]
fn test_register_to_celsius() {
    // Register value 71 should map to 31°C (71 - 40)
    assert_eq!(register_to_celsius(71), 31);
    assert_eq!(register_to_celsius(40), 0);
    assert_eq!(register_to_celsius(20), -20);
    assert_eq!(register_to_celsius(160), 120);
}

#[test]
fn test_celsius_to_fahrenheit() {
    assert_eq!(celsius_to_fahrenheit(0), 32);
    assert_eq!(celsius_to_fahrenheit(100), 212);
    assert_eq!(celsius_to_fahrenheit(31), 87); // 31°C = 87.8°F, rounded to 87 or 88
    assert_eq!(celsius_to_fahrenheit(-20), -4);
}

#[test]
fn test_temperature_roundtrip() {
    // Test that celsius -> register -> celsius works
    let original = 31;
    let register = celsius_to_register(original);
    let roundtrip = register_to_celsius(register);
    assert_eq!(original, roundtrip);
}

// Bitmap parsing tests

#[test]
fn test_parse_fan_status_bitmap_all_running() {
    // 0b1111 = all fans running
    let statuses = parse_fan_status_bitmap(0x000F);
    assert_eq!(statuses.len(), 4);
    assert!(statuses[0]); // Fan 1 running
    assert!(statuses[1]); // Fan 2 running
    assert!(statuses[2]); // Fan 3 running
    assert!(statuses[3]); // Fan 4 running
}

#[test]
fn test_parse_fan_status_bitmap_fan1_only() {
    // 0b0001 = only fan 1 running
    let statuses = parse_fan_status_bitmap(0x0001);
    assert_eq!(statuses.len(), 4);
    assert!(statuses[0]); // Fan 1 running
    assert!(!statuses[1]); // Fan 2 stopped
    assert!(!statuses[2]); // Fan 3 stopped
    assert!(!statuses[3]); // Fan 4 stopped
}

#[test]
fn test_parse_fan_status_bitmap_none_running() {
    // 0b0000 = no fans running
    let statuses = parse_fan_status_bitmap(0x0000);
    assert_eq!(statuses.len(), 4);
    assert!(!statuses[0]);
    assert!(!statuses[1]);
    assert!(!statuses[2]);
    assert!(!statuses[3]);
}

#[test]
fn test_parse_fan_fault_bitmap_all_normal() {
    // 0b1111 = all fans normal (bits inverted: 1=normal, 0=fault)
    let faults = parse_fan_fault_bitmap(0x000F);
    assert_eq!(faults.len(), 4);
    for fault in faults {
        assert_eq!(fault, FanStatus::Normal);
    }
}

#[test]
fn test_parse_fan_fault_bitmap_fan3_fault() {
    // 0xFB = 0b11111011 = Fan 3 has fault (bit 2 = 0)
    let faults = parse_fan_fault_bitmap(0x00FB);
    assert_eq!(faults.len(), 4);
    assert_eq!(faults[0], FanStatus::Normal); // Bit 0 = 1
    assert_eq!(faults[1], FanStatus::Normal); // Bit 1 = 1
    assert_eq!(faults[2], FanStatus::Fault); // Bit 2 = 0
    assert_eq!(faults[3], FanStatus::Normal); // Bit 3 = 1
}

#[test]
fn test_parse_fan_fault_bitmap_all_faults() {
    // 0b0000 = all fans have faults (inverted)
    let faults = parse_fan_fault_bitmap(0x0000);
    assert_eq!(faults.len(), 4);
    for fault in faults {
        assert_eq!(fault, FanStatus::Fault);
    }
}

// Combined temperature register tests

#[test]
fn test_parse_combined_temperature() {
    // 0x465A = high byte 0x46 (70 = 30°C), low byte 0x5A (90 = 50°C)
    let (low, high) = parse_combined_temperature(0x465A);
    assert_eq!(low, 30); // 70 - 40
    assert_eq!(high, 50); // 90 - 40
}

#[test]
fn test_encode_combined_temperature() {
    // Start temp 30°C (70), Full temp 50°C (90)
    let combined = encode_combined_temperature(30, 50);
    assert_eq!(combined, 0x465A);
}

#[test]
fn test_combined_temperature_roundtrip() {
    let original = (30, 50);
    let encoded = encode_combined_temperature(original.0, original.1);
    let decoded = parse_combined_temperature(encoded);
    assert_eq!(original, decoded);
}
