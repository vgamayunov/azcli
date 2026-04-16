use anyhow::Result;
use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, name: &str) -> Result<serde_json::Value> {
    let base = client.deployment_base_url_sub();
    client.deployment_show(&base, name).await
}
