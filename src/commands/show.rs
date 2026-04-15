use anyhow::Result;

use crate::api_client::BastionClient;

pub async fn execute(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let client = BastionClient::new().await?;
    execute_with_client(&client, resource_group, name).await
}

pub async fn execute_with_client(
    client: &BastionClient,
    resource_group: &str,
    name: &str,
) -> Result<serde_json::Value> {
    let bastion = client.show(resource_group, name).await?;
    Ok(serde_json::to_value(&bastion)?)
}
