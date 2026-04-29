use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, location: &str, gallery_unique_name: &str, image_name: &str, version: &str) -> Result<serde_json::Value> {
    client.show_shared_gallery_image_version(location, gallery_unique_name, image_name, version).await
}
