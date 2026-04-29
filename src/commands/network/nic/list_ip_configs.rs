use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, nic_name: &str) -> Result<serde_json::Value> {
    let result = client.list_nic_ip_configs(resource_group, nic_name).await?;
    match result.get("value") {
        Some(v) => Ok(v.clone()),
        None => Ok(result),
    }
}
