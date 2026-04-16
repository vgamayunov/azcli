use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str, size: &str) -> Result<serde_json::Value> {
    let body = serde_json::json!({
        "properties": {
            "hardwareProfile": {
                "vmSize": size
            }
        }
    });
    client.vm_update(resource_group, name, body).await
}
