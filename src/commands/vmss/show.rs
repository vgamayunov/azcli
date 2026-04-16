use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let vmss = client.show_vmss(resource_group, name).await?;
    Ok(vmss.to_flattened_value())
}
