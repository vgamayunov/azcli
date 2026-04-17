use anyhow::{Result, bail};

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    resource_group: Option<&str>,
    vm_name: Option<&str>,
    location: Option<&str>,
    expand_instance_view: bool,
) -> Result<serde_json::Value> {
    match (resource_group, vm_name, location) {
        (Some(rg), Some(vm), None) => client.list_vm_run_commands(rg, vm, expand_instance_view).await,
        (None, None, Some(loc)) => client.list_builtin_run_commands(loc).await,
        _ => bail!("provide either --vm-name with --resource-group (per-VM listing) or --location (built-in listing)"),
    }
}
