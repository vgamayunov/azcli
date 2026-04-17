use anyhow::Result;

use crate::arm_client::ArmClient;

pub async fn execute(
    client: &ArmClient,
    resource_group: &str,
    name: &str,
    port: &str,
    priority: i64,
    nsg_name: Option<&str>,
    apply_to_subnet: bool,
) -> Result<serde_json::Value> {
    let vm = client.show_vm(resource_group, name).await?;
    let vm_val = vm.to_flattened_value();

    let nic_ids: Vec<String> = vm_val.pointer("/networkProfile/networkInterfaces")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|n| n.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())).collect())
        .unwrap_or_default();

    if nic_ids.is_empty() {
        anyhow::bail!("VM '{name}' has no network interfaces");
    }

    let nic = client.get_network_interface(&nic_ids[0]).await?;

    let nsg_id = if apply_to_subnet {
        let subnet_id = nic.pointer("/properties/ipConfigurations")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|c| c.pointer("/properties/subnet/id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Could not find subnet for NIC"))?;

        let subnet = client.get_resource_by_id(subnet_id, "2023-11-01").await?;
        subnet.pointer("/properties/networkSecurityGroup/id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    } else {
        nic.pointer("/properties/networkSecurityGroup/id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    };

    let nsg_resource_id = match nsg_id {
        Some(id) => id,
        None => {
            let default_nsg_name = format!("{name}NSG");
            let nsg_n = nsg_name.unwrap_or(&default_nsg_name);
            let location = vm_val.get("location").and_then(|v| v.as_str()).unwrap_or("eastus");
            let new_nsg_id = format!(
                "/subscriptions/{sub}/resourceGroups/{rg}/providers/Microsoft.Network/networkSecurityGroups/{nsg}",
                sub = client.subscription_id(), rg = resource_group, nsg = nsg_n,
            );
            let nsg_body = serde_json::json!({
                "location": location,
                "properties": { "securityRules": [] },
            });
            client.put_resource_by_id(&new_nsg_id, "2023-11-01", nsg_body).await?;
            new_nsg_id
        }
    };

    let rule_name = format!("open-port-{port}");
    let rule_id = format!("{nsg_resource_id}/securityRules/{rule_name}");
    let rule_body = serde_json::json!({
        "properties": {
            "protocol": "*",
            "sourcePortRange": "*",
            "destinationPortRange": port,
            "sourceAddressPrefix": "*",
            "destinationAddressPrefix": "*",
            "access": "Allow",
            "priority": priority,
            "direction": "Inbound",
        }
    });

    let result = client.put_resource_by_id(&rule_id, "2023-11-01", rule_body).await?;
    Ok(result)
}
