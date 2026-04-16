use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, deployment_name: &str, operation_id: &str) -> Result<serde_json::Value> {
    client.show_deployment_operation(resource_group, deployment_name, operation_id).await
}
