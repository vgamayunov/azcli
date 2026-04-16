use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(client: &ArmClient, resource_group: Option<&str>, name: Option<&str>) -> Result<serde_json::Value> {
    let vms = client.list_vms(resource_group).await?;

    let vms_to_check: Vec<_> = match name {
        Some(n) => vms.into_iter().filter(|vm| vm.name == n).collect(),
        None => vms,
    };

    let mut results = Vec::new();

    for vm in &vms_to_check {
        let mut public_ips = Vec::new();
        let mut private_ips = Vec::new();

        if let Some(props) = &vm.properties {
            if let Some(net) = &props.network_profile {
                if let Some(nics) = &net.network_interfaces {
                    for nic_ref in nics {
                        if let Ok(nic) = client.get_network_interface(&nic_ref.id).await {
                            if let Some(ip_configs) = nic.pointer("/properties/ipConfigurations") {
                                if let Some(configs) = ip_configs.as_array() {
                                    for config in configs {
                                        if let Some(priv_ip) = config.pointer("/properties/privateIPAddress").and_then(|v| v.as_str()) {
                                            private_ips.push(serde_json::json!({
                                                "id": nic_ref.id,
                                                "privateIpAddress": priv_ip,
                                            }));
                                        }
                                        if let Some(pip_id) = config.pointer("/properties/publicIPAddress/id").and_then(|v| v.as_str()) {
                                            if let Ok(pip) = client.get_public_ip(pip_id).await {
                                                let ip_addr = pip.pointer("/properties/ipAddress")
                                                    .and_then(|v| v.as_str()).unwrap_or("");
                                                let fqdn = pip.pointer("/properties/dnsSettings/fqdn")
                                                    .and_then(|v| v.as_str()).unwrap_or("");
                                                public_ips.push(serde_json::json!({
                                                    "id": pip_id,
                                                    "ipAddress": ip_addr,
                                                    "ipAllocationMethod": pip.pointer("/properties/publicIPAllocationMethod").unwrap_or(&serde_json::Value::Null),
                                                    "name": pip.get("name").unwrap_or(&serde_json::Value::Null),
                                                    "fqdn": fqdn,
                                                }));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let rg = vm.id.as_deref()
            .and_then(|id| extract_rg(id))
            .unwrap_or_default();

        results.push(serde_json::json!({
            "virtualMachine": {
                "name": vm.name,
                "resourceGroup": rg,
                "network": {
                    "publicIpAddresses": public_ips,
                    "privateIpAddresses": private_ips,
                }
            }
        }));
    }

    Ok(serde_json::Value::Array(results))
}

fn extract_rg(id: &str) -> Option<String> {
    let lower = id.to_lowercase();
    let idx = lower.find("/resourcegroups/")?;
    let after = &id[idx + "/resourcegroups/".len()..];
    Some(after.split('/').next()?.to_string())
}
