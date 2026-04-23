use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use crate::arm_client::ArmClient;
use crate::commands::role::pim::{
    find_eligible_role, list_eligible_projected, resolve_scope, str_field,
};

pub async fn execute(
    client: &ArmClient,
    role: &str,
    justification: &str,
    duration: &str,
    scope: Option<&str>,
    subscription: Option<&str>,
) -> Result<serde_json::Value> {
    let scope = resolve_scope(client, scope, subscription);
    let principal_id = client.principal_id()?;
    let eligible = list_eligible_projected(client, &scope, &principal_id).await?;
    let role_entry = find_eligible_role(&eligible, role)?;

    let role_definition_id = str_field(role_entry, "roleDefinitionId")?;
    let eligibility_schedule_id = str_field(role_entry, "eligibilityScheduleId")?;
    let role_scope = str_field(role_entry, "scope")?;

    let request_name = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let body = serde_json::json!({
        "properties": {
            "principalId": principal_id,
            "roleDefinitionId": role_definition_id,
            "requestType": "SelfActivate",
            "linkedRoleEligibilityScheduleId": eligibility_schedule_id,
            "justification": justification,
            "scheduleInfo": {
                "startDateTime": now,
                "expiration": {
                    "type": "AfterDuration",
                    "endDateTime": null,
                    "duration": duration,
                }
            }
        }
    });

    client
        .create_role_assignment_schedule_request(role_scope, &request_name, body)
        .await
}
