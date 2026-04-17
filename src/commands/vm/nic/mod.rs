pub mod list;
pub mod show;
pub mod add;
pub mod remove;
pub mod set;

use anyhow::{Result, anyhow};

pub fn resolve_nic_id(subscription_id: &str, resource_group: &str, nic: &str) -> String {
    if nic.starts_with('/') {
        nic.to_string()
    } else {
        format!(
            "/subscriptions/{subscription_id}/resourceGroups/{resource_group}/providers/Microsoft.Network/networkInterfaces/{nic}"
        )
    }
}

pub fn nic_name_from_id(id: &str) -> &str {
    id.rsplit('/').next().unwrap_or(id)
}

/// Ensure exactly one NIC in the array is marked primary.
///
/// If `primary_nic` is provided, that NIC (by name match against the ID suffix)
/// becomes primary and all others are set to `primary: false`. Returns an error
/// if `primary_nic` is specified but no entry matches.
///
/// If `primary_nic` is not provided and no NIC already has `primary: true`,
/// the first entry is marked primary.
pub fn apply_primary(nics: &mut [serde_json::Value], primary_nic: Option<&str>) -> Result<()> {
    if nics.is_empty() {
        return Ok(());
    }

    if let Some(primary) = primary_nic {
        let primary_lower = primary.to_lowercase();
        let mut found = false;
        for n in nics.iter_mut() {
            let id = n.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let name = nic_name_from_id(id).to_lowercase();
            let is_match = name == primary_lower || id.to_lowercase() == primary_lower;
            n["primary"] = serde_json::json!(is_match);
            if is_match {
                found = true;
            }
        }
        if !found {
            return Err(anyhow!(
                "--primary-nic '{primary}' did not match any NIC in the resulting set"
            ));
        }
    } else {
        let any_primary = nics.iter().any(|n| {
            n.get("primary").and_then(|v| v.as_bool()).unwrap_or(false)
        });
        if !any_primary {
            nics[0]["primary"] = serde_json::json!(true);
            for n in nics.iter_mut().skip(1) {
                if n.get("primary").is_none() {
                    n["primary"] = serde_json::json!(false);
                }
            }
        }
    }
    Ok(())
}
