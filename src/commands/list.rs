use anyhow::Result;

use crate::api_client::BastionClient;

pub async fn execute(resource_group: Option<&str>) -> Result<serde_json::Value> {
    let client = BastionClient::new().await?;
    let bastions = client.list(resource_group).await?;

    Ok(serde_json::to_value(&bastions)?)
}
