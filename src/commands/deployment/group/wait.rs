use anyhow::Result;
use tracing::info;

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    name: &str,
    created: bool,
    updated: bool,
    deleted: bool,
    exists: bool,
    interval: u64,
    timeout: u64,
) -> Result<()> {
    let target = if created || updated {
        "Succeeded"
    } else if deleted {
        "__deleted__"
    } else if exists {
        "__exists__"
    } else {
        anyhow::bail!("Specify one of --created, --updated, --deleted, or --exists");
    };

    let start = std::time::Instant::now();
    loop {
        if start.elapsed().as_secs() >= timeout {
            anyhow::bail!("Timed out waiting for deployment '{name}'");
        }

        match client.show_deployment(resource_group, name).await {
            Ok(deployment) => {
                if target == "__deleted__" {
                    info!("Deployment '{name}' still exists, waiting...");
                } else if target == "__exists__" {
                    return Ok(());
                } else {
                    let state = deployment
                        .pointer("/properties/provisioningState")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if state == target {
                        return Ok(());
                    }
                    info!("Deployment '{name}' state: {state}, waiting for {target}...");
                }
            }
            Err(_) if target == "__deleted__" => return Ok(()),
            Err(e) => return Err(e),
        }

        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
    }
}
