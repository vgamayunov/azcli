use anyhow::Result;
use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: Option<&str>) -> Result<serde_json::Value> {
    if let Some(rg) = resource_group {
        client.list_private_dns_zones_by_rg(rg).await
    } else {
        client.list_private_dns_zones().await
    }
}
