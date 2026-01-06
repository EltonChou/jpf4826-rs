//! JPF4826 Modbus register address definitions.
//!
//! This module defines type-safe register addresses matching the
//! controller's Modbus-RTU register map.

// Rust guideline compliant 2026-01-06

/// Modbus register addresses for JPF4826 controller.
///
/// All register addresses follow the controller's register map
/// as documented in the JPF4826 protocol specification.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterAddress {
    /// Current temperature reading (INT16, Read-only).
    ///
    /// Value stored with +40 offset. Range: 0x0014-0x00A0 (-20°C to 120°C).
    CurrentTemperature = 0x0000,

    /// Fan running status bitmap (BITMAP, Read-only).
    ///
    /// Bit 0=Fan1, Bit 1=Fan2, Bit 2=Fan3, Bit 3=Fan4.
    /// 1=running, 0=stopped.
    FanStatus = 0x0001,

    /// Modbus device address (UINT16, Read/Write).
    ///
    /// Valid range: 0x0001-0x00FE (1-254). Broadcast address 0xFFFF supported.
    ModbusAddress = 0x0002,

    /// Manual speed control / Operating mode (UINT16, Read/Write).
    ///
    /// 0x0000-0x0064 (0-100) = Manual mode with speed percentage.
    /// 0xFFFF = Temperature-based automatic mode.
    ManualSpeedControl = 0x0003,

    /// Combined start/full speed temperature (UINT16, Read/Write).
    ///
    /// High byte: Start temperature (L).
    /// Low byte: Full speed temperature (H).
    /// Both use +40 offset.
    CombinedTemperature = 0x0004,

    /// Work mode / ECO mode (UINT16, Read/Write).
    ///
    /// 0x0000 = Shutdown mode (fan stops below L-3°C).
    /// 0x0001 = Minimum speed mode (20% below L-3°C).
    WorkMode = 0x0005,

    /// Number of fans connected (UINT16, Read/Write).
    ///
    /// Range: 0x0001-0x0004 (1-4 fans).
    /// 0x0000 = Disable fault detection.
    FanQuantity = 0x0006,

    /// Fan 1 speed in RPM (UINT16, Read-only).
    Fan1Speed = 0x0007,

    /// Fan 2 speed in RPM (UINT16, Read-only).
    Fan2Speed = 0x0008,

    /// Fan 3 speed in RPM (UINT16, Read-only).
    Fan3Speed = 0x0009,

    /// Fan 4 speed in RPM (UINT16, Read-only).
    Fan4Speed = 0x000A,

    /// PWM frequency selection (UINT16, Read/Write).
    ///
    /// 0x0000=500Hz, 0x0001=1kHz, 0x0002=2kHz,
    /// 0x0003=5kHz, 0x0004=10kHz, 0x0005=25kHz (default).
    PwmFrequency = 0x000B,

    /// Start temperature threshold (INT16, Read/Write).
    ///
    /// Temperature where fans start spinning. Stored with +40 offset.
    /// Range: 0x0014-0x00A0 (-20°C to 120°C).
    StartTemperature = 0x000C,

    /// Full speed temperature threshold (INT16, Read/Write).
    ///
    /// Temperature where fans reach 100% speed. Stored with +40 offset.
    /// Must be greater than start temperature.
    /// Range: 0x0014-0x00A0 (-20°C to 120°C).
    FullSpeedTemperature = 0x000D,

    /// Fan fault code bitmap (BITMAP, Read-only).
    ///
    /// Bit 0=Fan1, Bit 1=Fan2, Bit 2=Fan3, Bit 3=Fan4.
    /// 1=normal, 0=fault (inverted logic).
    FanFaultCode = 0x000E,

    /// Reset controller command (UINT16, Write-only).
    ///
    /// Write 0x00AA to reset/restart the controller.
    ResetController = 0x0020,
}

impl RegisterAddress {
    /// Returns the numeric register address.
    ///
    /// # Examples
    ///
    /// ```
    /// # use jpf4826_driver::registers::RegisterAddress;
    /// assert_eq!(RegisterAddress::CurrentTemperature.addr(), 0x0000);
    /// assert_eq!(RegisterAddress::ResetController.addr(), 0x0020);
    /// ```
    pub fn addr(self) -> u16 {
        self as u16
    }

    /// Returns the register address for a specific fan's RPM.
    ///
    /// # Examples
    ///
    /// ```
    /// # use jpf4826_driver::registers::RegisterAddress;
    /// assert_eq!(RegisterAddress::fan_speed_register(1), Some(RegisterAddress::Fan1Speed));
    /// assert_eq!(RegisterAddress::fan_speed_register(4), Some(RegisterAddress::Fan4Speed));
    /// assert_eq!(RegisterAddress::fan_speed_register(5), None);
    /// ```
    pub fn fan_speed_register(fan_index: u8) -> Option<Self> {
        match fan_index {
            1 => Some(RegisterAddress::Fan1Speed),
            2 => Some(RegisterAddress::Fan2Speed),
            3 => Some(RegisterAddress::Fan3Speed),
            4 => Some(RegisterAddress::Fan4Speed),
            _ => None,
        }
    }
}
