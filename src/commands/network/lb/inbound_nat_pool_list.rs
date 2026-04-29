use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, lb_name: &str) -> Result<serde_json::Value> {
    client.list_load_balancer_inbound_nat_pools(resource_group, lb_name).await
}
