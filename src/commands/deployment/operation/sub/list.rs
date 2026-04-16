use anyhow::Result;
use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, deployment_name: &str) -> Result<serde_json::Value> {
    let base = client.deployment_base_url_sub();
    let resp = client.deployment_operations_list(&base, deployment_name).await?;
    match resp.get("value") {
        Some(v) => Ok(v.clone()),
        None => Ok(resp),
    }
}
