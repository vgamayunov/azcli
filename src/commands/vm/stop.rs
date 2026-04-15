use anyhow::Result;
use tracing::info;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str, no_wait: bool, skip_deallocate: bool) -> Result<()> {
    let action = if skip_deallocate { "Powering off" } else { "Deallocating" };
    info!("{action} VM '{name}'...");
    client.stop_vm(resource_group, name, !skip_deallocate).await?;

    if no_wait {
        eprintln!("VM '{name}' stop initiated (no-wait).");
    } else {
        eprintln!("VM '{name}' stop initiated.");
    }
    Ok(())
}
