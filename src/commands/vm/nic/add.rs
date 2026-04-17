use anyhow::{Context, Result};

use crate::arm_client::ArmClient;
use super::{apply_primary, resolve_nic_id};

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

    for nic in nics {
        let id = resolve_nic_id(client.subscription_id(), resource_group, nic);
        let id_lower = id.to_lowercase();
        let already = nic_array.iter().any(|n| {
            n.get("id").and_then(|v| v.as_str())
                .map(|existing| existing.to_lowercase() == id_lower)
                .unwrap_or(false)
        });
        if !already {
            nic_array.push(serde_json::json!({ "id": id, "primary": false }));
        }
    }

    apply_primary(nic_array, primary_nic)?;

    let nic_array_clone = nic_array.clone();
    let patch_body = serde_json::json!({
        "properties": {
            "networkProfile": { "networkInterfaces": nic_array_clone }
        }
    });

    client.vm_update(resource_group, vm_name, patch_body).await
}
