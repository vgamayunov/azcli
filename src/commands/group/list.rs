use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient) -> Result<serde_json::Value> {
    let groups = client.list_resource_groups().await?;
    Ok(serde_json::to_value(&groups)?)
}
