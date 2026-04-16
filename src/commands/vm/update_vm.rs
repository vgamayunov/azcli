use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str, set_values: &[String]) -> Result<serde_json::Value> {
    let mut body = serde_json::json!({});

    for item in set_values {
        if let Some((path, value)) = item.split_once('=') {
            let parts: Vec<&str> = path.split('.').collect();
            let parsed_value = parse_value(value);
            set_nested(&mut body, &parts, parsed_value);
        }
    }

    client.vm_update(resource_group, name, body).await
}

fn parse_value(s: &str) -> serde_json::Value {
    if s == "true" {
        serde_json::Value::Bool(true)
    } else if s == "false" {
        serde_json::Value::Bool(false)
    } else if let Ok(n) = s.parse::<i64>() {
        serde_json::Value::Number(n.into())
    } else {
        serde_json::Value::String(s.to_string())
    }
}

fn set_nested(obj: &mut serde_json::Value, parts: &[&str], value: serde_json::Value) {
    if parts.is_empty() {
        return;
    }
    if parts.len() == 1 {
        obj[parts[0]] = value;
        return;
    }
    if obj.get(parts[0]).is_none() || !obj[parts[0]].is_object() {
        obj[parts[0]] = serde_json::json!({});
    }
    set_nested(&mut obj[parts[0]], &parts[1..], value);
}
