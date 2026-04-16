use anyhow::Result;
use tracing::info;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str, instance_ids: Option<&[String]>) -> Result<()> {
    info!("Starting VMSS '{name}'...");
    client.vmss_start(resource_group, name, instance_ids).await?;
    eprintln!("VMSS '{name}' start initiated.");
    Ok(())
}
