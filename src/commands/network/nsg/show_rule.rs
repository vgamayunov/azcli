use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, nsg_name: &str, name: &str) -> Result<serde_json::Value> {
    client.show_nsg_rule(resource_group, nsg_name, name).await
}
