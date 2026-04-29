use anyhow::Result;
use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, gateway_name: &str) -> Result<serde_json::Value> {
    client.show_application_gateway_settings(resource_group, gateway_name).await
}
