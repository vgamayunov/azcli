use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    name: &str,
    maximum_duration: &str,
    reboot_setting: &str,
    classifications_linux: Option<&[String]>,
    classifications_win: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut body = serde_json::json!({
        "maximumDuration": maximum_duration,
        "rebootSetting": reboot_setting,
    });

    if let Some(cls) = classifications_linux {
        body["linuxParameters"] = serde_json::json!({
            "classificationsToInclude": cls,
        });
    }
    if let Some(cls) = classifications_win {
        body["windowsParameters"] = serde_json::json!({
            "classificationsToInclude": cls,
        });
    }

    client.vm_install_patches(resource_group, name, body).await
}
