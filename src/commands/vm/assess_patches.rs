use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str) -> Result<serde_json::Value> {
    client.vm_assess_patches(resource_group, name).await
}
