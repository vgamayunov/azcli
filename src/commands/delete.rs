use anyhow::Result;
use tracing::info;

use crate::api_client::BastionClient;

pub async fn execute(resource_group: &str, name: &str) -> Result<()> {
    let client = BastionClient::new().await?;
    execute_with_client(&client, resource_group, name).await
}

pub async fn execute_with_client(
    client: &BastionClient,
    resource_group: &str,
    name: &str,
) -> Result<()> {
    info!("Deleting bastion host '{name}'...");
    client.delete(resource_group, name).await?;

    println!("Bastion host '{name}' deleted.");
    Ok(())
}
