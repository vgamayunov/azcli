use anyhow::Result;
use crate::arm_client::ArmClient;
use crate::commands::deployment::group::validate::{load_template, load_parameters};

pub async fn execute(
    client: &ArmClient,
    management_group_id: &str,
    name: &str,
    location: &str,
    template_file: Option<&str>,
    template_uri: Option<&str>,
    parameters: Option<&str>,
    result_format: Option<&str>,
) -> Result<serde_json::Value> {
    let template = load_template(template_file, template_uri)?;
    let params = load_parameters(parameters)?;

    let mut properties = serde_json::json!({
        "template": template,
        "mode": "Incremental",
    });
    if let Some(p) = params {
        properties["parameters"] = p;
    }
    if let Some(rf) = result_format {
        properties["whatIfSettings"] = serde_json::json!({ "resultFormat": rf });
    }

    let body = serde_json::json!({ "location": location, "properties": properties });
    let base = ArmClient::deployment_base_url_mg(management_group_id);
    client.deployment_what_if(&base, name, body).await
}
