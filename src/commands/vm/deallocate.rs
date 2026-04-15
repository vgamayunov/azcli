use anyhow::Result;
use tracing::info;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str, no_wait: bool) -> Result<()> {
    info!("Deallocating VM '{name}'...");
    client.stop_vm(resource_group, name, true).await?;

    if no_wait {
        eprintln!("VM '{name}' deallocate initiated (no-wait).");
    } else {
        eprintln!("VM '{name}' deallocated.");
    }
    Ok(())
}
