//! Modbus-RTU communication layer.
//!
//! This module provides the low-level Modbus-RTU protocol implementation
//! over serial port connection.

// Rust guideline compliant 2026-01-27

use crate::error::{Jpf4826Error, Result};
use std::time::Duration;
use tokio_modbus::client::Context;
use tokio_modbus::prelude::*;
use tokio_serial::SerialStream;

/// Default timeout for Modbus operations (10 seconds).
///
/// This value is used when no timeout is specified during client initialization.
/// The timeout applies to each individual Modbus read/write operation.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Modbus-RTU client for JPF4826 controller.
pub struct ModbusRtuClient {
    context: Context,
    slave_addr: std::cell::Cell<u8>,
    timeout: Duration,
}

impl ModbusRtuClient {
    /// Creates a new Modbus-RTU client connected to the specified serial port.
    ///
    /// # Arguments
    ///
    /// * `port` - Serial port path (e.g., "/dev/ttyUSB0", "COM3")
    /// * `slave_addr` - Modbus slave address (1-254)
    ///
    /// # Serial Port Configuration
    ///
    /// - Baud rate: 9600
    /// - Data bits: 8
    /// - Parity: None
    /// - Stop bits: 1
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Serial port cannot be opened
    /// - Port configuration fails
    pub async fn new(port: &str, slave_addr: u8) -> Result<Self> {
        log::debug!(
            "Initializing Modbus-RTU client: port={}, slave_addr={}",
            port,
            slave_addr
        );

        // Configure serial port according to JPF4826 specification
        log::debug!("Configuring serial port: 9600 8N1, no flow control");
        let builder = tokio_serial::new(port, 9600)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One)
            .flow_control(tokio_serial::FlowControl::None);

        // Open serial port
        log::debug!("Opening serial port: {}", port);
        let serial = SerialStream::open(&builder).map_err(|e| {
            log::error!("Failed to open serial port {}: {}", port, e);
            Jpf4826Error::serial(format!("Failed to open serial port {}: {}", port, e))
        })?;
        log::debug!("Serial port opened successfully");

        // Create Modbus-RTU context
        log::debug!("Attaching Modbus-RTU context to slave {}", slave_addr);
        let context = rtu::attach_slave(serial, Slave(slave_addr));

        log::debug!("Modbus-RTU client initialized successfully");
        Ok(Self {
            context,
            slave_addr: std::cell::Cell::new(slave_addr),
            timeout: DEFAULT_TIMEOUT,
        })
    }

    /// Returns the current operation timeout.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Sets the timeout for Modbus operations.
    ///
    /// This affects all subsequent read and write operations.
    // TODO: Consider validating that timeout is not zero to prevent immediate timeout errors.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Reads holding registers from the controller.
    ///
    /// # Arguments
    ///
    /// * `addr` - Starting register address
    /// * `count` - Number of consecutive registers to read
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Modbus communication fails
    /// - Operation times out
    pub async fn read_holding_registers(&mut self, addr: u16, count: u16) -> Result<Vec<u16>> {
        log::debug!(
            "Modbus READ: addr=0x{:04X}, count={}, timeout={:?}",
            addr,
            count,
            self.timeout
        );

        let operation = self.context.read_holding_registers(addr, count);

        let result = tokio::time::timeout(self.timeout, operation)
            .await
            .map_err(|_| {
                log::error!(
                    "Modbus READ timed out at 0x{:04X} after {:?}",
                    addr,
                    self.timeout
                );
                Jpf4826Error::timeout(self.timeout)
            })?
            .map_err(|e| {
                log::error!("Modbus READ failed at 0x{:04X}: {}", addr, e);
                Jpf4826Error::modbus(format!("Failed to read registers at 0x{:04X}: {}", addr, e))
            })?
            .map_err(|e| {
                log::error!("Modbus exception at 0x{:04X}: {:?}", addr, e);
                Jpf4826Error::modbus(format!("Modbus exception at 0x{:04X}: {:?}", addr, e))
            })?;

        log::debug!(
            "Modbus READ success: addr=0x{:04X}, values={:04X?}",
            addr,
            result
        );
        Ok(result)
    }

    /// Writes a single holding register to the controller.
    ///
    /// # Arguments
    ///
    /// * `addr` - Register address
    /// * `value` - 16-bit value to write
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Modbus communication fails
    /// - Operation times out
    pub async fn write_single_register(&mut self, addr: u16, value: u16) -> Result<()> {
        log::debug!(
            "Modbus WRITE: addr=0x{:04X}, value=0x{:04X}, timeout={:?}",
            addr,
            value,
            self.timeout
        );

        let operation = self.context.write_single_register(addr, value);

        tokio::time::timeout(self.timeout, operation)
            .await
            .map_err(|_| {
                log::error!(
                    "Modbus WRITE timed out at 0x{:04X} after {:?}",
                    addr,
                    self.timeout
                );
                Jpf4826Error::timeout(self.timeout)
            })?
            .map_err(|e| {
                log::error!("Modbus WRITE failed at 0x{:04X}: {}", addr, e);
                Jpf4826Error::modbus(format!("Failed to write register 0x{:04X}: {}", addr, e))
            })?
            .map_err(|e| {
                log::error!("Modbus exception at 0x{:04X}: {:?}", addr, e);
                Jpf4826Error::modbus(format!("Modbus exception at 0x{:04X}: {:?}", addr, e))
            })?;

        log::debug!("Modbus WRITE success: addr=0x{:04X}", addr);
        Ok(())
    }

    /// Returns the configured slave address.
    #[allow(dead_code)]
    pub fn slave_addr(&self) -> u8 {
        self.slave_addr.get()
    }

    /// Updates the configured slave address.
    ///
    /// This method should be called after successfully writing a new address
    /// to the controller's Modbus address register to keep the client in sync.
    pub(crate) fn set_slave_addr(&self, addr: u8) {
        self.slave_addr.set(addr);
    }
}
