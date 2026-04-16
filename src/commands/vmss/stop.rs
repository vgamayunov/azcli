use anyhow::Result;
use tracing::info;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str, instance_ids: Option<&[String]>) -> Result<()> {
    info!("Stopping VMSS '{name}'...");
    client.vmss_stop(resource_group, name, instance_ids).await?;
    eprintln!("VMSS '{name}' stop initiated.");
    Ok(())
}
