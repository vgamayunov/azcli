use anyhow::Result;

use crate::auth::token_provider::TokenProvider;

pub fn execute(provider: &TokenProvider) -> Result<serde_json::Value> {
    let cache = provider.cache();
    let active = provider.cache_default_subscription();
    let rows: Vec<serde_json::Value> = cache
        .accounts
        .iter()
        .filter(|a| a.profile.is_some())
        .map(|a| {
            let is_default = active.as_deref() == a.subscription_id.as_deref();
            serde_json::json!({
                "profile": a.profile,
                "subscriptionId": a.subscription_id,
                "subscriptionName": a.subscription_name,
                "tenantId": a.tenant_id,
                "authMethod": format!("{:?}", a.auth_method),
                "isDefault": is_default,
                "tokenExpiresAt": a.expires_at,
            })
        })
        .collect();
    Ok(serde_json::Value::Array(rows))
}
