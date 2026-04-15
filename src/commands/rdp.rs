use std::sync::Arc;

use anyhow::{Context, Result};
use tracing::{info, warn};

use crate::api_client::BastionClient;
use crate::models::AuthType;
use crate::tunnel::TunnelServer;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    resource_group: &str,
    bastion_name: &str,
    target_resource_id: Option<&str>,
    target_ip_address: Option<&str>,
    resource_port: u16,
    auth_type: Option<AuthType>,
    disable_gateway: bool,
    configure: bool,
    enable_mfa: bool,
) -> Result<()> {
    let client = Arc::new(BastionClient::new().await?);
    let bastion = client.show(resource_group, bastion_name).await?;

    let sku = bastion.sku.as_ref().context("Bastion has no SKU")?.name;
    let props = bastion.properties.as_ref().context("Bastion has no properties")?;

    if !sku.is_standard_or_higher() || props.enable_tunneling != Some(true) {
        anyhow::bail!("Bastion Host SKU must be Standard or Premium and Native Client must be enabled.");
    }

    let ip_connect = target_ip_address.is_some() && props.enable_ip_connect == Some(true);

    let is_aad = matches!(auth_type, Some(AuthType::Aad));
    let enable_mfa = enable_mfa || is_aad;

    if is_aad && (disable_gateway || ip_connect) {
        anyhow::bail!("AAD login is not supported for Disable Gateway & IP Connect scenarios.");
    }

    let (resolved_resource_id, hostname) = super::tunnel::resolve_target(
        &client,
        &bastion,
        resource_group,
        target_resource_id,
        target_ip_address,
        resource_port,
    )?;

    let bastion_endpoint =
        super::tunnel::get_bastion_endpoint(&client, &bastion, &resolved_resource_id, resource_port).await?;

    if disable_gateway || ip_connect {
        run_tunnel_mode(
            client,
            bastion_endpoint,
            sku,
            resolved_resource_id,
            resource_port,
            hostname,
            configure,
        )
        .await
    } else {
        run_web_mode(
            &client,
            &bastion_endpoint,
            &resolved_resource_id,
            resource_port,
            enable_mfa,
            configure,
        )
        .await
    }
}

async fn run_tunnel_mode(
    client: Arc<BastionClient>,
    bastion_endpoint: String,
    sku: crate::models::BastionSku,
    resource_id: String,
    resource_port: u16,
    hostname: Option<String>,
    configure: bool,
) -> Result<()> {
    let tunnel = TunnelServer::new(
        Arc::clone(&client),
        0,
        bastion_endpoint,
        sku,
        resource_id,
        resource_port,
        hostname,
    )
    .await?;

    let local_port = tunnel.local_port();

    let tunnel_clone = Arc::clone(&tunnel);
    tokio::spawn(async move {
        if let Err(e) = tunnel_clone.run().await {
            tracing::error!("Tunnel server error: {e:#}");
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let rdp_file_content = format!(
        "full address:s:localhost:{local_port}\n\
         alternate full address:s:localhost:{local_port}\n\
         use multimon:i:0\n"
    );

    let rdp_path = write_rdp_file(&rdp_file_content)?;
    warn!("RDP tunnel ready on port {local_port}");

    launch_rdp_client(&rdp_path, configure)?;

    tunnel.cleanup().await?;
    let _ = std::fs::remove_file(&rdp_path);
    Ok(())
}

async fn run_web_mode(
    client: &BastionClient,
    bastion_endpoint: &str,
    resource_id: &str,
    resource_port: u16,
    enable_mfa: bool,
    configure: bool,
) -> Result<()> {
    info!("Fetching RDP file from bastion...");
    let rdp_content = client
        .get_rdp_file(bastion_endpoint, resource_id, resource_port, enable_mfa)
        .await?;

    let rdp_path = write_rdp_file(&rdp_content)?;
    info!("RDP file saved to {rdp_path}");

    launch_rdp_client(&rdp_path, configure)?;

    let _ = std::fs::remove_file(&rdp_path);
    Ok(())
}

fn write_rdp_file(content: &str) -> Result<String> {
    let path = std::env::temp_dir().join(format!("conn_{}.rdp", uuid::Uuid::new_v4().as_hyphenated()));
    std::fs::write(&path, content).with_context(|| format!("Failed to write RDP file to {}", path.display()))?;
    Ok(path.to_string_lossy().to_string())
}

fn launch_rdp_client(rdp_path: &str, configure: bool) -> Result<()> {
    let rdp_client = find_rdp_client()?;

    let mut cmd = std::process::Command::new(&rdp_client);
    if configure {
        cmd.arg("/edit");
    }
    cmd.arg(rdp_path);

    let status = cmd
        .status()
        .with_context(|| format!("Failed to launch {rdp_client}"))?;

    info!("RDP client exited with status: {status}");
    Ok(())
}

fn find_rdp_client() -> Result<String> {
    for candidate in &["mstsc.exe", "xfreerdp", "wlfreerdp", "rdesktop"] {
        if which::which(candidate).is_ok() {
            return Ok(candidate.to_string());
        }
    }

    anyhow::bail!(
        "No RDP client found in PATH. Install xfreerdp (FreeRDP), rdesktop, or run on Windows with mstsc."
    )
}
