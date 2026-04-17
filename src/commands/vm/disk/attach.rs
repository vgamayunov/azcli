use anyhow::{Context, Result};

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    vm_name: &str,
    disk: &str,
    new: bool,
    size_gb: Option<i64>,
    sku: Option<&str>,
    lun: Option<i64>,
    caching: Option<&str>,
    enable_write_accelerator: bool,
) -> Result<serde_json::Value> {
    let disk_id = if disk.starts_with('/') {
        disk.to_string()
    } else {
        format!(
            "/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/disks/{}",
            client.subscription_id(), resource_group, disk
        )
    };
    let disk_name = disk_id.rsplit('/').next().unwrap_or(disk).to_string();

    if new {
        let vm_show = client.show_vm(resource_group, vm_name).await?;
        let vm_val = vm_show.to_flattened_value();
        let location = vm_val.get("location").and_then(|v| v.as_str())
            .context("VM has no location")?.to_string();

        let mut body = serde_json::json!({
            "location": location,
            "properties": {
                "creationData": { "createOption": "Empty" },
                "diskSizeGB": size_gb.unwrap_or(1023),
            },
        });
        if let Some(s) = sku {
            body["sku"] = serde_json::json!({ "name": s });
        }
        client.create_disk(resource_group, &disk_name, body).await?;
    }

    let vm = client.show_vm(resource_group, vm_name).await?;
    let mut vm_val = vm.to_flattened_value();

    let data_disks = vm_val.pointer_mut("/storageProfile/dataDisks")
        .and_then(|v| v.as_array_mut())
        .context("VM storageProfile.dataDisks not found")?;

    let next_lun = lun.unwrap_or_else(|| {
        let used: Vec<i64> = data_disks.iter()
            .filter_map(|d| d.get("lun").and_then(|v| v.as_i64())).collect();
        (0..).find(|i| !used.contains(i)).unwrap_or(0)
    });

    let mut entry = serde_json::json!({
        "lun": next_lun,
        "name": disk_name,
        "createOption": "Attach",
        "managedDisk": { "id": disk_id },
    });
    if let Some(c) = caching {
        entry["caching"] = serde_json::json!(c);
    }
    if enable_write_accelerator {
        entry["writeAcceleratorEnabled"] = serde_json::json!(true);
    }
    data_disks.push(entry);

    let data_disks_clone = data_disks.clone();
    let patch_body = serde_json::json!({
        "properties": {
            "storageProfile": { "dataDisks": data_disks_clone }
        }
    });

    client.vm_update(resource_group, vm_name, patch_body).await
}
