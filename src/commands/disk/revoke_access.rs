use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str) -> Result<()> {
    client.disk_revoke_access(resource_group, name).await
}
