use anyhow::Result;
use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, gateway_name: &str) -> Result<serde_json::Value> {
    client.list_application_gateway_ssl_policies(resource_group, gateway_name).await
}
