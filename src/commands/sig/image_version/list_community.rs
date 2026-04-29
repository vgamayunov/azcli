use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, location: &str, public_gallery_name: &str, image_name: &str) -> Result<serde_json::Value> {
    let result = client.list_community_gallery_image_versions(location, public_gallery_name, image_name).await?;
    match result.get("value") {
        Some(v) => Ok(v.clone()),
        None => Ok(result),
    }
}
