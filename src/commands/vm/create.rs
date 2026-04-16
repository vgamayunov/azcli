use anyhow::Result;

use crate::arm_client::ArmClient;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    name: &str,
    image: &str,
    location: &str,
    size: &str,
    admin_username: Option<&str>,
    admin_password: Option<&str>,
    ssh_key_value: Option<&str>,
    authentication_type: Option<&str>,
    subnet_id: Option<&str>,
    os_disk_size_gb: Option<i64>,
    data_disk_sizes_gb: &[i64],
    tags: Option<&[String]>,
) -> Result<serde_json::Value> {
    let admin_user = admin_username.unwrap_or("azureuser");
    let auth_type = authentication_type.unwrap_or(if ssh_key_value.is_some() { "ssh" } else { "password" });

    let mut os_profile = serde_json::json!({
        "computerName": name,
        "adminUsername": admin_user,
    });

    if auth_type == "password" {
        if let Some(pwd) = admin_password {
            os_profile["adminPassword"] = serde_json::Value::String(pwd.to_string());
        }
    } else if auth_type == "ssh" {
        if let Some(key) = ssh_key_value {
            let key_data = if key.starts_with('/') || key.starts_with('~') {
                let expanded = key.replace("~", &std::env::var("HOME").unwrap_or_default());
                std::fs::read_to_string(&expanded).unwrap_or_else(|_| key.to_string())
            } else {
                key.to_string()
            };
            os_profile["linuxConfiguration"] = serde_json::json!({
                "disablePasswordAuthentication": true,
                "ssh": {
                    "publicKeys": [{
                        "path": format!("/home/{admin_user}/.ssh/authorized_keys"),
                        "keyData": key_data.trim(),
                    }]
                }
            });
        }
    }

    let mut os_disk = serde_json::json!({
        "createOption": "FromImage",
        "managedDisk": { "storageAccountType": "Premium_LRS" },
    });
    if let Some(size_gb) = os_disk_size_gb {
        os_disk["diskSizeGB"] = serde_json::json!(size_gb);
    }

    let mut data_disks_json = Vec::new();
    for (i, &size_gb) in data_disk_sizes_gb.iter().enumerate() {
        data_disks_json.push(serde_json::json!({
            "lun": i,
            "diskSizeGB": size_gb,
            "createOption": "Empty",
            "managedDisk": { "storageAccountType": "Premium_LRS" },
        }));
    }

    let mut network_interfaces = Vec::new();
    if let Some(sid) = subnet_id {
        network_interfaces.push(serde_json::json!({
            "id": sid,
        }));
    }

    let (publisher, offer, sku, version) = parse_image(image);

    let mut body = serde_json::json!({
        "location": location,
        "properties": {
            "hardwareProfile": { "vmSize": size },
            "storageProfile": {
                "imageReference": {
                    "publisher": publisher,
                    "offer": offer,
                    "sku": sku,
                    "version": version,
                },
                "osDisk": os_disk,
                "dataDisks": data_disks_json,
            },
            "osProfile": os_profile,
            "networkProfile": {
                "networkInterfaces": network_interfaces
            }
        }
    });

    if let Some(tag_list) = tags {
        let mut tag_map = serde_json::Map::new();
        for tag in tag_list {
            if let Some((k, v)) = tag.split_once('=') {
                tag_map.insert(k.to_string(), serde_json::Value::String(v.to_string()));
            }
        }
        body["tags"] = serde_json::Value::Object(tag_map);
    }

    client.vm_create(resource_group, name, body).await
}

fn parse_image(image: &str) -> (&str, &str, &str, &str) {
    let parts: Vec<&str> = image.split(':').collect();
    if parts.len() == 4 {
        (parts[0], parts[1], parts[2], parts[3])
    } else {
        (image, "", "", "latest")
    }
}
