use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: Option<&str>) -> Result<serde_json::Value> {
    let vms = client.list_vms(resource_group).await?;
    let flattened: Vec<serde_json::Value> = vms.iter().map(|vm| vm.to_flattened_value()).collect();
    Ok(serde_json::Value::Array(flattened))
}
