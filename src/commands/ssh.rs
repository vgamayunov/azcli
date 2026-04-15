use std::process::Command as StdCommand;
use std::sync::Arc;

use anyhow::{Context, Result};
use tracing::{debug, info};

use crate::api_client::BastionClient;
use crate::models::AuthType;
use crate::tunnel::TunnelServer;

pub async fn execute(
    resource_group: &str,
    bastion_name: &str,
    auth_type: AuthType,
    target_resource_id: Option<&str>,
    target_ip_address: Option<&str>,
    resource_port: u16,
    username: Option<&str>,
    ssh_key: Option<&str>,
    extra_args: Vec<String>,
) -> Result<()> {
    let client = Arc::new(BastionClient::new().await?);
    execute_inner(
        client,
        resource_group,
        bastion_name,
        auth_type,
        target_resource_id,
        target_ip_address,
        resource_port,
        username,
        ssh_key,
        extra_args,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn execute_with_client(
    client: &BastionClient,
    resource_group: &str,
    bastion_name: &str,
    auth_type: AuthType,
    target_resource_id: Option<&str>,
    target_ip_address: Option<&str>,
    resource_port: u16,
    username: Option<&str>,
    ssh_key: Option<&str>,
    extra_args: Vec<String>,
) -> Result<()> {
    let client = Arc::new(client.clone());
    execute_inner(
        client,
        resource_group,
        bastion_name,
        auth_type,
        target_resource_id,
        target_ip_address,
        resource_port,
        username,
        ssh_key,
        extra_args,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn execute_inner(
    client: Arc<BastionClient>,
    resource_group: &str,
    bastion_name: &str,
    auth_type: AuthType,
    target_resource_id: Option<&str>,
    target_ip_address: Option<&str>,
    resource_port: u16,
    username: Option<&str>,
    ssh_key: Option<&str>,
    extra_args: Vec<String>,
) -> Result<()> {
    let bastion = client.show(resource_group, bastion_name).await?;

    let sku = bastion.sku.as_ref().context("Bastion has no SKU")?.name;

    if !sku.supports_native_client() {
        anyhow::bail!("Bastion Host SKU must be Standard, Premium, or Developer and Native Client must be enabled.");
    }

    let props = bastion.properties.as_ref().context("Bastion has no properties")?;
    if !matches!(sku, crate::models::BastionSku::Developer) && props.enable_tunneling != Some(true) {
        anyhow::bail!("Native Client (tunneling) must be enabled on the Bastion host.");
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

    let tunnel = TunnelServer::new(
        Arc::clone(&client),
        0,
        bastion_endpoint,
        sku,
        resolved_resource_id,
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

    let ssh_path = find_ssh()?;

    let mut cmd_args: Vec<String> = Vec::new();

    match auth_type {
        AuthType::Password => {
            let user = username.context("--username is required for password auth")?;
            cmd_args.push(format!("{user}@localhost"));
        }
        AuthType::SshKey => {
            let user = username.context("--username is required for ssh-key auth")?;
            let key = ssh_key.context("--ssh-key is required for ssh-key auth")?;
            cmd_args.push(format!("{user}@localhost"));
            cmd_args.extend(["-i".to_string(), key.to_string()]);
        }
        AuthType::Aad => {
            cmd_args.push("localhost".to_string());
        }
    }

    cmd_args.extend(["-p".to_string(), local_port.to_string()]);
    cmd_args.extend([
        "-o".to_string(),
        "StrictHostKeyChecking=no".to_string(),
        "-o".to_string(),
        "UserKnownHostsFile=/dev/null".to_string(),
        "-o".to_string(),
        "LogLevel=Error".to_string(),
    ]);
    cmd_args.extend(extra_args);

    debug!("Running: {} {}", ssh_path, cmd_args.join(" "));

    let status = StdCommand::new(&ssh_path)
        .args(&cmd_args)
        .status()
        .with_context(|| format!("Failed to execute {ssh_path}"))?;

    info!("SSH exited with status: {status}");

    tunnel.cleanup().await?;
    Ok(())
}

fn find_ssh() -> Result<String> {
    which::which("ssh")
        .map(|p| p.to_string_lossy().to_string())
        .context("ssh not found in PATH. Is OpenSSH client installed?")
}
