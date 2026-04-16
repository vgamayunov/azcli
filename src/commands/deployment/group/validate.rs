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
    let template = load_template(template_file, template_uri)?;
    let params = load_parameters(parameters)?;

    let mut properties = serde_json::json!({
        "mode": mode,
        "template": template,
    });
    if let Some(p) = params {
        properties["parameters"] = p;
    }

    let body = serde_json::json!({ "properties": properties });
    client.validate_deployment(resource_group, name, body).await
}

pub fn load_template(file: Option<&str>, uri: Option<&str>) -> Result<serde_json::Value> {
    if let Some(path) = file {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read template file '{}': {}", path, e))?;
        if path.ends_with(".bicep") {
            anyhow::bail!("Bicep files must be compiled to ARM JSON first (use `az bicep build`)");
        }
        serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse template JSON: {}", e))
    } else if let Some(uri) = uri {
        Ok(serde_json::json!({ "$schema": "uri-ref", "templateLink": { "uri": uri } }))
    } else {
        anyhow::bail!("Specify --template-file or --template-uri")
    }
}

pub fn load_parameters(params: Option<&str>) -> Result<Option<serde_json::Value>> {
    match params {
        None => Ok(None),
        Some(s) if s.starts_with('@') => {
            let path = &s[1..];
            let content = std::fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("Failed to read parameters file '{}': {}", path, e))?;
            let parsed: serde_json::Value = serde_json::from_str(&content)?;
            if let Some(p) = parsed.get("parameters") {
                Ok(Some(p.clone()))
            } else {
                Ok(Some(parsed))
            }
        }
        Some(s) => {
            let parsed: serde_json::Value = serde_json::from_str(s)?;
            Ok(Some(parsed))
        }
    }
}
