use anyhow::{Result, anyhow};

use crate::arm_client::ArmClient;
use super::{nic_name_from_id, resolve_nic_id};

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    vm_name: &str,
    nic: &str,
) -> Result<serde_json::Value> {
    let vm = client.show_vm(resource_group, vm_name).await?;
    let vm_val = vm.to_flattened_value();

    let nics_array = vm_val
        .pointer("/networkProfile/networkInterfaces")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("VM has no networkProfile.networkInterfaces"))?;

    let resolved_id = resolve_nic_id(client.subscription_id(), resource_group, nic);
    let resolved_id_lower = resolved_id.to_lowercase();
    let nic_lower = nic.to_lowercase();

    let attached = nics_array.iter().any(|n| {
        let id = n.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let id_lower = id.to_lowercase();
        id_lower == resolved_id_lower
            || nic_name_from_id(id).to_lowercase() == nic_lower
    });

    if !attached {
        return Err(anyhow!(
            "NIC '{nic}' is not attached to VM '{vm_name}' in resource group '{resource_group}'"
        ));
    }

    client.get_network_interface(&resolved_id).await
}
