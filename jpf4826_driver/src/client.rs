//! JPF4826 client implementation for Modbus-RTU communication.
//!
//! This module provides the main client interface for interacting with
//! JPF4826 fan controllers via serial Modbus-RTU protocol.

// Rust guideline compliant 2026-01-06

use crate::{
    conversions::*,
    error::{Jpf4826Error, Result},
    registers::RegisterAddress,
    types::*,
};

/// JPF4826 fan controller client.
///
/// Provides high-level and low-level APIs for reading and writing
/// controller registers via Modbus-RTU over serial connection.
///
/// # Examples
///
/// ```no_run
/// # use jpf4826_driver::Jpf4826Client;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Connect to controller at /dev/ttyUSB0, Modbus address 1
/// let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
///
/// // Read current temperature
/// let temp = client.temperature().await?;
/// println!("Temperature: {}°C", temp.value);
///
/// // Get full status
/// let status = client.status().await?;
/// println!("Fan count: {}", status.fan_count);
/// # Ok(())
/// # }
/// ```
pub struct Jpf4826Client {
    backend: ClientBackend,
}

/// Internal backend abstraction for testing.
enum ClientBackend {
    #[cfg(any(test, feature = "test-mock"))]
    Mock(MockBackend),
    RealModbus(crate::modbus::ModbusRtuClient),
}

#[cfg(any(test, feature = "test-mock"))]
pub(crate) struct MockBackend {
    pub controller: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<u16, u16>>>,
    slave_addr: std::cell::Cell<u8>,
}

#[cfg(any(test, feature = "test-mock"))]
impl MockBackend {
    fn read_registers(&self, start_addr: u16, count: u16) -> Vec<u16> {
        let registers = self.controller.lock().unwrap();
        (start_addr..start_addr + count)
            .map(|addr| registers.get(&addr).copied().unwrap_or(0))
            .collect()
    }

    fn set_slave_addr(&self, addr: u8) {
        self.slave_addr.set(addr);
    }

    pub(crate) fn slave_addr(&self) -> u8 {
        self.slave_addr.get()
    }
}

impl Jpf4826Client {
    /// Creates a new client connected to the specified serial port.
    ///
    /// # Arguments
    ///
    /// * `port` - Serial port path (e.g., "/dev/ttyUSB0", "COM3")
    /// * `slave_addr` - Modbus slave address (1-254)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::Jpf4826Client;
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// let client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Serial port cannot be opened
    /// - Modbus address is out of range (1-254)
    pub async fn new(port: &str, slave_addr: u8) -> Result<Self> {
        if !(1..=254).contains(&slave_addr) {
            return Err(Jpf4826Error::invalid_address(slave_addr));
        }

        let modbus_client = crate::modbus::ModbusRtuClient::new(port, slave_addr).await?;
        Ok(Self {
            backend: ClientBackend::RealModbus(modbus_client),
        })
    }

    /// Creates a mock client for testing (test-only).
    #[doc(hidden)]
    #[cfg(any(test, feature = "test-mock"))]
    pub async fn new_mock(
        registers: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<u16, u16>>>,
        slave_addr: u8,
    ) -> Self {
        Self {
            backend: ClientBackend::Mock(MockBackend {
                controller: registers,
                slave_addr: std::cell::Cell::new(slave_addr),
            }),
        }
    }

    /// Reads holding registers from the controller.
    ///
    /// Low-level method for reading raw register values. Most users should
    /// use the high-level methods like `temperature()` or `status()` instead.
    ///
    /// # Arguments
    ///
    /// * `register` - Starting register address
    /// * `count` - Number of consecutive registers to read
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::{Jpf4826Client, registers::RegisterAddress};
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// // Read temperature register
    /// let values = client.read(RegisterAddress::CurrentTemperature, 1).await?;
    /// println!("Raw temperature value: {}", values[0]);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if Modbus communication fails.
    pub async fn read(&mut self, register: RegisterAddress, count: u16) -> Result<Vec<u16>> {
        match &mut self.backend {
            #[cfg(any(test, feature = "test-mock"))]
            ClientBackend::Mock(mock) => Ok(mock.read_registers(register.addr(), count)),
            ClientBackend::RealModbus(modbus) => {
                modbus.read_holding_registers(register.addr(), count).await
            }
        }
    }

