use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, location: &str, shared_to_tenant: bool) -> Result<serde_json::Value> {
    let result = client.list_shared_galleries(location, shared_to_tenant).await?;
    match result.get("value") {
        Some(v) => Ok(v.clone()),
        None => Ok(result),
    }
}
