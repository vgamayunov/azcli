use anyhow::{Context, Result, bail};
use chrono::{Duration, Utc};
use tracing::debug;

use super::cache::{AuthMethod, CachedAccount};
use super::{OAuthErrorResponse, OAuthTokenResponse, token_endpoint};

pub async fn login(
    tenant: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<CachedAccount> {
    let client = reqwest::Client::new();

    let resp = client
        .post(token_endpoint(tenant))
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("grant_type", "client_credentials"),
            ("scope", super::MANAGEMENT_SCOPE),
        ])
        .send()
        .await
        .context("Service principal token request failed")?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        if let Ok(err) = serde_json::from_str::<OAuthErrorResponse>(&body) {
            bail!(
                "Service principal auth failed: {}: {}",
                err.error,
                err.error_description.unwrap_or_default()
            );
        }
        bail!("Service principal auth failed: {body}");
    }

    let token_resp: OAuthTokenResponse = resp.json().await.context("Failed to parse token response")?;

    let expires_at = token_resp
        .expires_in
        .map(|secs| Utc::now() + Duration::seconds(secs));

    debug!(
        "Service principal login successful (token {} chars)",
        token_resp.access_token.len()
    );

    Ok(CachedAccount {
        auth_method: AuthMethod::ServicePrincipalSecret,
        tenant_id: tenant.to_string(),
        subscription_id: None,
        subscription_name: None,
        access_token: Some(token_resp.access_token),
        refresh_token: None,
        expires_at,
        client_id: Some(client_id.to_string()),
        client_secret: Some(client_secret.to_string()),
        client_certificate_path: None,
        managed_identity_client_id: None,
    })
}
