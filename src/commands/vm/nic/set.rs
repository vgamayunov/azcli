use anyhow::Result;

use crate::arm_client::ArmClient;
use super::{apply_primary, resolve_nic_id};

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    vm_name: &str,
    nics: &[String],
    primary_nic: Option<&str>,
) -> Result<serde_json::Value> {
    let mut nic_array: Vec<serde_json::Value> = nics.iter()
        .map(|n| {
            let id = resolve_nic_id(client.subscription_id(), resource_group, n);
            serde_json::json!({ "id": id, "primary": false })
        })
        .collect();

    apply_primary(&mut nic_array, primary_nic)?;

    let patch_body = serde_json::json!({
        "properties": {
            "networkProfile": { "networkInterfaces": nic_array }
        }
    });

    client.vm_update(resource_group, vm_name, patch_body).await
}
