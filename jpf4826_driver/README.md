# jpf4826_driver

[![Crates.io](https://img.shields.io/crates/v/jpf4826_driver.svg)](https://crates.io/crates/jpf4826_driver)
[![Documentation](https://docs.rs/jpf4826_driver/badge.svg)](https://docs.rs/jpf4826_driver)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../LICENSE)

Rust driver library for JPF4826 4-channel PWM fan controller via Modbus-RTU protocol over RS485.

## Overview

The JPF4826 is an industrial-grade 4-channel PWM DC fan temperature controller with:
- Automatic temperature-based speed control
- Manual speed control mode
- Comprehensive fault detection
- RS485 Modbus-RTU interface ([doc](jpf4826_modbus.md))

This library provides a type-safe, async Rust interface for controlling JPF4826 devices over serial connections.

## Features

- ✅ **Type-safe API** - Enums prevent invalid values at compile time
- ✅ **Async/await** - Built on tokio for efficient async I/O
- ✅ **Cross-platform** - Works on Linux, macOS, and Windows
- ✅ **Comprehensive error handling** - Detailed error messages with context
- ✅ **Well-tested** - 70+ unit and integration tests
- ✅ **Mock support** - Hardware-independent testing with `test-mock` feature

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
jpf4826-driver = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

```rust
use jpf4826_driver::{Jpf4826Client, WorkMode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to controller at /dev/ttyUSB0, Modbus address 1
    let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;

    // Read current status
    let status = client.status().await?;
    println!("Temperature: {}°C", status.temperature_current.value);
    println!("Fan count: {}", status.fan_count);

    // Set temperature thresholds (start at 30°C, full speed at 50°C)
    client.set_temperature_threshold(30, 50).await?;

    // Enable automatic temperature mode
    client.set_auto_speed().await?;

    // Set ECO mode (maintain 20% speed below threshold)
    client.set_eco(WorkMode::MinimumSpeed).await?;

    Ok(())
}
```

## Usage Examples

### Reading Temperature and Fan Status

```rust
use jpf4826_driver::Jpf4826Client;

#[tokio::main]
async fn main() -> jpf4826_driver::Result<()> {
    let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;

    // Read current temperature
    let temp = client.temperature().await?;
    println!("Current temperature: {}°C", temp.value);

    // Read individual fan speed
    let rpm = client.fan_speed(1).await?;
    println!("Fan 1 speed: {} RPM", rpm);

    // Read all fan statuses
    let fans = client.fan_status().await?;
    for fan in fans {
        println!("Fan {}: {:?} @ {} RPM", fan.index, fan.status, fan.rpm);
    }

    Ok(())
}
```

### Manual Speed Control

```rust
use jpf4826_driver::Jpf4826Client;

#[tokio::main]
async fn main() -> jpf4826_driver::Result<()> {
    let mut client = Jpf4826Client::new("COM3", 1).await?;

    // Set fan speed to 75% (automatically enables manual mode)
    client.set_fan_speed(75).await?;

    println!("Fans set to 75% speed");

    // Return to automatic temperature control
    client.set_auto_speed().await?;

    Ok(())
}
```

### Configuration and Settings

```rust
use jpf4826_driver::{Jpf4826Client, PwmFrequency, WorkMode};

#[tokio::main]
async fn main() -> jpf4826_driver::Result<()> {
    let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;

    // Configure 3 fans
    client.set_fan_count(3).await?;

    // Set PWM frequency to 25 kHz
    client.set_pwm_frequency(PwmFrequency::Hz25000).await?;

    // Configure temperature range: start at 25°C, full speed at 40°C
    client.set_temperature_threshold(25, 40).await?;

    // Enable ECO mode (fans maintain 20% below threshold)
    client.set_eco(WorkMode::MinimumSpeed).await?;

    println!("Controller configured successfully");

    Ok(())
}
```

### Complete Status Report

```rust
use jpf4826_driver::Jpf4826Client;

#[tokio::main]
async fn main() -> jpf4826_driver::Result<()> {
    let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;

    // Read complete status
    let status = client.status().await?;

    println!("=== JPF4826 Controller Status ===");
    println!("ECO Mode: {}", status.eco_mode);
    println!("Modbus Address: 0x{:02X}", status.modbus_address);
    println!("PWM Frequency: {} Hz", status.pwm_frequency.to_hz());
    println!("Fan Count: {}", status.fan_count);
    println!("\nTemperature:");
    println!("  Current: {}°C", status.temperature_current.value);
    println!("  Low Threshold: {}°C", status.temperature_low_threshold.value);
    println!("  High Threshold: {}°C", status.temperature_high_threshold.value);
    println!("\nFans:");
    for fan in &status.fans {
        println!("  Fan {}: {:?} - {} RPM", fan.index, fan.status, fan.rpm);
    }

    Ok(())
}
```

### Low-Level Register Access

For advanced users who need direct register access:

```rust
use jpf4826_driver::{Jpf4826Client, registers::RegisterAddress};

#[tokio::main]
async fn main() -> jpf4826_driver::Result<()> {
    let mut client = Jpf4826Client::new("/dev/ttyUSB0", 1).await?;

    // Read raw register value
    let values = client.read(RegisterAddress::CurrentTemperature, 1).await?;
    println!("Raw temperature register: 0x{:04X}", values[0]);

    // Write raw register value
    client.write(RegisterAddress::WorkMode, 0x0001).await?;

    Ok(())
}
```

## Serial Port Configuration

The driver automatically configures the serial port with JPF4826 specifications:

- **Baud rate**: 9600
- **Data bits**: 8
- **Parity**: None
- **Stop bits**: 1
- **Flow control**: None

### Serial Port Paths

- **Linux**: `/dev/ttyUSB0`, `/dev/ttyS0`, etc.
- **macOS**: `/dev/tty.usbserial-XXXXXXXX`
- **Windows**: `COM3`, `COM4`, etc.

## API Documentation

### Core Types

- **`WorkMode`**: `Shutdown` or `MinimumSpeed` (ECO mode)
- **`PwmFrequency`**: 500, 1000, 2000, 5000, 10000, or 25000 Hz
- **`FanStatus`**: `Normal` or `Fault`
- **`TemperatureUnit`**: `Celsius` or `Fahrenheit`

### Main Client Methods

#### Connection
- `new(port: &str, slave_addr: u8) -> Result<Self>` - Create new client

#### Read Operations
- `status() -> Result<ControllerStatus>` - Complete status snapshot
- `temperature() -> Result<Temperature>` - Current temperature
- `fan_speed(index: u8) -> Result<u16>` - Individual fan speed (1-4)
- `fan_count() -> Result<u8>` - Configured fan count
- `fan_status() -> Result<Vec<FanInfo>>` - All fan statuses

#### Write Operations
- `set_auto_speed() -> Result<()>` - Switch to automatic temperature-based speed control
- `set_eco(mode: WorkMode) -> Result<()>` - Set ECO/work mode
- `set_fan_speed(speed_percent: u8) -> Result<()>` - Set manual speed (0-100%, automatically enables manual mode)
- `set_fan_count(count: u8) -> Result<()>` - Set fan count (0-4, 0=disable fault detection)
- `set_temperature_threshold(low: i16, high: i16) -> Result<()>` - Temperature range (-20 to 120°C)
- `set_pwm_frequency(freq: PwmFrequency) -> Result<()>` - PWM frequency
- `set_addr(addr: u8) -> Result<()>` - Change Modbus address (1-254)
- `reset() -> Result<()>` - Reset controller
- `disable_fault_detection() -> Result<()>` - Disable fault detection

#### Low-Level Access
- `read(register: RegisterAddress, count: u16) -> Result<Vec<u16>>` - Read registers
- `write(register: RegisterAddress, value: u16) -> Result<()>` - Write register

## Error Handling

The library uses a structured error type `Jpf4826Error` with detailed context:

```rust
use jpf4826_driver::Jpf4826Client;

#[tokio::main]
async fn main() {
    let mut client = match Jpf4826Client::new("/dev/ttyUSB0", 1).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            if e.is_serial() {
                eprintln!("Serial port error - check connection and permissions");
            }
            return;
        }
    };

    if let Err(e) = client.set_fan_speed(150).await {
        eprintln!("Invalid speed: {}", e);
        // Error: Speed 150 out of range (0-100)
    }
}
```

## Testing

### Running Tests

```bash
# Run all tests (requires test-mock feature for integration tests)
cargo test --features test-mock

# Run only unit tests
cargo test --lib

# Run with output
cargo test --features test-mock -- --nocapture

# Run specific test
cargo test --features test-mock test_read_temperature
```

### Test Coverage

The library includes comprehensive tests:
- **14 tests** - Type conversions and validation
- **13 tests** - Protocol conversions (temperature offset, bitmaps)
- **12 tests** - Read operations
- **26 tests** - Write operations
- **5 tests** - Mock controller functionality
- **36 tests** - Documentation examples

### Using Mock Client for Testing

The library provides a mock client for hardware-independent testing:

```toml
[dev-dependencies]
jpf4826-driver = { version = "0.1", features = ["test-mock"] }
```

```rust
#[cfg(test)]
mod tests {
    use jpf4826_driver::Jpf4826Client;
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_my_function() {
        // Create mock controller with initial state
        let registers = Arc::new(Mutex::new(HashMap::new()));
        registers.lock().unwrap().insert(0x0000, 71); // 31°C (with +40 offset)

        let mut client = Jpf4826Client::new_mock(registers.clone()).await;

        let temp = client.temperature().await.unwrap();
        assert_eq!(temp.value, 31);
    }
}
```

## Development

### Building

```bash
# Build library
cargo build

# Build with optimizations
cargo build --release

# Build documentation
cargo doc --open
```

### Code Style

This project follows Microsoft Rust Guidelines and uses:
- `rustfmt` for code formatting
- `clippy` for linting

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings
```

## Protocol Reference

For detailed protocol information, see [`jpf4826_modbus.md`](jpf4826_modbus.md).

### Key Protocol Details

- **Temperature Offset**: All temperatures stored with +40 offset (e.g., 31°C = 71 in register)
- **Temperature Range**: -20°C to 120°C
- **Fan Indices**: 1-4 (not 0-3)
- **Modbus Address Range**: 1-254
- **Manual Speed Range**: 0-100%
- **Fault Detection**: Disabled when fan count = 0

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.


## Resources

- [JPF4826 Product Page](https://www.jianpanfan.com/)
- [Modbus Protocol Specification](https://modbus.org/)
- [tokio-modbus Documentation](https://docs.rs/tokio-modbus/)
- [tokio-serial Documentation](https://docs.rs/tokio-serial/)

## Support

For issues and feature requests, please use the [GitHub issue tracker](https://github.com/EltonChou/jpf4826-rs/issues).
