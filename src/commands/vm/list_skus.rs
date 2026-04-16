use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    location: Option<&str>,
    resource_type: Option<&str>,
    size: Option<&str>,
    zone: bool,
) -> Result<serde_json::Value> {
    let result = client.vm_list_skus().await?;
    let items = match result.get("value").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return Ok(result),
    };

    let filtered: Vec<_> = items.into_iter().filter(|item| {
        if let Some(loc) = location {
            let locs = item.get("locations").and_then(|v| v.as_array());
            if let Some(locs) = locs {
                if !locs.iter().any(|l| l.as_str().map(|s| s.eq_ignore_ascii_case(loc)).unwrap_or(false)) {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Some(rt) = resource_type {
            if let Some(item_rt) = item.get("resourceType").and_then(|v| v.as_str()) {
                if !item_rt.eq_ignore_ascii_case(rt) {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Some(sz) = size {
            if let Some(item_name) = item.get("name").and_then(|v| v.as_str()) {
                if !item_name.to_lowercase().contains(&sz.to_lowercase()) {
                    return false;
                }
            } else {
                return false;
            }
        }
        if zone {
            let zone_details = item.get("locationInfo")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|li| li.get("zones"))
                .and_then(|v| v.as_array());
            if let Some(zones) = zone_details {
                if zones.is_empty() {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }).collect();

    Ok(serde_json::Value::Array(filtered))
}
