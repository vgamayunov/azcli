use anyhow::{Context, Result};
use tracing::debug;

pub async fn get_access_token() -> Result<String> {
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
        anyhow::bail!("Empty access token. Run 'az login' first.");
    }

    debug!("Acquired access token ({} chars)", token.len());
    Ok(token)
}

pub async fn get_subscription_id() -> Result<String> {
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
        anyhow::bail!("Empty subscription ID. Run 'az login' first.");
    }

    debug!("Using subscription: {sub_id}");
    Ok(sub_id)
}
