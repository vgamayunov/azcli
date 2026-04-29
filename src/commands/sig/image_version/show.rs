use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, gallery_name: &str, image_name: &str, version: &str) -> Result<serde_json::Value> {
    client.show_gallery_image_version(resource_group, gallery_name, image_name, version).await
}
