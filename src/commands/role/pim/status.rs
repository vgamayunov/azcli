use anyhow::Result;

use crate::arm_client::ArmClient;
use crate::commands::role::pim::{list_active_projected, resolve_scope};

pub async fn execute(
    client: &ArmClient,
    scope: Option<&str>,
    subscription: Option<&str>,
) -> Result<serde_json::Value> {
    let scope = resolve_scope(client, scope, subscription);
    let principal_id = client.principal_id()?;
    let active = list_active_projected(client, &scope, &principal_id).await?;
    Ok(serde_json::json!({
        "scope": scope,
        "principalId": principal_id,
        "active": active,
    }))
}
