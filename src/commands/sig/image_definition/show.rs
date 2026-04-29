use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, gallery_name: &str, image_name: &str) -> Result<serde_json::Value> {
    client.show_gallery_image_definition(resource_group, gallery_name, image_name).await
}
