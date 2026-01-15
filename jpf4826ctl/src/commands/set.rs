//! Set command implementation.

// Rust guideline compliant 2026-01-06

use jpf4826_driver::{Jpf4826Client, OperatingMode, PwmFrequency, WorkMode};

/// Arguments for the set command.
#[derive(Debug)]
pub struct SetArgs {
    pub auto_speed: bool,
    pub modbus_addr: Option<u8>,
    pub low_temp: Option<i16>,
    pub high_temp: Option<i16>,
    pub eco: Option<u8>,
    pub fan_qty: Option<u8>,
    pub pwm_freq: Option<u32>,
    pub manual_speed: Option<u8>,
}

impl SetArgs {
    /// Checks if all options are None (no arguments provided).
    pub fn is_empty(&self) -> bool {
        !self.auto_speed
            && self.modbus_addr.is_none()
            && self.low_temp.is_none()
            && self.high_temp.is_none()
            && self.eco.is_none()
            && self.fan_qty.is_none()
            && self.pwm_freq.is_none()
            && self.manual_speed.is_none()
    }
}

/// Executes the set command.
///
/// Applies one or more configuration changes to the controller.
///
/// # Arguments
///
/// * `client` - Connected JPF4826 client
/// * `args` - Set command arguments
pub async fn execute(client: &mut Jpf4826Client, args: SetArgs) -> anyhow::Result<()> {
    let mut operations_count = 0;

    // Set automatic temperature mode
    if args.auto_speed {
        client.set_mode(OperatingMode::Temperature).await?;
        operations_count += 1;
        println!("✓ Operating mode set to Temperature (automatic)");
    }

    // Set Modbus address
    if let Some(addr) = args.modbus_addr {
        client.set_addr(addr).await?;
        operations_count += 1;
        println!("✓ Modbus address set to {}", addr);
    }

    // Set temperature thresholds (can be set individually or together)
    match (args.low_temp, args.high_temp) {
        (Some(low), Some(high)) => {
            // Set both thresholds at once
            client.set_temperature_threshold(low, high).await?;
            operations_count += 1;
            println!(
                "✓ Temperature thresholds set: {}°C (low) to {}°C (high)",
                low, high
            );
        }
        (Some(low), None) => {
            // Set only low threshold
            client.set_start_temperature(low).await?;
            operations_count += 1;
            println!("✓ Start temperature set to {}°C", low);
        }
        (None, Some(high)) => {
            // Set only high threshold
            client.set_full_speed_temperature(high).await?;
            operations_count += 1;
            println!("✓ Full speed temperature set to {}°C", high);
        }
        (None, None) => {}
    }

    // Set ECO mode
    if let Some(eco) = args.eco {
        let work_mode = match eco {
            0 => WorkMode::MinimumSpeed,
            1 => WorkMode::Shutdown,
            _ => unreachable!("clap should validate this"),
        };
        client.set_eco(work_mode).await?;
        operations_count += 1;
        println!("✓ ECO mode set to {:?}", work_mode);
    }

    // Set fan quantity
    if let Some(qty) = args.fan_qty {
        client.set_fan_count(qty).await?;
        operations_count += 1;
        if qty == 0 {
            println!("✓ Fault detection disabled");
        } else {
            println!("✓ Fan quantity set to {}", qty);
        }
    }

    // Set PWM frequency
    if let Some(freq_hz) = args.pwm_freq {
        let freq = PwmFrequency::from_hz(freq_hz)
            .ok_or_else(|| anyhow::anyhow!("Invalid PWM frequency: {}", freq_hz))?;
        client.set_pwm_frequency(freq).await?;
        operations_count += 1;
        println!("✓ PWM frequency set to {} Hz", freq_hz);
    }

    // Set manual speed (automatically switches to manual mode)
    if let Some(speed) = args.manual_speed {
        // First switch to manual mode
        client.set_mode(OperatingMode::Manual).await?;
        operations_count += 1;
        println!("✓ Operating mode set to Manual");

        // Then set the speed
        client.set_fan_speed(speed).await?;
        operations_count += 1;
        println!("✓ Manual speed set to {}%", speed);
    }

    if operations_count > 0 {
        println!(
            "\n{} operation(s) completed successfully.",
            operations_count
        );
    }

    Ok(())
}
