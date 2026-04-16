use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, location: &str) -> Result<serde_json::Value> {
    let result = client.vm_list_sizes(location).await?;
    match result.get("value") {
        Some(v) => Ok(v.clone()),
        None => Ok(result),
    }
}
