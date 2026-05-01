use anyhow::Result;

use crate::auth::{list_subscriptions, token_provider::TokenProvider};

pub async fn execute(provider: &mut TokenProvider, name_or_id: &str) -> Result<serde_json::Value> {
    if let Some(cached) = provider.cache().find_by_selector(name_or_id) {
        let sub_id = cached.subscription_id.clone();
        let sub_name = cached.subscription_name.clone();
        let tenant_id = cached.tenant_id.clone();
        let profile = cached.profile.clone();
        if let Some(ref sid) = sub_id {
            let cache = provider.cache_mut();
            cache.default_subscription = Some(sid.clone());
            provider.save_cache()?;
            return Ok(serde_json::json!({
                "id": sid,
                "name": sub_name,
                "tenantId": tenant_id,
                "profile": profile,
                "isDefault": true,
            }));
        }
    }

    let token = provider.get_access_token().await?;
    let subs = list_subscriptions(&token).await?;

    let matched = subs.iter().find(|s| {
        let sub_id = s.id.rsplit('/').next().unwrap_or(&s.id);
        sub_id == name_or_id || s.id == name_or_id || s.display_name.as_deref() == Some(name_or_id)
    });

    let sub = matched.ok_or_else(|| {
        anyhow::anyhow!(
            "subscription {:?} not found in {} accessible subscriptions",
            name_or_id,
            subs.len()
        )
    })?;
    let sub_id = sub.id.rsplit('/').next().unwrap_or(&sub.id).to_string();

    {
        let cache = provider.cache_mut();
        cache.default_subscription = Some(sub_id.clone());
        if let Some(account) = cache
            .accounts
            .iter_mut()
            .find(|a| a.subscription_id.as_deref() == Some(&sub_id))
        {
            account.subscription_name = sub.display_name.clone();
        } else if let Some(template) = cache.accounts.first().cloned() {
            cache.accounts.insert(
                0,
                crate::auth::cache::CachedAccount {
                    subscription_id: Some(sub_id.clone()),
                    subscription_name: sub.display_name.clone(),
                    profile: None,
                    tenant_id: sub.tenant_id.clone().unwrap_or(template.tenant_id),
                    access_token: None,
                    refresh_token: template.refresh_token,
                    expires_at: None,
                    ..template
                },
            );
        }
    }
    provider.save_cache()?;

    Ok(serde_json::json!({
        "id": sub_id,
        "name": sub.display_name,
        "tenantId": sub.tenant_id,
        "isDefault": true,
    }))
}
