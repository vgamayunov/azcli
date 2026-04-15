use std::collections::HashMap;

use anyhow::Result;
use tracing::info;

use crate::api_client::BastionClient;
use crate::models::BastionSku;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    resource_group: &str,
    name: &str,
    location: &str,
    vnet_name: &str,
    public_ip: Option<&str>,
    sku: BastionSku,
    enable_tunneling: bool,
    enable_ip_connect: bool,
    file_copy: bool,
    disable_copy_paste: bool,
    kerberos: bool,
    session_recording: bool,
    shareable_link: bool,
    network_acls_ips: Option<Vec<String>>,
    zones: Option<Vec<String>>,
    tags: Option<HashMap<String, String>>,
) -> Result<serde_json::Value> {
    let client = BastionClient::new().await?;

    info!("Creating bastion host '{name}' in resource group '{resource_group}'...");

    let bastion = client
        .create(
            resource_group,
            name,
            location,
            vnet_name,
            public_ip,
            sku,
            enable_tunneling,
            enable_ip_connect,
            file_copy,
            disable_copy_paste,
            kerberos,
            session_recording,
            shareable_link,
            network_acls_ips.as_deref(),
            zones.as_deref(),
            tags,
        )
        .await?;

    Ok(serde_json::to_value(&bastion)?)
}
