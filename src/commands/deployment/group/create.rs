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

    let body = serde_json::json!({ "properties": properties });
    let base = client.deployment_base_url_group(resource_group);
    client.deployment_create(&base, name, body).await
}
