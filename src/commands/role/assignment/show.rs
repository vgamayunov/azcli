use anyhow::{Context, Result};

use crate::arm_client::ArmClient;
use crate::commands::role::pim::{resolve_scope, RoleNameCache};

pub async fn execute(
    client: &ArmClient,
    ids: Option<&str>,
    name: Option<&str>,
    scope: Option<&str>,
    subscription: Option<&str>,
) -> Result<serde_json::Value> {
    let full_id = match (ids, name) {
        (Some(id), _) => id.to_string(),
        (None, Some(n)) => {
            let scope = resolve_scope(client, scope, subscription);
            format!(
                "{}/providers/Microsoft.Authorization/roleAssignments/{}",
                scope.trim_end_matches('/'),
                n
            )
        }
        (None, None) => anyhow::bail!("must provide --ids or --name"),
    };

    let raw = client.get_role_assignment_by_id(&full_id).await
        .context("failed to fetch role assignment")?;

    let props = raw.get("properties").cloned().unwrap_or(serde_json::Value::Null);
    let role_def_id = props.get("roleDefinitionId").and_then(|v| v.as_str()).unwrap_or("");
    let mut cache = RoleNameCache::new(client);
    let role_name = cache.resolve(role_def_id).await;

    Ok(serde_json::json!({
        "roleName": role_name,
        "principalId": props.get("principalId"),
        "principalType": props.get("principalType"),
        "scope": props.get("scope"),
        "roleDefinitionId": role_def_id,
        "name": raw.get("name"),
        "id": raw.get("id"),
        "createdOn": props.get("createdOn"),
        "updatedOn": props.get("updatedOn"),
        "createdBy": props.get("createdBy"),
        "updatedBy": props.get("updatedBy"),
        "condition": props.get("condition"),
        "conditionVersion": props.get("conditionVersion"),
        "description": props.get("description"),
    }))
}
