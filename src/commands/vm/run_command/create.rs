use anyhow::{Context, Result, bail};

use crate::arm_client::ArmClient;
use super::parse_params;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    vm_name: &str,
    name: &str,
    location: Option<&str>,
    script: Option<&str>,
    script_uri: Option<&str>,
    command_id: Option<&str>,
    parameters: &[String],
    protected_parameters: &[String],
    run_as_user: Option<&str>,
    run_as_password: Option<&str>,
    async_execution: bool,
    timeout_in_seconds: Option<i64>,
    output_blob_uri: Option<&str>,
    error_blob_uri: Option<&str>,
) -> Result<serde_json::Value> {
    let loc = match location {
        Some(l) => l.to_string(),
        None => {
            let vm = client.show_vm(resource_group, vm_name).await?;
            vm.to_flattened_value().get("location").and_then(|v| v.as_str())
                .context("could not determine VM location")?.to_string()
        }
    };

    let mut source = serde_json::json!({});
    match (script, script_uri, command_id) {
        (Some(s), None, None) => { source["script"] = serde_json::json!(s); }
        (None, Some(u), None) => { source["scriptUri"] = serde_json::json!(u); }
        (None, None, Some(c)) => { source["commandId"] = serde_json::json!(c); }
        _ => bail!("specify exactly one of --script, --script-uri, --command-id"),
    }

    let mut properties = serde_json::json!({
        "source": source,
        "asyncExecution": async_execution,
    });

    let params = parse_params(parameters);
    if !params.is_empty() {
        properties["parameters"] = serde_json::json!(params);
    }
    let protected = parse_params(protected_parameters);
    if !protected.is_empty() {
        properties["protectedParameters"] = serde_json::json!(protected);
    }
    if let Some(u) = run_as_user { properties["runAsUser"] = serde_json::json!(u); }
    if let Some(p) = run_as_password { properties["runAsPassword"] = serde_json::json!(p); }
    if let Some(t) = timeout_in_seconds { properties["timeoutInSeconds"] = serde_json::json!(t); }
    if let Some(u) = output_blob_uri { properties["outputBlobUri"] = serde_json::json!(u); }
    if let Some(u) = error_blob_uri { properties["errorBlobUri"] = serde_json::json!(u); }

    let body = serde_json::json!({ "location": loc, "properties": properties });
    client.create_vm_run_command(resource_group, vm_name, name, body).await
}
