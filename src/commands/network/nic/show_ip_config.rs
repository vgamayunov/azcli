use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, nic_name: &str, name: &str) -> Result<serde_json::Value> {
    client.show_nic_ip_config(resource_group, nic_name, name).await
}