    /// Reads current temperature from the controller.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::Jpf4826Client;
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// let temp = client.temperature().await?;
    /// println!("Current: {}°C", temp.value);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if Modbus communication fails.
    pub async fn temperature(&mut self) -> Result<Temperature> {
        let values = self.read(RegisterAddress::CurrentTemperature, 1).await?;
        let celsius = register_to_celsius(values[0]);

        Ok(Temperature {
            value: celsius,
            unit: TemperatureUnit::Celsius,
        })
    }

    /// Reads speed of a specific fan in RPM.
    ///
    /// # Arguments
    ///
    /// * `index` - Fan number (1-4)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::Jpf4826Client;
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// let rpm = client.fan_speed(1).await?;
    /// println!("Fan 1: {} RPM", rpm);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Fan index is out of range (1-4)
    /// - Modbus communication fails
    pub async fn fan_speed(&mut self, index: u8) -> Result<u16> {
        let register = RegisterAddress::fan_speed_register(index)
            .ok_or_else(|| Jpf4826Error::new_invalid_fan_index(index))?;

        let values = self.read(register, 1).await?;
        Ok(values[0])
    }

    /// Reads the configured number of fans.
    ///
    /// Returns 0 if fault detection is disabled.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::Jpf4826Client;
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// let count = client.fan_count().await?;
    /// println!("Configured fans: {}", count);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if Modbus communication fails.
    pub async fn fan_count(&mut self) -> Result<u8> {
        let values = self.read(RegisterAddress::FanQuantity, 1).await?;
        Ok(values[0] as u8)
    }

    /// Reads status of all fans (running state, faults, speeds).
    ///
    /// Returns information for all 4 fan slots regardless of configured
    /// fan count. Check `fan_count()` to determine how many are active.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::Jpf4826Client;
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// let fans = client.fan_status().await?;
    /// for fan in fans {
    ///     println!("Fan {}: {:?} @ {} RPM", fan.index, fan.status, fan.rpm);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if Modbus communication fails.
    pub async fn fan_status(&mut self) -> Result<Vec<FanInfo>> {
        log::debug!("Reading fan status and speeds");

        // Read: fan status bitmap (0x0001), fan speeds (0x0007-0x000A), fault bitmap (0x000E)
        // We need separate reads since registers aren't consecutive
        log::debug!("Reading fan status bitmap from register 0x0001");
        let _status_bitmap = self.read(RegisterAddress::FanStatus, 1).await?[0];
        log::debug!("Status bitmap: {:#06X}", _status_bitmap);

        log::debug!("Reading fan speeds from registers 0x0007-0x000A");
        let speeds = self.read(RegisterAddress::Fan1Speed, 4).await?;
        log::debug!("Fan speeds: {:?} RPM", speeds);

        log::debug!("Reading fault bitmap from register 0x000E");
        let fault_bitmap = self.read(RegisterAddress::FanFaultCode, 1).await?[0];
        log::debug!("Fault bitmap: {:#06X}", fault_bitmap);

        let fault_statuses = parse_fan_fault_bitmap(fault_bitmap);

        let mut fans = Vec::with_capacity(4);
        for i in 0..4 {
            fans.push(FanInfo {
                index: (i + 1) as u8,
                status: fault_statuses[i],
                rpm: speeds[i],
            });
        }

        log::debug!("Assembled {} fan info entries", fans.len());
        Ok(fans)
    }

