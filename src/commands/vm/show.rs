use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    name: &str,
    show_details: bool,
) -> Result<serde_json::Value> {
    let vm = client.show_vm(resource_group, name).await?;
    let mut value = vm.to_flattened_value();

    if !show_details {
        return Ok(value);
    }

    let mut power_state: Option<String> = None;
    if let Ok(iv) = client.vm_get_instance_view(resource_group, name).await {
        if let Some(statuses) = iv.pointer("/statuses").and_then(|v| v.as_array()) {
            for s in statuses {
                if let Some(code) = s.get("code").and_then(|v| v.as_str()) {
                    if let Some(rest) = code.strip_prefix("PowerState/") {
                        power_state = Some(rest.to_string());
                        break;
                    }
                }
            }
        }
        if let Some(obj) = value.as_object_mut() {
            obj.insert("instanceView".to_string(), iv);
        }
    }

    let mut public_ips: Vec<String> = Vec::new();
    let mut private_ips: Vec<String> = Vec::new();
    let mut fqdns: Vec<String> = Vec::new();

    if let Some(props) = &vm.properties {
        if let Some(net) = &props.network_profile {
            if let Some(nics) = &net.network_interfaces {
                for nic_ref in nics {
                    let Ok(nic) = client.get_network_interface(&nic_ref.id).await else {
                        continue;
                    };
                    let Some(configs) = nic
                        .pointer("/properties/ipConfigurations")
                        .and_then(|v| v.as_array())
                    else {
                        continue;
                    };
                    for cfg in configs {
                        if let Some(p) = cfg
                            .pointer("/properties/privateIPAddress")
                            .and_then(|v| v.as_str())
                        {
                            private_ips.push(p.to_string());
                        }
                        if let Some(pip_id) = cfg
                            .pointer("/properties/publicIPAddress/id")
                            .and_then(|v| v.as_str())
                        {
                            if let Ok(pip) = client.get_public_ip(pip_id).await {
                                if let Some(ip) = pip
                                    .pointer("/properties/ipAddress")
                                    .and_then(|v| v.as_str())
                                {
                                    public_ips.push(ip.to_string());
                                }
                                if let Some(fqdn) = pip
                                    .pointer("/properties/dnsSettings/fqdn")
                                    .and_then(|v| v.as_str())
                                {
                                    fqdns.push(fqdn.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(obj) = value.as_object_mut() {
        if let Some(ps) = power_state {
            obj.insert("powerState".to_string(), serde_json::Value::String(ps));
        }
        obj.insert(
            "publicIps".to_string(),
            serde_json::Value::String(public_ips.join(",")),
        );
        obj.insert(
            "privateIps".to_string(),
            serde_json::Value::String(private_ips.join(",")),
        );
        obj.insert(
            "fqdns".to_string(),
            serde_json::Value::String(fqdns.join(",")),
        );
    }

    Ok(value)
}
