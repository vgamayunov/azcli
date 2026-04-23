use anyhow::Result;

use crate::arm_client::ArmClient;
use crate::commands::role::pim::resolve_scope;

pub async fn execute(
    client: &ArmClient,
    name: Option<&str>,
    scope: Option<&str>,
    subscription: Option<&str>,
    custom_role_only: bool,
) -> Result<serde_json::Value> {
    let scope = resolve_scope(client, scope, subscription);
    let filter = name.map(|n| format!("roleName eq '{}'", n));
    let raw = client.list_role_definitions(&scope, filter.as_deref()).await?;
    let items = raw.get("value").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    let mut out = Vec::with_capacity(items.len());
    for def in items {
        let props = def.get("properties").cloned().unwrap_or(serde_json::Value::Null);
        let role_type = props.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if custom_role_only && role_type != "CustomRole" {
            continue;
        }
        out.push(serde_json::json!({
            "roleName": props.get("roleName"),
            "type": role_type,
            "description": props.get("description"),
            "name": def.get("name"),
            "id": def.get("id"),
            "assignableScopes": props.get("assignableScopes"),
            "permissions": props.get("permissions"),
        }));
    }
    Ok(serde_json::Value::Array(out))
}
