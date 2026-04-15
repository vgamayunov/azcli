use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, name: &str) -> Result<serde_json::Value> {
    let group = client.show_resource_group(name).await?;
    Ok(serde_json::to_value(&group)?)
}
