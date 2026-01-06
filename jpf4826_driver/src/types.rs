//! Core types for JPF4826 fan controller protocol.
//!
//! This module defines type-safe representations for controller modes,
//! statuses, and data structures matching the Modbus register protocol.

// Rust guideline compliant 2026-01-06

use serde::{Deserialize, Serialize};

/// Operating mode for fan speed control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OperatingMode {
    /// Automatic speed control based on temperature sensor.
    Temperature,
    /// Manual speed control with fixed percentage.
    Manual,
}

impl OperatingMode {
    /// Converts to Modbus register value for mode setting.
    ///
    /// # Examples
    ///
    /// ```
    /// # use jpf4826_driver::types::OperatingMode;
    /// assert_eq!(OperatingMode::Temperature.to_register_value(), 0xFFFF);
    /// ```
    pub fn to_register_value(self) -> u16 {
        match self {
            OperatingMode::Temperature => 0xFFFF,
            OperatingMode::Manual => 0x0000, // Actual speed set via separate register
        }
    }
}

/// Work mode determining fan behavior below start temperature.
///
/// This is also known as ECO mode in the controller documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkMode {
    /// Fan stops completely below (low_threshold - 3°C).
    Shutdown,
    /// Fan maintains 20% speed below (low_threshold - 3°C).
    MinimumSpeed,
}

impl WorkMode {
    /// Converts to Modbus register value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use jpf4826_driver::types::WorkMode;
    /// assert_eq!(WorkMode::Shutdown.to_register_value(), 0x0000);
    /// assert_eq!(WorkMode::MinimumSpeed.to_register_value(), 0x0001);
    /// ```
    pub fn to_register_value(self) -> u16 {
        match self {
            WorkMode::Shutdown => 0x0000,
            WorkMode::MinimumSpeed => 0x0001,
        }
    }

    /// Creates WorkMode from Modbus register value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use jpf4826_driver::types::WorkMode;
    /// assert_eq!(WorkMode::from_register_value(0x0000), Some(WorkMode::Shutdown));
    /// assert_eq!(WorkMode::from_register_value(0x0001), Some(WorkMode::MinimumSpeed));
    /// assert_eq!(WorkMode::from_register_value(0x0002), None);
    /// ```
    pub fn from_register_value(value: u16) -> Option<Self> {
        match value {
            0x0000 => Some(WorkMode::Shutdown),
            0x0001 => Some(WorkMode::MinimumSpeed),
            _ => None,
        }
    }
}

/// Fan operational status from controller diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FanStatus {
    /// Fan operating correctly.
    Normal,
    /// Fan fault detected.
    Fault,
}

/// Temperature unit for display and conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TemperatureUnit {
    /// Celsius temperature scale.
    Celsius,
    /// Fahrenheit temperature scale.
    Fahrenheit,
}

/// PWM frequency for fan control signal.
///
/// JPF4826 supports six fixed frequency options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PwmFrequency {
    /// 500 Hz PWM frequency.
    Hz500,
    /// 1000 Hz PWM frequency.
    Hz1000,
    /// 2000 Hz PWM frequency.
    Hz2000,
    /// 5000 Hz PWM frequency.
    Hz5000,
    /// 10000 Hz PWM frequency.
    Hz10000,
    /// 25000 Hz PWM frequency (default).
    Hz25000,
}

impl PwmFrequency {
    /// Converts to Modbus register value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use jpf4826_driver::types::PwmFrequency;
    /// assert_eq!(PwmFrequency::Hz500.to_register_value(), 0x0000);
    /// assert_eq!(PwmFrequency::Hz25000.to_register_value(), 0x0005);
    /// ```
    pub fn to_register_value(self) -> u16 {
        match self {
            PwmFrequency::Hz500 => 0x0000,
            PwmFrequency::Hz1000 => 0x0001,
            PwmFrequency::Hz2000 => 0x0002,
            PwmFrequency::Hz5000 => 0x0003,
            PwmFrequency::Hz10000 => 0x0004,
            PwmFrequency::Hz25000 => 0x0005,
        }
    }

    /// Creates PwmFrequency from Modbus register value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use jpf4826_driver::types::PwmFrequency;
    /// assert_eq!(PwmFrequency::from_register_value(0x0000), Some(PwmFrequency::Hz500));
    /// assert_eq!(PwmFrequency::from_register_value(0x0006), None);
    /// ```
    pub fn from_register_value(value: u16) -> Option<Self> {
        match value {
            0x0000 => Some(PwmFrequency::Hz500),
            0x0001 => Some(PwmFrequency::Hz1000),
            0x0002 => Some(PwmFrequency::Hz2000),
            0x0003 => Some(PwmFrequency::Hz5000),
            0x0004 => Some(PwmFrequency::Hz10000),
            0x0005 => Some(PwmFrequency::Hz25000),
            _ => None,
        }
    }

    /// Returns frequency value in Hertz.
    ///
    /// # Examples
    ///
    /// ```
    /// # use jpf4826_driver::types::PwmFrequency;
    /// assert_eq!(PwmFrequency::Hz25000.to_hz(), 25000);
    /// ```
    pub fn to_hz(self) -> u32 {
        match self {
            PwmFrequency::Hz500 => 500,
            PwmFrequency::Hz1000 => 1000,
            PwmFrequency::Hz2000 => 2000,
            PwmFrequency::Hz5000 => 5000,
            PwmFrequency::Hz10000 => 10000,
            PwmFrequency::Hz25000 => 25000,
        }
    }

    /// Creates PwmFrequency from Hertz value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use jpf4826_driver::types::PwmFrequency;
    /// assert_eq!(PwmFrequency::from_hz(25000), Some(PwmFrequency::Hz25000));
    /// assert_eq!(PwmFrequency::from_hz(3000), None);
    /// ```
    pub fn from_hz(hz: u32) -> Option<Self> {
        match hz {
            500 => Some(PwmFrequency::Hz500),
            1000 => Some(PwmFrequency::Hz1000),
            2000 => Some(PwmFrequency::Hz2000),
            5000 => Some(PwmFrequency::Hz5000),
            10000 => Some(PwmFrequency::Hz10000),
            25000 => Some(PwmFrequency::Hz25000),
            _ => None,
        }
    }
}

/// Temperature reading with associated unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Temperature {
    /// Temperature value.
    pub value: i16,
    /// Temperature unit.
    pub unit: TemperatureUnit,
}

/// Individual fan status and speed information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FanInfo {
    /// Fan index (1-4).
    pub index: u8,
    /// Operational status.
    pub status: FanStatus,
    /// Rotation speed in RPM.
    pub rpm: u16,
}

/// Complete controller status snapshot.
///
/// This structure mirrors the JSON schema defined in
/// `schemas/jpf4826-status-response.schema.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControllerStatus {
    /// Current operating mode.
    pub mode: OperatingMode,
    /// ECO mode enabled (true = minimum speed, false = shutdown).
    pub eco_mode: bool,
    /// Modbus address (1-254).
    pub modbus_address: u8,
    /// PWM frequency setting.
    pub pwm_frequency: PwmFrequency,
    /// Number of fans configured (0-4, 0 = fault detection disabled).
    pub fan_count: u8,
    /// Current temperature reading.
    pub temperature_current: Temperature,
    /// Temperature threshold where fans start spinning.
    pub temperature_low_threshold: Temperature,
    /// Temperature threshold where fans reach 100% speed.
    pub temperature_high_threshold: Temperature,
    /// Status of individual fans.
    pub fans: Vec<FanInfo>,
}
