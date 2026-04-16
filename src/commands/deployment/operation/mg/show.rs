use anyhow::Result;
use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, management_group_id: &str, deployment_name: &str, operation_id: &str) -> Result<serde_json::Value> {
    let base = ArmClient::deployment_base_url_mg(management_group_id);
    client.deployment_operations_show(&base, deployment_name, operation_id).await
}
