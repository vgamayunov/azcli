use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: Option<&str>) -> Result<serde_json::Value> {
    let items = client.list_vmss(resource_group).await?;
    let flattened: Vec<serde_json::Value> = items.iter().map(|v| v.to_flattened_value()).collect();
    Ok(serde_json::Value::Array(flattened))
}
