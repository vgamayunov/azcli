use anyhow::Result;

use crate::arm_client::ArmClient;
use crate::commands::role::pim::resolve_scope;

pub async fn execute(
    client: &ArmClient,
    name: &str,
    scope: Option<&str>,
    subscription: Option<&str>,
) -> Result<serde_json::Value> {
    let scope = resolve_scope(client, scope, subscription);

    let filter = if looks_like_guid(name) {
        None
    } else {
        Some(format!("roleName eq '{}'", name))
    };

    let raw = client.list_role_definitions(&scope, filter.as_deref()).await?;
    let items = raw.get("value").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    let matched: Vec<serde_json::Value> = if looks_like_guid(name) {
        items
            .into_iter()
            .filter(|d| {
                d.get("name")
                    .and_then(|v| v.as_str())
                    .map(|n| n.eq_ignore_ascii_case(name))
                    .unwrap_or(false)
            })
            .collect()
    } else {
        items
    };

    match matched.len() {
        0 => anyhow::bail!("no role definition found matching {:?} at scope {}", name, scope),
        1 => {
            let def = &matched[0];
            let props = def.get("properties").cloned().unwrap_or(serde_json::Value::Null);
            Ok(serde_json::json!({
                "roleName": props.get("roleName"),
                "type": props.get("type"),
                "description": props.get("description"),
                "name": def.get("name"),
                "id": def.get("id"),
                "assignableScopes": props.get("assignableScopes"),
                "permissions": props.get("permissions"),
            }))
        }
        _ => anyhow::bail!(
            "multiple role definitions match {:?}; pass the GUID name to disambiguate",
            name
        ),
    }
}

fn looks_like_guid(s: &str) -> bool {
    s.len() == 36 && s.chars().enumerate().all(|(i, c)| match i {
        8 | 13 | 18 | 23 => c == '-',
        _ => c.is_ascii_hexdigit(),
    })
}
