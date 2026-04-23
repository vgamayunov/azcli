use anyhow::Result;

use crate::auth::token_provider::TokenProvider;

pub fn execute(provider: &TokenProvider, name_or_id: Option<&str>) -> Result<serde_json::Value> {
    let cache = provider.cache();
    let target = match name_or_id {
        Some(needle) => cache
            .accounts
            .iter()
            .find(|a| {
                a.subscription_id.as_deref() == Some(needle)
                    || a.subscription_name.as_deref() == Some(needle)
            })
            .ok_or_else(|| anyhow::anyhow!("no cached account matches {:?}", needle))?,
        None => cache
            .active_account()
            .ok_or_else(|| anyhow::anyhow!("not logged in. Run 'azcli login'."))?,
    };

    let active = provider.cache_default_subscription();
    let is_default = active.as_deref() == target.subscription_id.as_deref();

    Ok(serde_json::json!({
        "id": target.subscription_id,
        "name": target.subscription_name,
        "tenantId": target.tenant_id,
        "authMethod": format!("{:?}", target.auth_method),
        "isDefault": is_default,
        "tokenExpiresAt": target.expires_at,
    }))
}