    /// Reads complete controller status.
    ///
    /// This method performs a bulk read of all status registers and
    /// assembles them into a comprehensive status structure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::Jpf4826Client;
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// let status = client.status().await?;
    /// println!("Mode: {:?}", status.mode);
    /// println!("Temperature: {}°C", status.temperature_current.value);
    /// println!("Fans: {}", status.fan_count);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if Modbus communication fails.
    pub async fn status(&mut self) -> Result<ControllerStatus> {
        log::debug!("Reading controller status (15 registers starting from 0x0000)");

        // Read all status registers at once (0x0000-0x000E = 15 registers)
        let values = self.read(RegisterAddress::CurrentTemperature, 15).await?;
        log::debug!("Received {} register values", values.len());
        log::debug!("Raw register values: {:04X?}", values);

        let current_temp = register_to_celsius(values[0]);
        let modbus_address = values[2] as u8;
        let manual_speed_raw = values[3];
        let work_mode_raw = values[5];
        let fan_count = values[6] as u8;
        let pwm_freq_raw = values[11];
        let start_temp = register_to_celsius(values[12]);
        let full_temp = register_to_celsius(values[13]);

        log::debug!(
            "Parsed values: temp={}, addr={}, mode_raw={:#06X}, fans={}",
            current_temp,
            modbus_address,
            manual_speed_raw,
            fan_count
        );

        // Determine operating mode
        let mode = if manual_speed_raw == 0xFFFF {
            OperatingMode::Temperature
        } else {
            OperatingMode::Manual
        };

        // Determine ECO mode (work mode)
        let eco_mode = work_mode_raw == 1;

        // Parse PWM frequency
        let pwm_frequency =
            PwmFrequency::from_register_value(pwm_freq_raw).unwrap_or(PwmFrequency::Hz25000);

        log::debug!("Reading fan status for {} fans...", fan_count);
        // Get fan status
        let fans = self.fan_status().await?;
        log::debug!("Fan status retrieved successfully");

        Ok(ControllerStatus {
            mode,
            eco_mode,
            modbus_address,
            pwm_frequency,
            fan_count,
            temperature_current: Temperature {
                value: current_temp,
                unit: TemperatureUnit::Celsius,
            },
            temperature_low_threshold: Temperature {
                value: start_temp,
                unit: TemperatureUnit::Celsius,
            },
            temperature_high_threshold: Temperature {
                value: full_temp,
                unit: TemperatureUnit::Celsius,
            },
            fans,
        })
    }

    // === Write Operations ===

    /// Writes a single holding register to the controller.
    ///
    /// Low-level method for writing raw register values. Most users should
    /// use the high-level methods like `set_mode()` or `reset()` instead.
    ///
    /// The Modbus protocol validates the write by verifying the controller
    /// echoes back the same register address and value.
    ///
    /// # Arguments
    ///
    /// * `register` - Register address to write
    /// * `value` - 16-bit value to write
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Modbus communication fails
    /// - Controller response is invalid or does not match the written value
    pub async fn write(&mut self, register: RegisterAddress, value: u16) -> Result<()> {
        match &mut self.backend {
            #[cfg(any(test, feature = "test-mock"))]
            ClientBackend::Mock(mock) => {
                mock.controller
                    .lock()
                    .unwrap()
                    .insert(register.addr(), value);
                Ok(())
            }
            ClientBackend::RealModbus(modbus) => {
                modbus.write_single_register(register.addr(), value).await
            }
        }
    }

    /// Resets the controller.
    ///
    /// Sends the reset command (0x00AA) to register 0x0020.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::Jpf4826Client;
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// client.reset().await?;
    /// println!("Controller reset");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if Modbus communication fails.
    pub async fn reset(&mut self) -> Result<()> {
        self.write(RegisterAddress::ResetController, 0x00AA).await
    }

    /// Sets the operating mode (Temperature or Manual).
    ///
    /// In temperature mode, fan speed is controlled automatically based on
    /// temperature sensor readings. In manual mode, use `set_fan_speed()`
    /// to control speed directly.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::{Jpf4826Client, OperatingMode};
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// client.set_mode(OperatingMode::Temperature).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if Modbus communication fails.
    pub async fn set_mode(&mut self, mode: OperatingMode) -> Result<()> {
        let value = mode.to_register_value();
        self.write(RegisterAddress::ManualSpeedControl, value).await
    }

