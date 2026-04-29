use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str) -> Result<serde_json::Value> {
    client.list_load_balancer_inbound_nat_rule_port_mappings(resource_group, name).await
}
