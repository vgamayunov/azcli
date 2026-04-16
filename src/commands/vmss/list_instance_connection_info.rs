use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let instances = client.list_vmss_instances(resource_group, name, Some("instanceView")).await?;

    let mut connections = Vec::new();
    for inst in &instances {
        let instance_id = inst.instance_id.as_deref().unwrap_or("?");
        let name = inst.name.as_deref().unwrap_or("?");

        let mut ips = Vec::new();
        if let Some(props) = &inst.properties {
            if let Some(net) = &props.network_profile {
                if let Some(nics) = &net.network_interfaces {
                    for nic in nics {
                        ips.push(nic.id.clone());
                    }
                }
            }
        }

        connections.push(serde_json::json!({
            "instanceId": instance_id,
            "name": name,
            "networkInterfaces": ips,
        }));
    }

    Ok(serde_json::Value::Array(connections))
}