    /// Sets the ECO/work mode.
    ///
    /// Determines fan behavior when temperature falls below (start_temp - 3°C).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::{Jpf4826Client, WorkMode};
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// // Fans maintain 20% speed when below start temperature
    /// client.set_eco(WorkMode::MinimumSpeed).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if Modbus communication fails.
    pub async fn set_eco(&mut self, mode: WorkMode) -> Result<()> {
        let value = mode.to_register_value();
        self.write(RegisterAddress::WorkMode, value).await
    }

    /// Sets manual fan speed percentage (Manual mode only).
    ///
    /// Speed percentage range: 0-100. Controller must be in Manual mode.
    ///
    /// # Arguments
    ///
    /// * `speed_percent` - Speed percentage (0-100)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::{Jpf4826Client, OperatingMode};
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// // Switch to manual mode and set 75% speed
    /// client.set_mode(OperatingMode::Manual).await?;
    /// client.set_fan_speed(75).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Speed is greater than 100
    /// - Modbus communication fails
    pub async fn set_fan_speed(&mut self, speed_percent: u8) -> Result<()> {
        if speed_percent > 100 {
            return Err(Jpf4826Error::invalid_speed(speed_percent));
        }
        self.write(RegisterAddress::ManualSpeedControl, speed_percent as u16)
            .await
    }

    /// Sets the number of fans connected to the controller.
    ///
    /// Valid range: 1-4. Set to 0 to disable fault detection.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of fans (0-4)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::Jpf4826Client;
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// client.set_fan_count(3).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Count is greater than 4
    /// - Modbus communication fails
    pub async fn set_fan_count(&mut self, count: u8) -> Result<()> {
        if count > 4 {
            return Err(Jpf4826Error::invalid_parameter(format!(
                "Fan count {} out of range (0-4)",
                count
            )));
        }
        self.write(RegisterAddress::FanQuantity, count as u16).await
    }

    /// Disables fan fault detection.
    ///
    /// Equivalent to calling `set_fan_count(0)`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::Jpf4826Client;
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// client.disable_fault_detection().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if Modbus communication fails.
    pub async fn disable_fault_detection(&mut self) -> Result<()> {
        self.set_fan_count(0).await
    }

    /// Sets the Modbus device address.
    ///
    /// Valid range: 1-254. The controller will respond to this address
    /// on subsequent Modbus requests.
    ///
    /// # Arguments
    ///
    /// * `addr` - New Modbus address (1-254)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::Jpf4826Client;
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// client.set_addr(5).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Address is 0 or greater than 254
    /// - Modbus communication fails
    pub async fn set_addr(&mut self, addr: u8) -> Result<()> {
        if !(1..=254).contains(&addr) {
            return Err(Jpf4826Error::invalid_address(addr));
        }
        self.write(RegisterAddress::ModbusAddress, addr as u16)
            .await?;

        // Update the client's internal address to match the controller
        match &self.backend {
            #[cfg(any(test, feature = "test-mock"))]
            ClientBackend::Mock(mock) => mock.set_slave_addr(addr),
            ClientBackend::RealModbus(modbus) => modbus.set_slave_addr(addr),
        }

        Ok(())
    }

    /// Sets the PWM frequency for fan control.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::{Jpf4826Client, PwmFrequency};
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// client.set_pwm_frequency(PwmFrequency::Hz25000).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if Modbus communication fails.
    pub async fn set_pwm_frequency(&mut self, freq: PwmFrequency) -> Result<()> {
        let value = freq.to_register_value();
        self.write(RegisterAddress::PwmFrequency, value).await
    }

