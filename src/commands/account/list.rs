use anyhow::Result;

use crate::auth::{list_subscriptions, token_provider::TokenProvider};

pub async fn execute(provider: &mut TokenProvider) -> Result<serde_json::Value> {
    let token = provider.get_access_token().await?;
    let subs = list_subscriptions(&token).await?;
    let active = provider.cache_default_subscription();

    let out: Vec<serde_json::Value> = subs
        .into_iter()
        .map(|s| {
            let sub_id = s.id.rsplit('/').next().unwrap_or(&s.id).to_string();
            let is_default = active.as_deref() == Some(&sub_id);
            serde_json::json!({
                "id": sub_id,
                "name": s.display_name,
                "tenantId": s.tenant_id,
                "state": s.state,
                "isDefault": is_default,
            })
        })
        .collect();

    Ok(serde_json::Value::Array(out))
}
