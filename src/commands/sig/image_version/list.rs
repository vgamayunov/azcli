use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, gallery_name: &str, image_name: &str) -> Result<serde_json::Value> {
    let result = client.list_gallery_image_versions(resource_group, gallery_name, image_name).await?;
    match result.get("value") {
        Some(v) => Ok(v.clone()),
        None => Ok(result),
    }
}
