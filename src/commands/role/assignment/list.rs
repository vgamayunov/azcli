use anyhow::Result;

use crate::arm_client::ArmClient;
use crate::commands::role::pim::{resolve_scope, RoleNameCache};

pub async fn execute(
    client: &ArmClient,
    assignee: Option<&str>,
    role: Option<&str>,
    scope: Option<&str>,
    resource_group: Option<&str>,
    subscription: Option<&str>,
    include_groups: bool,
    all: bool,
) -> Result<serde_json::Value> {
    let effective_scope = if all {
        format!("/subscriptions/{}", subscription.unwrap_or_else(|| client.subscription_id()))
    } else if let Some(rg) = resource_group {
        format!(
            "/subscriptions/{}/resourceGroups/{}",
            subscription.unwrap_or_else(|| client.subscription_id()),
            rg
        )
    } else {
        resolve_scope(client, scope, subscription)
    };

    let filter = build_filter(assignee, all, include_groups, scope.is_some() || resource_group.is_some());

    let raw = client.list_role_assignments(&effective_scope, filter.as_deref()).await?;
    let items = raw.get("value").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    let mut role_filter_lower: Option<String> = role.map(|r| r.to_lowercase());
    let role_is_guid = role.map(|r| looks_like_guid(r)).unwrap_or(false);

    let mut cache = RoleNameCache::new(client);
    let mut out = Vec::with_capacity(items.len());
    for assignment in items {
        let props = assignment.get("properties").cloned().unwrap_or(serde_json::Value::Null);
        let role_def_id = props.get("roleDefinitionId").and_then(|v| v.as_str()).unwrap_or("");
        let role_name = cache.resolve(role_def_id).await;

        if let Some(needle) = &role_filter_lower {
            let matched = if role_is_guid {
                role_def_id.to_lowercase().ends_with(needle)
            } else {
                role_name.to_lowercase() == *needle
            };
            if !matched {
                continue;
            }
        } else {
            role_filter_lower = None;
        }

        out.push(serde_json::json!({
            "roleName": role_name,
            "principalId": props.get("principalId"),
            "principalType": props.get("principalType"),
            "scope": props.get("scope"),
            "roleDefinitionId": role_def_id,
            "name": assignment.get("name"),
            "id": assignment.get("id"),
            "createdOn": props.get("createdOn"),
            "updatedOn": props.get("updatedOn"),
            "createdBy": props.get("createdBy"),
            "updatedBy": props.get("updatedBy"),
            "condition": props.get("condition"),
            "conditionVersion": props.get("conditionVersion"),
            "description": props.get("description"),
        }));
    }

    Ok(serde_json::Value::Array(out))
}

fn build_filter(
    assignee: Option<&str>,
    all: bool,
    include_groups: bool,
    has_explicit_scope: bool,
) -> Option<String> {
    match (assignee, all, has_explicit_scope) {
        (Some(oid), true, _) | (Some(oid), false, false) => {
            if include_groups {
                Some(format!("assignedTo('{}')", oid))
            } else {
                Some(format!("principalId eq '{}'", oid))
            }
        }
        (Some(oid), false, true) => {
            if include_groups {
                Some(format!("atScope() and assignedTo('{}')", oid))
            } else {
                Some("atScope()".to_string())
            }
        }
        (None, true, _) => None,
        (None, false, _) => Some("atScope()".to_string()),
    }
}

fn looks_like_guid(s: &str) -> bool {
    s.len() == 36 && s.chars().enumerate().all(|(i, c)| match i {
        8 | 13 | 18 | 23 => c == '-',
        _ => c.is_ascii_hexdigit(),
    })
}
