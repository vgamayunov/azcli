use anyhow::Result;
use tracing::info;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str) -> Result<()> {
    info!("Deleting deployment '{name}'...");
    let base = client.deployment_base_url_group(resource_group);
    client.deployment_delete(&base, name).await?;
    eprintln!("Deployment '{name}' deleted.");
    Ok(())
}
