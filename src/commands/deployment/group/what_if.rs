use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    name: &str,
    template_file: Option<&str>,
    template_uri: Option<&str>,
    parameters: Option<&str>,
    mode: &str,
    result_format: Option<&str>,
) -> Result<serde_json::Value> {
    let template = super::validate::load_template(template_file, template_uri)?;
    let params = super::validate::load_parameters(parameters)?;

    let mut properties = serde_json::json!({
        "mode": mode,
        "template": template,
    });
    if let Some(p) = params {
        properties["parameters"] = p;
    }
    if let Some(rf) = result_format {
        properties["whatIfSettings"] = serde_json::json!({ "resultFormat": rf });
    }

    let body = serde_json::json!({ "properties": properties });
    client.what_if_deployment(resource_group, name, body).await
}
