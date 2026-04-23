pub mod activate;
pub mod deactivate;
pub mod list;
pub mod status;

use anyhow::{Context, Result};
use std::collections::HashMap;

use crate::arm_client::ArmClient;

pub struct RoleNameCache<'a> {
    client: &'a ArmClient,
    cache: HashMap<String, String>,
}

impl<'a> RoleNameCache<'a> {
    pub fn new(client: &'a ArmClient) -> Self {
        Self { client, cache: HashMap::new() }
    }

    pub async fn resolve(&mut self, role_definition_id: &str) -> String {
        if let Some(name) = self.cache.get(role_definition_id) {
            return name.clone();
        }
        let name = match self.client.get_role_definition_by_id(role_definition_id).await {
            Ok(def) => def
                .get("properties")
                .and_then(|p| p.get("roleName"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| last_segment(role_definition_id).to_string()),
            Err(_) => last_segment(role_definition_id).to_string(),
        };
        self.cache.insert(role_definition_id.to_string(), name.clone());
        name
    }
}

pub fn last_segment(id: &str) -> &str {
    id.rsplit('/').next().unwrap_or(id)
}

pub fn resolve_scope(client: &ArmClient, scope: Option<&str>, subscription: Option<&str>) -> String {
    if let Some(s) = scope {
        return s.to_string();
    }
    let sub = subscription.unwrap_or_else(|| client.subscription_id());
    format!("/subscriptions/{}", sub)
}

pub async fn list_eligible_projected(
    client: &ArmClient,
    scope: &str,
    principal_id: &str,
) -> Result<Vec<serde_json::Value>> {
    let raw = client.list_eligible_role_schedules(scope, principal_id).await?;
    let items = raw
        .get("value")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut cache = RoleNameCache::new(client);
    let mut out = Vec::with_capacity(items.len());
    for instance in items {
        let props = instance.get("properties").cloned().unwrap_or(serde_json::Value::Null);
        let role_def_id = props.get("roleDefinitionId").and_then(|v| v.as_str()).unwrap_or("");
        let role_name = cache.resolve(role_def_id).await;
        out.push(serde_json::json!({
            "roleName": role_name,
            "roleDefinitionId": role_def_id,
            "scope": props.get("scope").and_then(|v| v.as_str()).unwrap_or(scope),
            "eligibilityScheduleId": props.get("roleEligibilityScheduleId").and_then(|v| v.as_str()).unwrap_or(""),
            "startDateTime": props.get("startDateTime"),
            "endDateTime": props.get("endDateTime"),
            "memberType": props.get("memberType"),
            "principalId": props.get("principalId"),
        }));
    }
    Ok(out)
}

pub async fn list_active_projected(
    client: &ArmClient,
    scope: &str,
    principal_id: &str,
) -> Result<Vec<serde_json::Value>> {
    let raw = client.list_active_role_schedules(scope, principal_id).await?;
    let items = raw
        .get("value")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut cache = RoleNameCache::new(client);
    let mut out = Vec::with_capacity(items.len());
    for instance in items {
        let props = instance.get("properties").cloned().unwrap_or(serde_json::Value::Null);
        let role_def_id = props.get("roleDefinitionId").and_then(|v| v.as_str()).unwrap_or("");
        let role_name = cache.resolve(role_def_id).await;
        out.push(serde_json::json!({
            "roleName": role_name,
            "roleDefinitionId": role_def_id,
            "scope": props.get("scope").and_then(|v| v.as_str()).unwrap_or(scope),
            "assignmentType": props.get("assignmentType"),
            "startDateTime": props.get("startDateTime"),
            "endDateTime": props.get("endDateTime"),
            "memberType": props.get("memberType"),
            "principalId": props.get("principalId"),
            "linkedRoleEligibilityScheduleId": props.get("linkedRoleEligibilityScheduleId"),
        }));
    }
    Ok(out)
}

pub fn find_eligible_role<'a>(
    items: &'a [serde_json::Value],
    name: &str,
) -> Result<&'a serde_json::Value> {
    let needle = name.to_lowercase();
    let matches: Vec<&serde_json::Value> = items
        .iter()
        .filter(|r| {
            r.get("roleName")
                .and_then(|v| v.as_str())
                .map(|n| n.to_lowercase() == needle)
                .unwrap_or(false)
        })
        .collect();
    match matches.len() {
        0 => anyhow::bail!("no eligible role found matching {:?}", name),
        1 => Ok(matches[0]),
        _ => {
            let scopes: Vec<String> = matches
                .iter()
                .map(|m| m.get("scope").and_then(|v| v.as_str()).unwrap_or("?").to_string())
                .collect();
            anyhow::bail!(
                "multiple eligible roles found matching {:?} at scopes: {}\nUse --scope to specify which one",
                name,
                scopes.join(", ")
            )
        }
    }
}

pub fn find_active_role<'a>(
    items: &'a [serde_json::Value],
    name: &str,
) -> Result<&'a serde_json::Value> {
    let needle = name.to_lowercase();
    let matches: Vec<&serde_json::Value> = items
        .iter()
        .filter(|r| {
            r.get("roleName")
                .and_then(|v| v.as_str())
                .map(|n| n.to_lowercase() == needle)
                .unwrap_or(false)
        })
        .collect();
    match matches.len() {
        0 => anyhow::bail!("no active assignment found matching {:?}", name),
        1 => Ok(matches[0]),
        _ => {
            let scopes: Vec<String> = matches
                .iter()
                .map(|m| m.get("scope").and_then(|v| v.as_str()).unwrap_or("?").to_string())
                .collect();
            anyhow::bail!(
                "multiple active assignments found matching {:?} at scopes: {}\nUse --scope to specify which one",
                name,
                scopes.join(", ")
            )
        }
    }
}

pub fn str_field<'a>(v: &'a serde_json::Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(|x| x.as_str())
        .with_context(|| format!("missing or non-string field {key:?}"))
}
