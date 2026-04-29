use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, location: &str, gallery_unique_name: &str) -> Result<serde_json::Value> {
    client.show_shared_gallery(location, gallery_unique_name).await
}
