//! JPF4826 fan controller CLI tool.
//!
//! Command-line utility for controlling JPF4826 4-channel PWM fan controllers
//! via Modbus-RTU over serial connection.

// Rust guideline compliant 2026-01-06

use clap::Parser;

mod cli;
mod commands;
mod output;

use cli::{Cli, Commands};
use jpf4826_driver::Jpf4826Client;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Main application logic.
async fn run() -> anyhow::Result<()> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Initialize logger based on verbose flag
    let log_level = if cli.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Warn
    };
    env_logger::Builder::new()
        .filter_level(log_level)
        .format_timestamp_micros()
        .init();

    // If no subcommand provided, print help and exit
    if cli.command.is_none() {
        Cli::parse_from(["jpf4826ctl", "--help"]);
        unreachable!();
    }

    // Validate required global options
    let port = cli.get_port().map_err(|e| anyhow::anyhow!(e))?;
    let addr = cli.get_addr().map_err(|e| anyhow::anyhow!(e))?;

    // Extract command (safe because we checked is_none above)
    let command = cli.command.expect("command must be present");

    // If set command with no options, show help
    if let Commands::Set {
        mode,
        modbus_addr,
        low_temp,
        high_temp,
        eco,
        fan_qty,
        pwm_freq,
        manual_speed,
    } = &command
    {
        let args = commands::set::SetArgs {
            mode: *mode,
            modbus_addr: *modbus_addr,
            low_temp: *low_temp,
            high_temp: *high_temp,
            eco: *eco,
            fan_qty: *fan_qty,
            pwm_freq: *pwm_freq,
            manual_speed: *manual_speed,
        };
        if args.is_empty() {
            Cli::parse_from(["jpf4826ctl", "set", "--help"]);
            unreachable!();
        }
    }

    log::debug!("Connecting to port: {}, address: {}", port, addr);

    // Create client connection
    let mut client = Jpf4826Client::new(&port, addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to controller: {}", e))?;

    log::debug!("Successfully connected to controller");

    // Execute command
    log::debug!("Executing command: {:?}", command);
    match command {
        Commands::Status { json, temp_unit } => {
            commands::status::execute(&mut client, json, temp_unit).await?;
        }
        Commands::Set {
            mode,
            modbus_addr,
            low_temp,
            high_temp,
            eco,
            fan_qty,
            pwm_freq,
            manual_speed,
        } => {
            let args = commands::set::SetArgs {
                mode,
                modbus_addr,
                low_temp,
                high_temp,
                eco,
                fan_qty,
                pwm_freq,
                manual_speed,
            };
            commands::set::execute(&mut client, args).await?;
        }
        Commands::Reset => {
            commands::reset::execute(&mut client).await?;
        }
    }

    Ok(())
}
