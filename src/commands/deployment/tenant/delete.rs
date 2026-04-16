use anyhow::Result;
use tracing::info;
use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, name: &str) -> Result<()> {
    info!("Deleting deployment '{name}'...");
    let base = ArmClient::deployment_base_url_tenant();
    client.deployment_delete(&base, name).await?;
    eprintln!("Deployment '{name}' deleted.");
    Ok(())
}
