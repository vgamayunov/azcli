use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let result = client.vm_list_resize_options(resource_group, name).await?;
    match result.get("value") {
        Some(v) => Ok(v.clone()),
        None => Ok(result),
    }
}
