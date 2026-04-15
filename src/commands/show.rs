use anyhow::Result;

use crate::api_client::BastionClient;

pub async fn execute(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let client = BastionClient::new().await?;
    let bastion = client.show(resource_group, name).await?;

    Ok(serde_json::to_value(&bastion)?)
}
