use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    vm_name: &str,
    disk_name: &str,
    force_detach: bool,
) -> Result<serde_json::Value> {
    let vm = client.show_vm(resource_group, vm_name).await?;
    let mut vm_val = vm.to_flattened_value();

    let data_disks = vm_val.pointer_mut("/storageProfile/dataDisks")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| anyhow::anyhow!("VM has no data disks"))?;

    let disk_lower = disk_name.to_lowercase();
    if force_detach {
        for d in data_disks.iter_mut() {
            let matches = d.get("name").and_then(|v| v.as_str())
                .map(|n| n.eq_ignore_ascii_case(disk_name)).unwrap_or(false)
                || d.pointer("/managedDisk/id").and_then(|v| v.as_str())
                    .map(|id| id.to_lowercase().ends_with(&format!("/{disk_lower}"))).unwrap_or(false);
            if matches {
                d["toBeDetached"] = serde_json::json!(true);
                d["detachOption"] = serde_json::json!("ForceDetach");
            }
        }
    } else {
        data_disks.retain(|d| {
            let name_match = d.get("name").and_then(|v| v.as_str())
                .map(|n| n.eq_ignore_ascii_case(disk_name)).unwrap_or(false);
            let id_match = d.pointer("/managedDisk/id").and_then(|v| v.as_str())
                .map(|id| id.to_lowercase().ends_with(&format!("/{disk_lower}"))).unwrap_or(false);
            !(name_match || id_match)
        });
    }

    let data_disks_clone = data_disks.clone();
    let patch_body = serde_json::json!({
        "properties": {
            "storageProfile": { "dataDisks": data_disks_clone }
        }
    });

    client.vm_update(resource_group, vm_name, patch_body).await
}
