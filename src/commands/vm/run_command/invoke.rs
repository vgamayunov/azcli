use anyhow::Result;

use crate::arm_client::ArmClient;
use super::parse_params;

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    vm_name: &str,
    command_id: &str,
    scripts: &[String],
    parameters: &[String],
) -> Result<serde_json::Value> {
    let mut body = serde_json::json!({ "commandId": command_id });
    if !scripts.is_empty() {
        body["script"] = serde_json::json!(scripts);
    }
    let params = parse_params(parameters);
    if !params.is_empty() {
        body["parameters"] = serde_json::json!(params);
    }
    client.vm_run_command_invoke(resource_group, vm_name, body).await
}
