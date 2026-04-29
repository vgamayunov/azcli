use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, location: &str, gallery_unique_name: &str) -> Result<serde_json::Value> {
    let result = client.list_shared_gallery_image_definitions(location, gallery_unique_name).await?;
    match result.get("value") {
        Some(v) => Ok(v.clone()),
        None => Ok(result),
    }
}
