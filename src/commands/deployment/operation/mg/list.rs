use anyhow::Result;
use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, management_group_id: &str, deployment_name: &str) -> Result<serde_json::Value> {
    let base = ArmClient::deployment_base_url_mg(management_group_id);
    let resp = client.deployment_operations_list(&base, deployment_name).await?;
    match resp.get("value") {
        Some(v) => Ok(v.clone()),
        None => Ok(resp),
    }
}
