use anyhow::{Context, Result};

use crate::arm_client::ArmClient;
use super::{apply_primary, nic_name_from_id, resolve_nic_id};

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    vm_name: &str,
    nics: &[String],
    primary_nic: Option<&str>,
) -> Result<serde_json::Value> {
    let vm = client.show_vm(resource_group, vm_name).await?;
    let mut vm_val = vm.to_flattened_value();

    let nic_array = vm_val
        .pointer_mut("/networkProfile/networkInterfaces")
        .and_then(|v| v.as_array_mut())
        .context("VM has no networkProfile.networkInterfaces")?;

    let to_remove: Vec<String> = nics.iter()
        .map(|n| resolve_nic_id(client.subscription_id(), resource_group, n).to_lowercase())
        .collect();
    let to_remove_names: Vec<String> = nics.iter().map(|n| n.to_lowercase()).collect();

    nic_array.retain(|n| {
        let id = n.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let id_lower = id.to_lowercase();
        let name_lower = nic_name_from_id(id).to_lowercase();
        !(to_remove.contains(&id_lower) || to_remove_names.contains(&name_lower))
    });

    apply_primary(nic_array, primary_nic)?;

    let nic_array_clone = nic_array.clone();
    let patch_body = serde_json::json!({
        "properties": {
            "networkProfile": { "networkInterfaces": nic_array_clone }
        }
    });

    client.vm_update(resource_group, vm_name, patch_body).await
}
