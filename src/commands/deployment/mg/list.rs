use anyhow::Result;
use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, management_group_id: &str) -> Result<serde_json::Value> {
    let base = ArmClient::deployment_base_url_mg(management_group_id);
    let resp = client.deployment_list(&base).await?;
    match resp.get("value") {
        Some(v) => Ok(v.clone()),
        None => Ok(resp),
    }
}
