use std::collections::HashMap;

use anyhow::Result;
use tracing::{debug, warn};

use crate::auth::{
    SubscriptionInfo, TenantInfo, acquire_tenant_token, decode_jwt_claims, list_subscriptions,
    list_tenants, token_provider::TokenProvider,
};

pub async fn execute(provider: &mut TokenProvider) -> Result<serde_json::Value> {
    let home_token = provider.get_access_token().await?;
    let claims = decode_jwt_claims(&home_token).unwrap_or(serde_json::Value::Null);
    let user_name = claims
        .get("preferred_username")
        .or_else(|| claims.get("upn"))
        .or_else(|| claims.get("unique_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let user_type = if claims.get("idtyp").and_then(|v| v.as_str()) == Some("app") {
        "servicePrincipal"
    } else {
        "user"
    };
    let home_tenant = claims
        .get("tid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let tenants = list_tenants(&home_token).await?;
    debug!("found {} tenants", tenants.len());

    let refresh_token = provider
        .cache()
        .active_account()
        .and_then(|a| a.refresh_token.clone());

    let active_default = provider.cache_default_subscription();

    let tenant_map: HashMap<String, TenantInfo> = tenants
        .iter()
        .cloned()
        .map(|t| (t.tenant_id.clone(), t))
        .collect();

    let mut all_rows: Vec<serde_json::Value> = Vec::new();

    let mut joinset = tokio::task::JoinSet::new();
    for tenant in tenants.iter().cloned() {
        let home_token = home_token.clone();
        let refresh_token = refresh_token.clone();
        let home_tenant = home_tenant.clone();
        joinset.spawn(async move {
            let token = if tenant.tenant_id == home_tenant {
                home_token
            } else if let Some(rt) = refresh_token {
                match acquire_tenant_token(&rt, &tenant.tenant_id).await {
                    Ok(resp) => resp.access_token,
                    Err(e) => {
                        warn!("skipping tenant {}: {e:#}", tenant.tenant_id);
                        return (tenant, Vec::<SubscriptionInfo>::new());
                    }
                }
            } else {
                warn!("skipping tenant {}: no refresh token", tenant.tenant_id);
                return (tenant, Vec::new());
            };

            match list_subscriptions(&token).await {
                Ok(subs) => (tenant, subs),
                Err(e) => {
                    warn!(
                        "list subscriptions failed for tenant {}: {e:#}",
                        tenant.tenant_id
                    );
                    (tenant, Vec::new())
                }
            }
        });
    }

    while let Some(joined) = joinset.join_next().await {
        let (tenant, subs) = joined?;
        for sub in subs {
            all_rows.push(render_subscription(
                &sub,
                &tenant,
                &tenant_map,
                &user_name,
                user_type,
                active_default.as_deref(),
            ));
        }
    }

    all_rows.sort_by(|a, b| {
        a.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .cmp(b.get("name").and_then(|v| v.as_str()).unwrap_or(""))
    });

    Ok(serde_json::Value::Array(all_rows))
}

fn render_subscription(
    sub: &SubscriptionInfo,
    listing_tenant: &TenantInfo,
    tenant_map: &HashMap<String, TenantInfo>,
    user_name: &str,
    user_type: &str,
    active_default: Option<&str>,
) -> serde_json::Value {
    let sub_id = sub.id.rsplit('/').next().unwrap_or(&sub.id).to_string();
    let is_default = active_default == Some(&sub_id);
    let home_tenant_id = sub
        .tenant_id
        .as_deref()
        .unwrap_or(&listing_tenant.tenant_id);
    let tenant_for_meta = tenant_map.get(home_tenant_id).unwrap_or(listing_tenant);

    serde_json::json!({
        "cloudName": "AzureCloud",
        "homeTenantId": home_tenant_id,
        "id": sub_id,
        "isDefault": is_default,
        "managedByTenants": Vec::<serde_json::Value>::new(),
        "name": sub.display_name,
        "state": sub.state,
        "tenantDefaultDomain": tenant_for_meta.default_domain,
        "tenantDisplayName": tenant_for_meta.display_name,
        "tenantId": listing_tenant.tenant_id,
        "user": {
            "name": user_name,
            "type": user_type,
        },
    })
}
