use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    name: &str,
    vhd_name_prefix: &str,
    storage_container: &str,
    overwrite: bool,
) -> Result<serde_json::Value> {
    let body = serde_json::json!({
        "vhdPrefix": vhd_name_prefix,
        "destinationContainerName": storage_container,
        "overwriteVhds": overwrite,
    });
    client.vm_post_action_with_body(resource_group, name, "capture", body).await
}
