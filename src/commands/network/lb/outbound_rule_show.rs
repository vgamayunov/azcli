use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, lb_name: &str, name: &str) -> Result<serde_json::Value> {
    client.show_load_balancer_outbound_rule(resource_group, lb_name, name).await
}
