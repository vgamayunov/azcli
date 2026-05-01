use anyhow::{Context, Result, bail};
use chrono::{Duration, Utc};
use serde::Deserialize;
use tracing::debug;

use super::cache::{AuthMethod, CachedAccount};

const IMDS_ENDPOINT: &str =
    "http://169.254.169.254/metadata/identity/oauth2/token";

#[derive(Debug, Deserialize)]
struct ImdsTokenResponse {
    access_token: String,
    expires_in: Option<String>,
    resource: Option<String>,
    token_type: Option<String>,
}

pub async fn login(client_id: Option<&str>) -> Result<CachedAccount> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .context("Failed to create HTTP client for IMDS")?;

    let mut req = client
        .get(IMDS_ENDPOINT)
        .query(&[
            ("api-version", "2018-02-01"),
            ("resource", "https://management.azure.com/"),
        ])
        .header("Metadata", "true");

    if let Some(cid) = client_id {
        req = req.query(&[("client_id", cid)]);
    }

    let resp = req.send().await.context(
        "Failed to contact IMDS endpoint. Is this running on an Azure VM/container with managed identity enabled?",
    )?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        bail!("Managed identity token request failed ({status}): {body}");
    }

    let token_resp: ImdsTokenResponse =
        resp.json().await.context("Failed to parse IMDS token response")?;

    let expires_at = token_resp
        .expires_in
        .and_then(|s| s.parse::<i64>().ok())
        .map(|secs| Utc::now() + Duration::seconds(secs));

    debug!(
        "Managed identity login successful (token {} chars, resource={:?})",
        token_resp.access_token.len(),
        token_resp.resource
    );

    Ok(CachedAccount {
        auth_method: AuthMethod::ManagedIdentity,
        tenant_id: "managed-identity".to_string(),
        subscription_id: None,
        subscription_name: None,
        profile: None,
        access_token: Some(token_resp.access_token),
        refresh_token: None,
        expires_at,
        client_id: None,
        client_secret: None,
        client_certificate_path: None,
        managed_identity_client_id: client_id.map(|s| s.to_string()),
    })
}
