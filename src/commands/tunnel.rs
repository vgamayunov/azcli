use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::signal;
use tracing::{info, warn};

use crate::api_client::BastionClient;
use crate::models::BastionSku;
use crate::tunnel::TunnelServer;

pub async fn execute(
    resource_group: &str,
    bastion_name: &str,
    target_resource_id: Option<&str>,
    target_ip_address: Option<&str>,
    resource_port: u16,
    local_port: u16,
    timeout: Option<u64>,
) -> Result<()> {
    let client = Arc::new(BastionClient::new().await?);
    let bastion = client.show(resource_group, bastion_name).await?;

    let sku = bastion
        .sku
        .as_ref()
        .context("Bastion has no SKU")?
        .name;

    let props = bastion.properties.as_ref().context("Bastion has no properties")?;

    if !sku.is_standard_or_higher() || props.enable_tunneling != Some(true) {
        anyhow::bail!("Bastion Host SKU must be Standard or Premium and Native Client must be enabled.");
    }

    let (resolved_resource_id, hostname) = resolve_target(
        &client,
        &bastion,
        resource_group,
        target_resource_id,
        target_ip_address,
        resource_port,
    )?;

    let bastion_endpoint = get_bastion_endpoint(&client, &bastion, &resolved_resource_id, resource_port).await?;

    let tunnel = TunnelServer::new(
        Arc::clone(&client),
        local_port,
        bastion_endpoint,
        sku,
        resolved_resource_id,
        resource_port,
        hostname,
    )
    .await?;

    let actual_port = tunnel.local_port();
    warn!("Tunnel is ready, connect on port {actual_port}");
    warn!("Ctrl + C to close");

    let tunnel_clone = Arc::clone(&tunnel);
    tokio::spawn(async move {
        if let Err(e) = tunnel_clone.run().await {
            tracing::error!("Tunnel server error: {e:#}");
        }
    });

    match timeout {
        Some(secs) => {
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(secs)) => {
                    info!("Timeout reached, shutting down tunnel");
                }
                _ = signal::ctrl_c() => {
                    info!("Ctrl+C received, shutting down tunnel");
                }
            }
        }
        None => {
            signal::ctrl_c().await?;
            info!("Ctrl+C received, shutting down tunnel");
        }
    }

    tunnel.cleanup().await?;
    Ok(())
}

pub(crate) fn resolve_target(
    client: &BastionClient,
    bastion: &crate::models::BastionHost,
    resource_group: &str,
    target_resource_id: Option<&str>,
    target_ip_address: Option<&str>,
    resource_port: u16,
) -> Result<(String, Option<String>)> {
    let props = bastion.properties.as_ref().context("Bastion has no properties")?;

    if let Some(ip) = target_ip_address {
        ip.parse::<std::net::IpAddr>()
            .map_err(|_| anyhow::anyhow!("Invalid IP address: {ip}"))?;

        if props.enable_ip_connect != Some(true) {
            anyhow::bail!(
                "--target-ip-address cannot be used when IpConnect is not enabled. Use --target-resource-id instead."
            );
        }

        if ![22, 3389].contains(&resource_port) {
            anyhow::bail!("Custom ports are not allowed for IP connect. Allowed ports: 22, 3389.");
        }

        let resource_id = format!(
            "/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/bh-hostConnect/{}",
            client.subscription_id(),
            resource_group,
            ip
        );

        return Ok((resource_id, Some(ip.to_string())));
    }

    let resource_id = target_resource_id
        .context("Either --target-resource-id or --target-ip-address must be provided")?
        .to_string();

    Ok((resource_id, None))
}

pub(crate) async fn get_bastion_endpoint(
    client: &BastionClient,
    bastion: &crate::models::BastionHost,
    resource_id: &str,
    resource_port: u16,
) -> Result<String> {
    let sku = bastion.sku.as_ref().context("Bastion has no SKU")?.name;
    let props = bastion.properties.as_ref().context("Bastion has no properties")?;

    match sku {
        BastionSku::QuickConnect | BastionSku::Developer => {
            client
                .get_developer_endpoint(bastion, resource_id, resource_port)
                .await
        }
        _ => props
            .dns_name
            .clone()
            .context("Bastion host has no DNS name"),
    }
}
