use anyhow::Result;
use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient) -> Result<serde_json::Value> {
    let result = client.list_vpn_gateways().await?;
    match result.get("value") {
        Some(v) => Ok(v.clone()),
        None => Ok(result),
    }
}
