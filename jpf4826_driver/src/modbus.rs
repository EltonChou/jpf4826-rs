//! Modbus-RTU communication layer.
//!
//! This module provides the low-level Modbus-RTU protocol implementation
//! over serial port connection.

// Rust guideline compliant 2026-01-06

use crate::error::{Jpf4826Error, Result};
use tokio_modbus::client::Context;
use tokio_modbus::prelude::*;
use tokio_serial::SerialStream;

/// Modbus-RTU client for JPF4826 controller.
pub struct ModbusRtuClient {
    context: Context,
    slave_addr: std::cell::Cell<u8>,
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
        // Configure serial port according to JPF4826 specification
        let builder = tokio_serial::new(port, 9600)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One)
            .flow_control(tokio_serial::FlowControl::None);

        // Open serial port
        let serial = SerialStream::open(&builder).map_err(|e| {
            Jpf4826Error::serial(format!("Failed to open serial port {}: {}", port, e))
        })?;

        // Create Modbus-RTU context
        let context = rtu::attach_slave(serial, Slave(slave_addr));

        Ok(Self {
            context,
            slave_addr: std::cell::Cell::new(slave_addr),
        })
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
    /// Returns error if Modbus communication fails.
    pub async fn read_holding_registers(&mut self, addr: u16, count: u16) -> Result<Vec<u16>> {
        self.context
            .read_holding_registers(addr, count)
            .await
            .map_err(|e| {
                Jpf4826Error::modbus(format!("Failed to read registers at 0x{:04X}: {}", addr, e))
            })?
            .map_err(|e| {
                Jpf4826Error::modbus(format!("Modbus exception at 0x{:04X}: {:?}", addr, e))
            })
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
    /// Returns error if Modbus communication fails.
    pub async fn write_single_register(&mut self, addr: u16, value: u16) -> Result<()> {
        self.context
            .write_single_register(addr, value)
            .await
            .map_err(|e| {
                Jpf4826Error::modbus(format!("Failed to write register 0x{:04X}: {}", addr, e))
            })?
            .map_err(|e| {
                Jpf4826Error::modbus(format!("Modbus exception at 0x{:04X}: {:?}", addr, e))
            })
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
