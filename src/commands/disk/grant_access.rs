use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    name: &str,
    access: &str,
    duration_in_seconds: i64,
) -> Result<serde_json::Value> {
    client.disk_grant_access(resource_group, name, access, duration_in_seconds).await
}
