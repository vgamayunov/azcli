use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    name: &str,
    time: Option<&str>,
    off: bool,
    email: Option<&str>,
    webhook: Option<&str>,
    location: &str,
) -> Result<serde_json::Value> {
    if off {
        client.vm_auto_shutdown_delete(resource_group, name).await?;
        eprintln!("Auto-shutdown disabled for VM '{name}'.");
        return Ok(serde_json::json!({"status": "disabled"}));
    }

    let time_of_day = time.unwrap_or("1900");

    let mut notification_settings = serde_json::json!({
        "status": "Disabled",
        "timeInMinutes": 30,
    });

    if email.is_some() || webhook.is_some() {
        notification_settings["status"] = serde_json::json!("Enabled");
        if let Some(e) = email {
            notification_settings["emailRecipient"] = serde_json::json!(e);
        }
        if let Some(w) = webhook {
            notification_settings["webhookUrl"] = serde_json::json!(w);
        }
    }

    let body = serde_json::json!({
        "location": location,
        "properties": {
            "status": "Enabled",
            "taskType": "ComputeVmShutdownTask",
            "dailyRecurrence": { "time": time_of_day },
            "timeZoneId": "UTC",
            "targetResourceId": format!(
                "/subscriptions/{sub}/resourceGroups/{rg}/providers/Microsoft.Compute/virtualMachines/{name}",
                sub = "", rg = resource_group, name = name,
            ),
            "notificationSettings": notification_settings,
        }
    });

    client.vm_auto_shutdown(resource_group, name, body).await
}
