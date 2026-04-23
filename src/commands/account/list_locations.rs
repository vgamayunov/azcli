use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, subscription: Option<&str>) -> Result<serde_json::Value> {
    let sub = subscription.unwrap_or_else(|| client.subscription_id());
    let raw = client.list_locations(sub).await?;
    let items = raw.get("value").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    let out: Vec<serde_json::Value> = items
        .into_iter()
        .map(|loc| {
            serde_json::json!({
                "name": loc.get("name"),
                "displayName": loc.get("displayName"),
                "regionalDisplayName": loc.get("regionalDisplayName"),
                "metadata": loc.get("metadata"),
                "id": loc.get("id"),
            })
        })
        .collect();
    Ok(serde_json::Value::Array(out))
}
