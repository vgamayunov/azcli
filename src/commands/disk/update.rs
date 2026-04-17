use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    name: &str,
    size_gb: Option<i64>,
    sku: Option<&str>,
) -> Result<serde_json::Value> {
    let mut body = serde_json::json!({ "properties": {} });
    if let Some(size) = size_gb {
        body["properties"]["diskSizeGB"] = serde_json::json!(size);
    }
    if let Some(s) = sku {
        body["sku"] = serde_json::json!({ "name": s });
    }
    client.update_disk(resource_group, name, body).await
}
