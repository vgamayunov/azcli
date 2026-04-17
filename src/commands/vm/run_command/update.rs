use anyhow::{Result, bail};

use crate::arm_client::ArmClient;
use super::parse_params;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    vm_name: &str,
    name: &str,
    script: Option<&str>,
    script_uri: Option<&str>,
    command_id: Option<&str>,
    parameters: &[String],
    protected_parameters: &[String],
    run_as_user: Option<&str>,
    run_as_password: Option<&str>,
    timeout_in_seconds: Option<i64>,
    output_blob_uri: Option<&str>,
    error_blob_uri: Option<&str>,
) -> Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();

    let source_count = [script.is_some(), script_uri.is_some(), command_id.is_some()]
        .iter().filter(|b| **b).count();
    if source_count > 1 {
        bail!("specify at most one of --script, --script-uri, --command-id");
    }
    if source_count == 1 {
        let mut source = serde_json::json!({});
        if let Some(s) = script { source["script"] = serde_json::json!(s); }
        if let Some(u) = script_uri { source["scriptUri"] = serde_json::json!(u); }
        if let Some(c) = command_id { source["commandId"] = serde_json::json!(c); }
        properties.insert("source".into(), source);
    }

    let params = parse_params(parameters);
    if !params.is_empty() {
        properties.insert("parameters".into(), serde_json::json!(params));
    }
    let protected = parse_params(protected_parameters);
    if !protected.is_empty() {
        properties.insert("protectedParameters".into(), serde_json::json!(protected));
    }
    if let Some(u) = run_as_user { properties.insert("runAsUser".into(), serde_json::json!(u)); }
    if let Some(p) = run_as_password { properties.insert("runAsPassword".into(), serde_json::json!(p)); }
    if let Some(t) = timeout_in_seconds { properties.insert("timeoutInSeconds".into(), serde_json::json!(t)); }
    if let Some(u) = output_blob_uri { properties.insert("outputBlobUri".into(), serde_json::json!(u)); }
    if let Some(u) = error_blob_uri { properties.insert("errorBlobUri".into(), serde_json::json!(u)); }

    if properties.is_empty() {
        bail!("no updates specified");
    }

    let body = serde_json::json!({ "properties": properties });
    client.update_vm_run_command(resource_group, vm_name, name, body).await
}
