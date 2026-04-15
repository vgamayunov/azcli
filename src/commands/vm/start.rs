use anyhow::Result;
use tracing::info;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str) -> Result<()> {
    info!("Starting VM '{name}'...");
    client.start_vm(resource_group, name).await?;
    eprintln!("VM '{name}' start initiated.");
    Ok(())
}
