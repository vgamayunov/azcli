use anyhow::Result;
use tracing::info;
use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, name: &str) -> Result<()> {
    info!("Deleting deployment '{name}'...");
    let base = client.deployment_base_url_sub();
    client.deployment_delete(&base, name).await?;
    eprintln!("Deployment '{name}' deleted.");
    Ok(())
}
