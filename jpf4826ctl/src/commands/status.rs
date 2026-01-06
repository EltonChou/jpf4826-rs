//! Status command implementation.

// Rust guideline compliant 2026-01-06

use crate::output::{convert_to_fahrenheit, format_status_json, format_status_text};
use jpf4826_driver::Jpf4826Client;

/// Executes the status command.
///
/// Reads controller status and outputs in text or JSON format.
///
/// # Arguments
///
/// * `client` - Connected JPF4826 client
/// * `json` - Output JSON format if true, text otherwise
/// * `temp_unit` - Temperature unit (0=Celsius, 1=Fahrenheit)
pub async fn execute(
    client: &mut Jpf4826Client,
    json: bool,
    temp_unit: Option<u8>,
) -> anyhow::Result<()> {
    // Read status from controller
    let mut status = client.status().await?;

    // Convert to Fahrenheit if requested
    if temp_unit == Some(1) {
        status = convert_to_fahrenheit(status);
    }

    // Output in requested format
    if json {
        let output = format_status_json(&status)?;
        println!("{}", output);
    } else {
        let output = format_status_text(&status);
        print!("{}", output);
    }

    Ok(())
}
