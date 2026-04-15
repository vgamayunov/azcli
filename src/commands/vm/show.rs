use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let vm = client.show_vm(resource_group, name).await?;
    Ok(vm.to_flattened_value())
}
