use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, nsg_name: &str) -> Result<serde_json::Value> {
    let result = client.list_nsg_rules(resource_group, nsg_name).await?;
    match result.get("value") {
        Some(v) => Ok(v.clone()),
        None => Ok(result),
    }
}
