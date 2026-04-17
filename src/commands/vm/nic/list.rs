use anyhow::{Context, Result};

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    vm_name: &str,
) -> Result<serde_json::Value> {
    let vm = client.show_vm(resource_group, vm_name).await?;
    let vm_val = vm.to_flattened_value();

    let nics = vm_val
        .pointer("/networkProfile/networkInterfaces")
        .cloned()
        .context("VM has no networkProfile.networkInterfaces")?;

    Ok(nics)
}
