use anyhow::Result;
use tracing::info;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str, instance_ids: &[String]) -> Result<()> {
    info!("Updating instances in VMSS '{name}'...");
    client.vmss_update_instances(resource_group, name, instance_ids).await?;
    eprintln!("VMSS '{name}' update-instances initiated.");
    Ok(())
}
