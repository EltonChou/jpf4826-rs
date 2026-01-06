//! Reset command implementation.

// Rust guideline compliant 2026-01-06

use jpf4826_driver::Jpf4826Client;

/// Executes the reset command.
///
/// Sends reset command to the controller.
///
/// # Arguments
///
/// * `client` - Connected JPF4826 client
pub async fn execute(client: &mut Jpf4826Client) -> anyhow::Result<()> {
    client.reset().await?;
    println!("Controller reset command sent successfully.");
    Ok(())
}
