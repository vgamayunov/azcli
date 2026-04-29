use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    name: &str,
    output_name: Option<&str>,
) -> Result<serde_json::Value> {
    if let Some(output) = output_name {
        return client
            .show_image_template_run_output(resource_group, name, output)
            .await;
    }
    let result = client
        .list_image_template_run_outputs(resource_group, name)
        .await?;
    match result.get("value") {
        Some(value) => Ok(value.clone()),
        None => Ok(result),
    }
}
