//! Core types for JPF4826 fan controller protocol.
//!
//! This module defines type-safe representations for controller modes,
//! statuses, and data structures matching the Modbus register protocol.

// Rust guideline compliant 2026-01-16

use serde::{Deserialize, Serialize};

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
///
/// # JSON Serialization
///
/// Serializes to/from JSON object format:
/// ```json
/// {"value": 25000, "unit": "Hz"}
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

// Custom serde implementations to match JSON schema format
impl serde::Serialize for PwmFrequency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("PwmFrequency", 2)?;
        state.serialize_field("value", &self.to_hz())?;
        state.serialize_field("unit", "Hz")?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for PwmFrequency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PwmFrequencyHelper {
            value: u32,
            #[allow(dead_code)]
            unit: String,
        }

        let helper = PwmFrequencyHelper::deserialize(deserializer)?;
        PwmFrequency::from_hz(helper.value).ok_or_else(|| {
            serde::de::Error::custom(format!("Invalid PWM frequency: {}", helper.value))
        })
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
///
/// # JSON Serialization
///
/// Temperature fields are serialized as a nested object:
/// ```json
/// {
///   "temperature": {
///     "current": {...},
///     "low_threshold": {...},
///     "high_threshold": {...}
///   }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ControllerStatus {
    /// ECO mode enabled (true = shutdown mode, false = minimum speed mode).
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

// Custom serde implementations to match JSON schema format
impl serde::Serialize for ControllerStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ControllerStatus", 6)?;
        state.serialize_field("eco_mode", &self.eco_mode)?;
        state.serialize_field("modbus_address", &self.modbus_address)?;
        state.serialize_field("pwm_frequency", &self.pwm_frequency)?;
        state.serialize_field("fan_count", &self.fan_count)?;

        // Nest temperature fields under "temperature" key
        #[derive(Serialize)]
        struct TemperatureNested {
            current: Temperature,
            low_threshold: Temperature,
            high_threshold: Temperature,
        }

        let temp_nested = TemperatureNested {
            current: self.temperature_current,
            low_threshold: self.temperature_low_threshold,
            high_threshold: self.temperature_high_threshold,
        };
        state.serialize_field("temperature", &temp_nested)?;
        state.serialize_field("fans", &self.fans)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for ControllerStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TemperatureNested {
            current: Temperature,
            low_threshold: Temperature,
            high_threshold: Temperature,
        }

        #[derive(Deserialize)]
        struct ControllerStatusHelper {
            eco_mode: bool,
            modbus_address: u8,
            pwm_frequency: PwmFrequency,
            fan_count: u8,
            temperature: TemperatureNested,
            fans: Vec<FanInfo>,
        }

        let helper = ControllerStatusHelper::deserialize(deserializer)?;
        Ok(ControllerStatus {
            eco_mode: helper.eco_mode,
            modbus_address: helper.modbus_address,
            pwm_frequency: helper.pwm_frequency,
            fan_count: helper.fan_count,
            temperature_current: helper.temperature.current,
            temperature_low_threshold: helper.temperature.low_threshold,
            temperature_high_threshold: helper.temperature.high_threshold,
            fans: helper.fans,
        })
    }
}
