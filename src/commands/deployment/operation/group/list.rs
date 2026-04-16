use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, deployment_name: &str) -> Result<serde_json::Value> {
    let resp = client.list_deployment_operations(resource_group, deployment_name).await?;
    match resp.get("value") {
        Some(v) => Ok(v.clone()),
        None => Ok(resp),
    }
}
