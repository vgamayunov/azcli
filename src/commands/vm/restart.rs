use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str, no_wait: bool) -> Result<()> {
    client.vm_post_action(resource_group, name, "restart").await?;
    if no_wait {
        eprintln!("VM '{name}' restart initiated (no-wait).");
    } else {
        eprintln!("VM '{name}' restarted.");
    }
    Ok(())
}
