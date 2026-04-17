use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    location: Option<&str>,
    zone: bool,
) -> Result<serde_json::Value> {
    let result = client.list_disk_skus().await?;
    let items = match result.get("value").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return Ok(result),
    };

    let filtered: Vec<_> = items.into_iter().filter(|item| {
        if let Some(loc) = location {
            let locs = item.get("locations").and_then(|v| v.as_array());
            match locs {
                Some(locs) if locs.iter().any(|l| l.as_str().map(|s| s.eq_ignore_ascii_case(loc)).unwrap_or(false)) => {}
                _ => return false,
            }
        }
        if zone {
            let zone_details = item.get("locationInfo")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|li| li.get("zones"))
                .and_then(|v| v.as_array());
            match zone_details {
                Some(zones) if !zones.is_empty() => {}
                _ => return false,
            }
        }
        true
    }).collect();

    Ok(serde_json::Value::Array(filtered))
}
