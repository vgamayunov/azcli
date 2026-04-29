use anyhow::Result;
use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, zone_name: &str, name: &str, record_type: &str) -> Result<serde_json::Value> {
    client.show_private_dns_record_set(resource_group, zone_name, name, record_type).await
}
