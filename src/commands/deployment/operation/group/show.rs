use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, deployment_name: &str, operation_id: &str) -> Result<serde_json::Value> {
    let base = client.deployment_base_url_group(resource_group);
    client.deployment_operations_show(&base, deployment_name, operation_id).await
}
