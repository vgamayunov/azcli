use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let base = client.deployment_base_url_group(resource_group);
    client.deployment_show(&base, name).await
}
