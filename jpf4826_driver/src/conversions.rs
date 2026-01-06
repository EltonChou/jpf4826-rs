//! Conversion utilities for JPF4826 protocol values.
//!
//! This module handles transformations between controller register values
//! and human-readable representations, including temperature offsets,
//! bitmap parsing, and unit conversions.

// Rust guideline compliant 2026-01-06

use crate::types::FanStatus;

/// Offset added to Celsius temperatures in Modbus registers.
///
/// JPF4826 stores temperatures with a +40 offset to handle negative values.
/// Temperature range: -20°C to 120°C maps to register values 20 to 160.
const TEMPERATURE_OFFSET: i16 = 40;

/// Converts Celsius temperature to Modbus register value.
///
/// # Examples
///
/// ```
/// # use jpf4826_driver::conversions::celsius_to_register;
/// assert_eq!(celsius_to_register(31), 71); // 31 + 40
/// assert_eq!(celsius_to_register(0), 40);
/// assert_eq!(celsius_to_register(-20), 20);
/// ```
pub fn celsius_to_register(celsius: i16) -> u16 {
    (celsius + TEMPERATURE_OFFSET) as u16
}

/// Converts Modbus register value to Celsius temperature.
///
/// # Examples
///
/// ```
/// # use jpf4826_driver::conversions::register_to_celsius;
/// assert_eq!(register_to_celsius(71), 31); // 71 - 40
/// assert_eq!(register_to_celsius(40), 0);
/// assert_eq!(register_to_celsius(160), 120);
/// ```
pub fn register_to_celsius(register: u16) -> i16 {
    register as i16 - TEMPERATURE_OFFSET
}

/// Converts Celsius to Fahrenheit.
///
/// # Examples
///
/// ```
/// # use jpf4826_driver::conversions::celsius_to_fahrenheit;
/// assert_eq!(celsius_to_fahrenheit(0), 32);
/// assert_eq!(celsius_to_fahrenheit(100), 212);
/// ```
pub fn celsius_to_fahrenheit(celsius: i16) -> i16 {
    (celsius * 9 / 5) + 32
}

/// Parses fan running status from bitmap register.
///
/// Register 0x0001 contains fan status bits where:
/// - Bit 0 = Fan 1 (1=running, 0=stopped)
/// - Bit 1 = Fan 2
/// - Bit 2 = Fan 3
/// - Bit 3 = Fan 4
///
/// # Examples
///
/// ```
/// # use jpf4826_driver::conversions::parse_fan_status_bitmap;
/// // 0x0001 = binary 0001 = only Fan 1 running
/// let statuses = parse_fan_status_bitmap(0x0001);
/// assert_eq!(statuses.len(), 4);
/// assert!(statuses[0]); // Fan 1 running
/// assert!(!statuses[1]); // Fan 2 stopped
/// ```
pub fn parse_fan_status_bitmap(bitmap: u16) -> [bool; 4] {
    [
        (bitmap & 0x01) != 0, // Fan 1
        (bitmap & 0x02) != 0, // Fan 2
        (bitmap & 0x04) != 0, // Fan 3
        (bitmap & 0x08) != 0, // Fan 4
    ]
}

/// Parses fan fault status from fault code bitmap.
///
/// Register 0x000E contains fault status bits where:
/// - Bit N: 1 = normal, 0 = fault (inverted logic)
/// - Bit 0 = Fan 1
/// - Bit 1 = Fan 2
/// - Bit 2 = Fan 3
/// - Bit 3 = Fan 4
///
/// # Examples
///
/// ```
/// # use jpf4826_driver::conversions::parse_fan_fault_bitmap;
/// # use jpf4826_driver::types::FanStatus;
/// // 0x00FB = binary 11111011 = Fan 3 fault (bit 2 = 0)
/// let faults = parse_fan_fault_bitmap(0x00FB);
/// assert_eq!(faults[2], FanStatus::Fault);
/// assert_eq!(faults[0], FanStatus::Normal);
/// ```
pub fn parse_fan_fault_bitmap(bitmap: u16) -> [FanStatus; 4] {
    [
        if (bitmap & 0x01) != 0 {
            FanStatus::Normal
        } else {
            FanStatus::Fault
        }, // Fan 1
        if (bitmap & 0x02) != 0 {
            FanStatus::Normal
        } else {
            FanStatus::Fault
        }, // Fan 2
        if (bitmap & 0x04) != 0 {
            FanStatus::Normal
        } else {
            FanStatus::Fault
        }, // Fan 3
        if (bitmap & 0x08) != 0 {
            FanStatus::Normal
        } else {
            FanStatus::Fault
        }, // Fan 4
    ]
}

/// Parses combined temperature register (0x0004).
///
/// Register 0x0004 stores start and full speed temperatures:
/// - High byte: Start temperature (low threshold)
/// - Low byte: Full speed temperature (high threshold)
///
/// Both values use +40 offset.
///
/// # Examples
///
/// ```
/// # use jpf4826_driver::conversions::parse_combined_temperature;
/// // 0x465A: high byte 0x46 (70 = 30°C), low byte 0x5A (90 = 50°C)
/// let (low, high) = parse_combined_temperature(0x465A);
/// assert_eq!(low, 30);
/// assert_eq!(high, 50);
/// ```
pub fn parse_combined_temperature(combined: u16) -> (i16, i16) {
    let high_byte = (combined >> 8) & 0xFF; // Start temp
    let low_byte = combined & 0xFF; // Full speed temp

    let start_temp = register_to_celsius(high_byte);
    let full_temp = register_to_celsius(low_byte);

    (start_temp, full_temp)
}

/// Encodes start and full temperatures into combined register.
///
/// Creates the 16-bit value for register 0x0004.
///
/// # Examples
///
/// ```
/// # use jpf4826_driver::conversions::encode_combined_temperature;
/// // Start 30°C, Full 50°C -> 0x465A
/// assert_eq!(encode_combined_temperature(30, 50), 0x465A);
/// ```
pub fn encode_combined_temperature(start_celsius: i16, full_celsius: i16) -> u16 {
    let start_register = celsius_to_register(start_celsius) as u8;
    let full_register = celsius_to_register(full_celsius) as u8;

    ((start_register as u16) << 8) | (full_register as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_offset_constant() {
        assert_eq!(TEMPERATURE_OFFSET, 40);
    }
}
