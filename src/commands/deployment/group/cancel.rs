use anyhow::Result;
use tracing::info;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str) -> Result<()> {
    info!("Cancelling deployment '{name}'...");
    client.cancel_deployment(resource_group, name).await?;
    eprintln!("Deployment '{name}' cancel initiated.");
    Ok(())
}
