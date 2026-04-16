use anyhow::Result;

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
    let start = std::time::Instant::now();

    loop {
        if start.elapsed().as_secs() > timeout {
            anyhow::bail!("Wait timed out after {timeout}s");
        }

        let result = client.show_vm(resource_group, name).await;

        match &result {
            Ok(vm) => {
                let state = vm.to_flattened_value();
                let prov_state = state.get("provisioningState")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if exists {
                    return Ok(());
                }
                if created && prov_state == "Succeeded" {
                    return Ok(());
                }
                if updated && prov_state == "Succeeded" {
                    return Ok(());
                }
            }
            Err(_) => {
                if deleted {
                    return Ok(());
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
    }
}
