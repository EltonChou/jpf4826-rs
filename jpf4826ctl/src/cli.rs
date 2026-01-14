//! Command-line argument definitions using clap.

// Rust guideline compliant 2026-01-06

use clap::{Parser, Subcommand};

/// JPF4826 fan controller CLI utility.
#[derive(Parser, Debug)]
#[command(
    name = "jpf4826ctl",
    version,
    about = "Control JPF4826 fan controller via Modbus-RTU",
    long_about = None
)]
pub struct Cli {
    /// Serial port path (e.g., /dev/ttyUSB0, COM3)
    #[arg(
        short = 'p',
        long = "port",
        env = "JPF4826_PORT",
        help = "Serial port (falls back to JPF4826_PORT env var)"
    )]
    pub port: Option<String>,

    /// Modbus device address (1-254)
    #[arg(
        short = 'a',
        long = "addr",
        env = "JPF4826_ADDR",
        value_parser = clap::value_parser!(u8).range(1..=254),
        help = "Modbus address (falls back to JPF4826_ADDR env var)"
    )]
    pub addr: Option<u8>,

    /// Enable verbose logging (debug output)
    #[arg(short = 'v', long = "verbose", global = true)]
    pub verbose: bool,

    /// Command to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Display controller status
    Status {
        /// Output in JSON format
        #[arg(long)]
        json: bool,

        /// Temperature unit (0=Celsius, 1=Fahrenheit)
        #[arg(long, value_parser = clap::value_parser!(u8).range(0..=1))]
        temp_unit: Option<u8>,
    },

    /// Set controller registers
    Set {
        /// Operating mode (0=Temperature, 1=Manual)
        #[arg(long, value_parser = clap::value_parser!(u8).range(0..=1))]
        mode: Option<u8>,

        /// Modbus address (1-254)
        #[arg(long, value_parser = clap::value_parser!(u8).range(1..=254))]
        modbus_addr: Option<u8>,

        /// Start temperature threshold (-20 to 120°C)
        #[arg(long, value_parser = clap::value_parser!(i16).range(-20..=120))]
        low_temp: Option<i16>,

        /// Full speed temperature threshold (-20 to 120°C)
        #[arg(long, value_parser = clap::value_parser!(i16).range(-20..=120))]
        high_temp: Option<i16>,

        /// ECO/work mode (0=Shutdown, 1=Minimum speed)
        #[arg(long, value_parser = clap::value_parser!(u8).range(0..=1))]
        eco: Option<u8>,

        /// Number of fans (1-4, 0=disable fault detection)
        #[arg(long, value_parser = clap::value_parser!(u8).range(0..=4))]
        fan_qty: Option<u8>,

        /// PWM frequency (500, 1000, 2000, 5000, 10000, 25000 Hz)
        #[arg(long, value_parser = validate_pwm_freq)]
        pwm_freq: Option<u32>,

        /// Manual speed percentage (0-100, only for manual mode)
        #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
        manual_speed: Option<u8>,
    },

    /// Reset the controller
    Reset,
}

/// Validates PWM frequency value
fn validate_pwm_freq(s: &str) -> Result<u32, String> {
    let freq: u32 = s.parse().map_err(|_| format!("Invalid number: {}", s))?;

    match freq {
        500 | 1000 | 2000 | 5000 | 10000 | 25000 => Ok(freq),
        _ => Err(format!(
            "Invalid PWM frequency: {}. Valid values: 500, 1000, 2000, 5000, 10000, 25000",
            freq
        )),
    }
}

impl Cli {
    /// Validates and retrieves the serial port, either from args or environment
    pub fn get_port(&self) -> Result<String, String> {
        self.port
            .clone()
            .ok_or_else(|| "Serial port not specified. Use --port or set JPF4826_PORT".to_string())
    }

    /// Validates and retrieves the Modbus address, either from args or environment
    pub fn get_addr(&self) -> Result<u8, String> {
        self.addr.ok_or_else(|| {
            "Modbus address not specified. Use --addr or set JPF4826_ADDR".to_string()
        })
    }
}
