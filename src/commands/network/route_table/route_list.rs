use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, table_name: &str) -> Result<serde_json::Value> {
    client.list_routes(resource_group, table_name).await
}
