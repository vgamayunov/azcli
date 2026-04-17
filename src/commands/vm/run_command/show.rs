use anyhow::{Result, bail};

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    resource_group: Option<&str>,
    vm_name: Option<&str>,
    name: Option<&str>,
    location: Option<&str>,
    command_id: Option<&str>,
    instance_view: bool,
) -> Result<serde_json::Value> {
    match (resource_group, vm_name, name, location, command_id) {
        (Some(rg), Some(vm), Some(n), None, None) => {
            client.show_vm_run_command(rg, vm, n, instance_view).await
        }
        (None, None, None, Some(loc), Some(cmd_id)) => {
            client.show_builtin_run_command(loc, cmd_id).await
        }
        _ => bail!("provide either (--vm-name, --resource-group, --name) for a VM run command or (--location, --command-id) for a built-in"),
    }
}
