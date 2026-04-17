use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: Option<&str>) -> Result<serde_json::Value> {
    let result = client.list_disks(resource_group).await?;
    match result.get("value") {
        Some(value) => Ok(value.clone()),
        None => Ok(result),
    }
}
