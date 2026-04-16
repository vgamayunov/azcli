use anyhow::Result;
use tracing::info;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str, new_capacity: i64) -> Result<()> {
    info!("Scaling VMSS '{name}' to {new_capacity} instances...");
    client.vmss_scale(resource_group, name, new_capacity).await?;
    eprintln!("VMSS '{name}' scale to {new_capacity} initiated.");
    Ok(())
}
