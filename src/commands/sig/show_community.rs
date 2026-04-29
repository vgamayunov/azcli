use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, location: &str, public_gallery_name: &str) -> Result<serde_json::Value> {
    client.show_community_gallery(location, public_gallery_name).await
}
