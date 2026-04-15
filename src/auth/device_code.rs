use anyhow::{Context, Result, bail};
use chrono::{Duration, Utc};
use serde::Deserialize;
use tracing::debug;

use super::cache::{AuthMethod, CachedAccount};
use super::{
    AZURE_CLI_CLIENT_ID, COMMON_TENANT, MANAGEMENT_SCOPE, OAuthErrorResponse,
    OAuthTokenResponse, device_code_endpoint, token_endpoint,
};

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: i64,
    interval: Option<u64>,
    message: Option<String>,
}

pub async fn login(tenant: Option<&str>) -> Result<CachedAccount> {
    let tenant = tenant.unwrap_or(COMMON_TENANT);
    let client = reqwest::Client::new();

    let resp = client
        .post(device_code_endpoint(tenant))
        .form(&[
            ("client_id", AZURE_CLI_CLIENT_ID),
            ("scope", MANAGEMENT_SCOPE),
        ])
        .send()
        .await
        .context("Device code request failed")?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("Device code request failed: {body}");
    }

    let dc: DeviceCodeResponse = resp.json().await.context("Failed to parse device code response")?;

    if let Some(ref msg) = dc.message {
        eprintln!("{msg}");
    } else {
        eprintln!(
            "To sign in, use a web browser to open {} and enter the code {}",
            dc.verification_uri, dc.user_code
        );
    }

    let poll_interval = std::time::Duration::from_secs(dc.interval.unwrap_or(5));
    let deadline = Utc::now() + Duration::seconds(dc.expires_in);

    loop {
        if Utc::now() >= deadline {
            bail!("Device code expired. Please try again.");
        }

        tokio::time::sleep(poll_interval).await;

        let resp = client
            .post(token_endpoint(tenant))
            .form(&[
                ("client_id", AZURE_CLI_CLIENT_ID),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", &dc.device_code),
            ])
            .send()
            .await
            .context("Token poll request failed")?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();

        if status.is_success() {
            let token_resp: OAuthTokenResponse =
                serde_json::from_str(&body).context("Failed to parse token response")?;

            let expires_at = token_resp
                .expires_in
                .map(|secs| Utc::now() + Duration::seconds(secs));

            debug!(
                "Device code login successful (token {} chars)",
                token_resp.access_token.len()
            );

            return Ok(CachedAccount {
                auth_method: AuthMethod::DeviceCode,
                tenant_id: tenant.to_string(),
                subscription_id: None,
                subscription_name: None,
                access_token: Some(token_resp.access_token),
                refresh_token: token_resp.refresh_token,
                expires_at,
                client_id: None,
                client_secret: None,
                client_certificate_path: None,
                managed_identity_client_id: None,
            });
        }

        if let Ok(err) = serde_json::from_str::<OAuthErrorResponse>(&body) {
            match err.error.as_str() {
                "authorization_pending" => {
                    debug!("Authorization pending, polling again...");
                    continue;
                }
                "slow_down" => {
                    debug!("Slow down requested, increasing interval");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    continue;
                }
                "expired_token" => {
                    bail!("Device code expired. Please try again.");
                }
                _ => {
                    bail!(
                        "Device code auth failed: {}: {}",
                        err.error,
                        err.error_description.unwrap_or_default()
                    );
                }
            }
        }

        bail!("Unexpected token response ({status}): {body}");
    }
}
