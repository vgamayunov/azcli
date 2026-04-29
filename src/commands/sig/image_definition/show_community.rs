use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, location: &str, public_gallery_name: &str, image_name: &str) -> Result<serde_json::Value> {
    client.show_community_gallery_image_definition(location, public_gallery_name, image_name).await
}
