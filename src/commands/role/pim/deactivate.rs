use anyhow::Result;
use uuid::Uuid;

use crate::arm_client::ArmClient;
use crate::commands::role::pim::{
    find_active_role, list_active_projected, resolve_scope, str_field,
};

pub async fn execute(
    client: &ArmClient,
    role: &str,
    scope: Option<&str>,
    subscription: Option<&str>,
) -> Result<serde_json::Value> {
    let scope = resolve_scope(client, scope, subscription);
    let principal_id = client.principal_id()?;
    let active = list_active_projected(client, &scope, &principal_id).await?;
    let role_entry = find_active_role(&active, role)?;

    let role_definition_id = str_field(role_entry, "roleDefinitionId")?;
    let role_scope = str_field(role_entry, "scope")?;

    let request_name = Uuid::new_v4().to_string();

    let body = serde_json::json!({
        "properties": {
            "principalId": principal_id,
            "roleDefinitionId": role_definition_id,
            "requestType": "SelfDeactivate",
        }
    });

    client
        .create_role_assignment_schedule_request(role_scope, &request_name, body)
        .await
}
