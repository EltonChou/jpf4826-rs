//! Rust driver for JPF4826 4-channel PWM fan controller.
//!
//! This library provides a type-safe interface to control JPF4826 fan controllers
//! via Modbus-RTU protocol over RS485 serial connections.
//!
//! # Examples
//!
//! ```no_run
//! # use jpf4826_driver::Jpf4826Client;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to controller
//! let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;
//!
//! // Read status
//! let status = client.status().await?;
//! println!("Temperature: {}°C", status.temperature_current.value);
//! # Ok(())
//! # }
//! ```

// Rust guideline compliant 2026-01-27

#[doc(inline)]
pub use client::Jpf4826Client;
#[doc(inline)]
pub use error::{Jpf4826Error, Result};
#[doc(inline)]
pub use modbus::DEFAULT_TIMEOUT;
#[doc(inline)]
pub use types::*;

pub mod client;
pub mod conversions;
pub mod error;
mod modbus;
pub mod registers;
pub mod types;
