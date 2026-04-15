use std::collections::HashMap;

use anyhow::Result;

use crate::api_client::BastionClient;
use crate::models::BastionSku;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    resource_group: &str,
    name: &str,
    sku: Option<BastionSku>,
    enable_tunneling: Option<bool>,
    enable_ip_connect: Option<bool>,
    file_copy: Option<bool>,
    disable_copy_paste: Option<bool>,
    kerberos: Option<bool>,
    session_recording: Option<bool>,
    shareable_link: Option<bool>,
    network_acls_ips: Option<Vec<String>>,
    tags: Option<HashMap<String, String>>,
) -> Result<serde_json::Value> {
    let client = BastionClient::new().await?;
    let bastion = client
        .update(
            resource_group,
            name,
            sku,
            enable_tunneling,
            enable_ip_connect,
            file_copy,
            disable_copy_paste,
            kerberos,
            session_recording,
            shareable_link,
            network_acls_ips.as_deref(),
            tags,
        )
        .await?;

    Ok(serde_json::to_value(&bastion)?)
}
