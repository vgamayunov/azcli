use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, vnet_name: &str, name: &str) -> Result<serde_json::Value> {
    client.show_subnet(resource_group, vnet_name, name).await
}
