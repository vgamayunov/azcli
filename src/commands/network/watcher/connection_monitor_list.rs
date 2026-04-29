use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, watcher_name: &str) -> Result<serde_json::Value> {
    client.list_connection_monitors(resource_group, watcher_name).await
}
