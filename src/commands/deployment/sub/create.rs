use anyhow::Result;
use crate::arm_client::ArmClient;
use crate::commands::deployment::group::validate::{load_template, load_parameters};

pub async fn execute(
    client: &ArmClient,
    name: &str,
    location: &str,
    template_file: Option<&str>,
    template_uri: Option<&str>,
    parameters: Option<&str>,
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

    let body = serde_json::json!({ "location": location, "properties": properties });
    let base = client.deployment_base_url_sub();
    client.deployment_create(&base, name, body).await
}
