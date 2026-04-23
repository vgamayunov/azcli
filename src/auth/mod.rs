pub mod cache;
pub mod device_code;
pub mod interactive;
pub mod managed_identity;
pub mod service_principal;
pub mod token_provider;

pub use token_provider::TokenProvider;

use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::debug;

pub const AZURE_CLI_CLIENT_ID: &str = "04b07795-8ddb-461a-bbee-02f9e1bf7b46";
pub const MANAGEMENT_SCOPE: &str = "https://management.azure.com/.default offline_access";
pub const COMMON_TENANT: &str = "organizations";

pub fn token_endpoint(tenant: &str) -> String {
    format!("https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token")
}

pub fn authorize_endpoint(tenant: &str) -> String {
    format!("https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize")
}

pub fn device_code_endpoint(tenant: &str) -> String {
    format!("https://login.microsoftonline.com/{tenant}/oauth2/v2.0/devicecode")
}

#[derive(Debug, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub token_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionInfo {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SubscriptionListResponse {
    value: Vec<SubscriptionInfo>,
}

pub async fn list_subscriptions(access_token: &str) -> Result<Vec<SubscriptionInfo>> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://management.azure.com/subscriptions?api-version=2022-12-01")
        .bearer_auth(access_token)
        .send()
        .await
        .context("Failed to list subscriptions")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("List subscriptions failed ({status}): {body}");
    }

    let list: SubscriptionListResponse = resp.json().await.context("Failed to parse subscription list")?;
    Ok(list.value)
}

pub async fn get_access_token_az_cli() -> Result<String> {
    let output = tokio::process::Command::new("az")
        .args(["account", "get-access-token", "--query", "accessToken", "-o", "tsv"])
        .output()
        .await
        .context("Failed to run 'az account get-access-token'. Is Azure CLI installed and logged in?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("az account get-access-token failed: {stderr}");
    }

    let token = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in access token")?
        .trim()
        .to_string();

    if token.is_empty() {
        anyhow::bail!("Empty access token from az CLI");
    }

    debug!("Acquired access token from az CLI ({} chars)", token.len());
    Ok(token)
}

pub async fn get_subscription_id_az_cli() -> Result<String> {
    let output = tokio::process::Command::new("az")
        .args(["account", "show", "--query", "id", "-o", "tsv"])
        .output()
        .await
        .context("Failed to run 'az account show'. Is Azure CLI installed and logged in?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("az account show failed: {stderr}");
    }

    let sub_id = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in subscription ID")?
        .trim()
        .to_string();

    if sub_id.is_empty() {
        anyhow::bail!("Empty subscription ID from az CLI");
    }

    debug!("Using subscription from az CLI: {sub_id}");
    Ok(sub_id)
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantInfo {
    #[serde(rename = "tenantId")]
    pub tenant_id: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "defaultDomain")]
    pub default_domain: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TenantListResponse {
    value: Vec<TenantInfo>,
}

pub async fn list_tenants(access_token: &str) -> Result<Vec<TenantInfo>> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://management.azure.com/tenants?api-version=2022-12-01")
        .bearer_auth(access_token)
        .send()
        .await
        .context("Failed to list tenants")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("List tenants failed ({status}): {body}");
    }

    let list: TenantListResponse = resp.json().await.context("Failed to parse tenant list")?;
    Ok(list.value)
}

pub async fn acquire_tenant_token(
    refresh_token: &str,
    tenant_id: &str,
) -> Result<OAuthTokenResponse> {
    let client = reqwest::Client::new();
    let resp = client
        .post(token_endpoint(tenant_id))
        .form(&[
            ("client_id", AZURE_CLI_CLIENT_ID),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("scope", MANAGEMENT_SCOPE),
        ])
        .send()
        .await
        .context("Tenant token exchange request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Tenant token exchange failed for {tenant_id} ({status}): {body}");
    }

    resp.json()
        .await
        .context("Failed to parse tenant token response")
}

pub fn decode_jwt_claims(token: &str) -> Result<serde_json::Value> {
    use base64::Engine;
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        anyhow::bail!("invalid JWT: expected 3 segments, got {}", parts.len());
    }
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .context("failed to base64-decode JWT payload")?;
    serde_json::from_slice(&decoded).context("failed to parse JWT payload as JSON")
}
