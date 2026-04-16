use anyhow::Result;
use tracing::info;
use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, management_group_id: &str, name: &str) -> Result<()> {
    info!("Cancelling deployment '{name}'...");
    let base = ArmClient::deployment_base_url_mg(management_group_id);
    client.deployment_cancel(&base, name).await?;
    eprintln!("Deployment '{name}' cancel initiated.");
    Ok(())
}
