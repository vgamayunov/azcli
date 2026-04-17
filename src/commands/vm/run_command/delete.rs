use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    vm_name: &str,
    name: &str,
) -> Result<serde_json::Value> {
    client.delete_vm_run_command(resource_group, vm_name, name).await?;
    Ok(serde_json::json!({ "status": "deleted", "name": name }))
}
