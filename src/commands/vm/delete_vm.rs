use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str, force: bool, no_wait: bool) -> Result<()> {
    client.vm_delete(resource_group, name, force).await?;
    if no_wait {
        eprintln!("VM '{name}' delete initiated (no-wait).");
    } else {
        eprintln!("VM '{name}' deleted.");
    }
    Ok(())
}
