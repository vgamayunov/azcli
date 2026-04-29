use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, zone_name: &str, record_type: &str) -> Result<serde_json::Value> {
    client.list_dns_record_sets(resource_group, zone_name, record_type).await
}
