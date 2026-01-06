//! Error types for JPF4826 driver operations.
//!
//! This module defines structured error types for all failure modes
//! in the driver, following best practices for error handling.

// Rust guideline compliant 2026-01-06

use std::backtrace::Backtrace;
use std::fmt;

/// Result type alias for JPF4826 driver operations.
pub type Result<T> = std::result::Result<T, Jpf4826Error>;

/// Error type for JPF4826 driver operations.
///
/// This structured error type captures all failure modes with
/// contextual information and backtrace for debugging.
#[derive(Debug)]
pub struct Jpf4826Error {
    kind: ErrorKind,
    backtrace: Backtrace,
}

/// Internal error classification.
#[derive(Debug)]
pub(crate) enum ErrorKind {
    /// Modbus protocol communication error.
    Modbus(String),
    /// Serial port communication error.
    Serial(String),
    /// Invalid parameter provided to API.
    InvalidParameter(String),
    /// Temperature threshold constraint violation.
    InvalidThresholds { low: i16, high: i16 },
    /// Fan index out of valid range (1-4).
    InvalidFanIndex(u8),
    /// Modbus address out of valid range (1-254).
    InvalidAddress(u8),
    /// Manual speed percentage out of valid range (0-100).
    InvalidSpeed(u8),
}

impl Jpf4826Error {
    /// Creates error for Modbus communication failure.
    pub(crate) fn modbus<E: fmt::Display>(err: E) -> Self {
        Self {
            kind: ErrorKind::Modbus(err.to_string()),
            backtrace: Backtrace::capture(),
        }
    }

    /// Creates error for serial port failure.
    pub(crate) fn serial<E: fmt::Display>(err: E) -> Self {
        Self {
            kind: ErrorKind::Serial(err.to_string()),
            backtrace: Backtrace::capture(),
        }
    }

    /// Creates error for invalid parameter.
    pub(crate) fn invalid_parameter<S: Into<String>>(msg: S) -> Self {
        Self {
            kind: ErrorKind::InvalidParameter(msg.into()),
            backtrace: Backtrace::capture(),
        }
    }

    /// Creates error for invalid temperature thresholds.
    pub(crate) fn invalid_thresholds(low: i16, high: i16) -> Self {
        Self {
            kind: ErrorKind::InvalidThresholds { low, high },
            backtrace: Backtrace::capture(),
        }
    }

    /// Creates error for invalid fan index.
    pub(crate) fn new_invalid_fan_index(index: u8) -> Self {
        Self {
            kind: ErrorKind::InvalidFanIndex(index),
            backtrace: Backtrace::capture(),
        }
    }

    /// Creates error for invalid Modbus address.
    pub(crate) fn invalid_address(addr: u8) -> Self {
        Self {
            kind: ErrorKind::InvalidAddress(addr),
            backtrace: Backtrace::capture(),
        }
    }

    /// Creates error for invalid speed percentage.
    pub(crate) fn invalid_speed(speed: u8) -> Self {
        Self {
            kind: ErrorKind::InvalidSpeed(speed),
            backtrace: Backtrace::capture(),
        }
    }

    /// Returns true if error is due to Modbus communication.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::{Jpf4826Client, Result};
    /// # async fn example() -> Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// match client.temperature().await {
    ///     Err(e) if e.is_modbus() => println!("Modbus communication failed"),
    ///     Err(e) if e.is_serial() => println!("Serial port error"),
    ///     Err(e) => println!("Other error: {}", e),
    ///     Ok(temp) => println!("Temperature: {}°C", temp.value),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_modbus(&self) -> bool {
        matches!(self.kind, ErrorKind::Modbus(_))
    }

    /// Returns true if error is due to serial port failure.
    pub fn is_serial(&self) -> bool {
        matches!(self.kind, ErrorKind::Serial(_))
    }

    /// Returns true if error is due to invalid parameter.
    pub fn is_invalid_parameter(&self) -> bool {
        matches!(self.kind, ErrorKind::InvalidParameter(_))
    }

    /// Returns the fan index if error is due to invalid fan index.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jpf4826_driver::{Jpf4826Client, Result};
    /// # async fn example() -> Result<()> {
    /// # let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
    /// if let Err(e) = client.fan_speed(5).await {
    ///     if let Some(index) = e.invalid_fan_index() {
    ///         println!("Fan index {} is out of range (1-4)", index);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn invalid_fan_index(&self) -> Option<u8> {
        if let ErrorKind::InvalidFanIndex(index) = self.kind {
            Some(index)
        } else {
            None
        }
    }

    /// Returns the backtrace for debugging.
    ///
    /// Set `RUST_BACKTRACE=1` environment variable to capture backtraces.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl fmt::Display for Jpf4826Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Modbus(msg) => write!(f, "Modbus communication error: {}", msg),
            ErrorKind::Serial(msg) => write!(f, "Serial port error: {}", msg),
            ErrorKind::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            ErrorKind::InvalidThresholds { low, high } => {
                write!(
                    f,
                    "Temperature threshold error: high ({}°C) must be greater than low ({}°C)",
                    high, low
                )
            }
            ErrorKind::InvalidFanIndex(index) => {
                write!(f, "Fan index {} out of range (1-4)", index)
            }
            ErrorKind::InvalidAddress(addr) => {
                write!(f, "Modbus address {} out of range (1-254)", addr)
            }
            ErrorKind::InvalidSpeed(speed) => {
                write!(f, "Manual speed {}% out of range (0-100)", speed)
            }
        }
    }
}

impl std::error::Error for Jpf4826Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
