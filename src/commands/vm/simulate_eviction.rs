use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str) -> Result<()> {
    client.vm_post_action(resource_group, name, "simulateEviction").await?;
    eprintln!("VM '{name}' eviction simulated.");
    Ok(())
}