    /// Sets temperature thresholds for automatic fan control.
    ///
    /// Fans start spinning at `low` temperature and reach 100% speed at
    /// `high` temperature. Constraintmust be: `high` > `low`.
    ///
    /// # Arguments
    ///
    /// * `low` - Start temperature in Celsius (-20 to 120)
    /// * `high` - Full speed temperature in Celsius (-20 to 120)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::Jpf4826Client;
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// // Start at 30°C, full speed at 50°C
    /// client.set_temperature_threshold(30, 50).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - `high` is not greater than `low`
    /// - Temperatures are out of range (-20 to 120°C)
    /// - Modbus communication fails
    pub async fn set_temperature_threshold(&mut self, low: i16, high: i16) -> Result<()> {
        // Validate constraint
        if high <= low {
            return Err(Jpf4826Error::invalid_thresholds(low, high));
        }

        // Validate range
        if !(-20..=120).contains(&low) {
            return Err(Jpf4826Error::invalid_parameter(format!(
                "Low temperature {}°C out of range (-20 to 120)",
                low
            )));
        }
        if !(-20..=120).contains(&high) {
            return Err(Jpf4826Error::invalid_parameter(format!(
                "High temperature {}°C out of range (-20 to 120)",
                high
            )));
        }

        // Write both registers
        let low_value = celsius_to_register(low);
        let high_value = celsius_to_register(high);

        self.write(RegisterAddress::StartTemperature, low_value)
            .await?;
        self.write(RegisterAddress::FullSpeedTemperature, high_value)
            .await?;

        Ok(())
    }

    /// Sets only the start (low) temperature threshold.
    ///
    /// The new low temperature must be less than the current high temperature.
    /// This method reads the current high threshold to validate the constraint.
    ///
    /// # Arguments
    ///
    /// * `low` - Start temperature in Celsius (-20 to 120)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::Jpf4826Client;
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// // Set only the start temperature to 25°C
    /// client.set_start_temperature(25).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Temperature is out of range (-20 to 120°C)
    /// - New low temperature is not less than current high temperature
    /// - Modbus communication fails
    pub async fn set_start_temperature(&mut self, low: i16) -> Result<()> {
        // Validate range
        if !(-20..=120).contains(&low) {
            return Err(Jpf4826Error::invalid_parameter(format!(
                "Start temperature {}°C out of range (-20 to 120)",
                low
            )));
        }

        // Read current high threshold to validate constraint
        let values = self.read(RegisterAddress::FullSpeedTemperature, 1).await?;
        let current_high = register_to_celsius(values[0]);

        // Validate constraint
        if low >= current_high {
            return Err(Jpf4826Error::invalid_thresholds(low, current_high));
        }

        // Write low temperature register
        let low_value = celsius_to_register(low);
        self.write(RegisterAddress::StartTemperature, low_value)
            .await?;

        Ok(())
    }

    /// Sets only the full speed (high) temperature threshold.
    ///
    /// The new high temperature must be greater than the current low temperature.
    /// This method reads the current low threshold to validate the constraint.
    ///
    /// # Arguments
    ///
    /// * `high` - Full speed temperature in Celsius (-20 to 120)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::Jpf4826Client;
    /// # #[tokio::main]
    /// # async fn main() -> jpf4826_driver::Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// // Set only the full speed temperature to 45°C
    /// client.set_full_speed_temperature(45).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Temperature is out of range (-20 to 120°C)
    /// - New high temperature is not greater than current low temperature
    /// - Modbus communication fails
    pub async fn set_full_speed_temperature(&mut self, high: i16) -> Result<()> {
        // Validate range
        if !(-20..=120).contains(&high) {
            return Err(Jpf4826Error::invalid_parameter(format!(
                "Full speed temperature {}°C out of range (-20 to 120)",
                high
            )));
        }

        // Read current low threshold to validate constraint
        let values = self.read(RegisterAddress::StartTemperature, 1).await?;
        let current_low = register_to_celsius(values[0]);

        // Validate constraint
        if high <= current_low {
            return Err(Jpf4826Error::invalid_thresholds(current_low, high));
        }

        // Write high temperature register
        let high_value = celsius_to_register(high);
        self.write(RegisterAddress::FullSpeedTemperature, high_value)
            .await?;

        Ok(())
    }

    /// Returns the current slave address (test-only helper).
    ///
    /// This method is only available when testing and allows verification
    /// that the client's internal address is properly synchronized after
    /// calling `set_addr()`.
    #[doc(hidden)]
    #[cfg(any(test, feature = "test-mock"))]
    pub fn slave_addr(&self) -> u8 {
        match &self.backend {
            #[cfg(any(test, feature = "test-mock"))]
            ClientBackend::Mock(mock) => mock.slave_addr(),
            ClientBackend::RealModbus(modbus) => modbus.slave_addr(),
        }
    }
}
