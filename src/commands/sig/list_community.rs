use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, location: Option<&str>) -> Result<serde_json::Value> {
    client.list_community_galleries(location, 30).await
}
