use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str) -> Result<()> {
    client.vm_post_action(resource_group, name, "performMaintenance").await?;
    eprintln!("VM '{name}' maintenance initiated.");
    Ok(())
}
