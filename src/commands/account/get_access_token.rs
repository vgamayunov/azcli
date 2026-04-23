use anyhow::Result;

use crate::auth::token_provider::TokenProvider;

pub async fn execute(provider: &mut TokenProvider) -> Result<serde_json::Value> {
    let token = provider.get_access_token().await?;
    let cache = provider.cache();
    let account = cache.active_account();

    Ok(serde_json::json!({
        "accessToken": token,
        "expiresOn": account.and_then(|a| a.expires_at).map(|d| d.to_rfc3339()),
        "subscription": account.and_then(|a| a.subscription_id.clone()),
        "tenant": account.map(|a| a.tenant_id.clone()),
        "tokenType": "Bearer",
    }))
}
