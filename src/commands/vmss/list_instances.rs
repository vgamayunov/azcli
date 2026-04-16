use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str, expand: Option<&str>) -> Result<serde_json::Value> {
    let instances = client.list_vmss_instances(resource_group, name, expand).await?;
    let flattened: Vec<serde_json::Value> = instances.iter().map(|v| v.to_flattened_value()).collect();
    Ok(serde_json::Value::Array(flattened))
}
