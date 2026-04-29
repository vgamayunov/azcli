use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, watcher_name: &str, name: &str) -> Result<serde_json::Value> {
    client.show_flow_log(resource_group, watcher_name, name).await
}
