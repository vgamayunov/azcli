use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    name: &str,
    location: &str,
    size_gb: Option<i64>,
    sku: Option<&str>,
    source: Option<&str>,
    zone: Option<&str>,
    hyper_v_generation: Option<&str>,
    os_type: Option<&str>,
) -> Result<serde_json::Value> {
    let mut properties = serde_json::json!({});

    if let Some(size) = size_gb {
        properties["diskSizeGB"] = serde_json::json!(size);
    }
    if let Some(hvg) = hyper_v_generation {
        properties["hyperVGeneration"] = serde_json::json!(hvg);
    }
    if let Some(os) = os_type {
        properties["osType"] = serde_json::json!(os);
    }

    let creation_data = if let Some(src) = source {
        if src.starts_with('/') {
            serde_json::json!({ "createOption": "Copy", "sourceResourceId": src })
        } else if src.starts_with("http") {
            serde_json::json!({ "createOption": "Import", "sourceUri": src })
        } else {
            serde_json::json!({ "createOption": "Copy", "sourceResourceId": src })
        }
    } else {
        serde_json::json!({ "createOption": "Empty" })
    };
    properties["creationData"] = creation_data;

    let mut body = serde_json::json!({
        "location": location,
        "properties": properties,
    });

    if let Some(s) = sku {
        body["sku"] = serde_json::json!({ "name": s });
    }
    if let Some(z) = zone {
        body["zones"] = serde_json::json!([z]);
    }

    client.create_disk(resource_group, name, body).await
}
