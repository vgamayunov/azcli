use anyhow::{Context, Result};
use tracing::{debug, info};

use crate::api_client::BastionClient;

pub async fn execute(
    resource_group: &str,
    name: &str,
    created: bool,
    updated: bool,
    deleted: bool,
    exists: bool,
    interval: u64,
    timeout: u64,
) -> Result<()> {
    let client = BastionClient::new().await?;
    execute_with_client(&client, resource_group, name, created, updated, deleted, exists, interval, timeout).await
}

#[allow(clippy::too_many_arguments)]
pub async fn execute_with_client(
    client: &BastionClient,
    resource_group: &str,
    name: &str,
    created: bool,
    updated: bool,
    deleted: bool,
    exists: bool,
    interval: u64,
    timeout: u64,
) -> Result<()> {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(timeout);
    let poll_interval = std::time::Duration::from_secs(interval);

    loop {
        if tokio::time::Instant::now() >= deadline {
            anyhow::bail!("Timed out waiting for bastion host '{name}' after {timeout}s");
        }

        let result = client.show(resource_group, name).await;

        match &result {
            Ok(bastion) => {
                let state = bastion
                    .properties
                    .as_ref()
                    .and_then(|p| p.provisioning_state.as_deref())
                    .unwrap_or("Unknown");

                debug!("Bastion '{name}' provisioning state: {state}");

                if deleted {
                    debug!("Waiting for deletion, but resource still exists");
                } else if exists {
                    info!("Bastion host '{name}' exists (state: {state})");
                    return Ok(());
                } else if created && state == "Succeeded" {
                    info!("Bastion host '{name}' created successfully");
                    return Ok(());
                } else if updated && state == "Succeeded" {
                    info!("Bastion host '{name}' updated successfully");
                    return Ok(());
                }
            }
            Err(e) => {
                if deleted {
                    info!("Bastion host '{name}' deleted");
                    return Ok(());
                }
                let err_str = format!("{e:#}");
                if err_str.contains("404") || err_str.contains("ResourceNotFound") {
                    if deleted {
                        info!("Bastion host '{name}' deleted");
                        return Ok(());
                    }
                    debug!("Bastion '{name}' not found, retrying...");
                } else {
                    return Err(result.unwrap_err()).context("Failed to poll bastion host");
                }
            }
        }

        tokio::time::sleep(poll_interval).await;
    }
}
