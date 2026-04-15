use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
pub enum BastionSku {
    Basic,
    Standard,
    Developer,
    QuickConnect,
    Premium,
}

impl fmt::Display for BastionSku {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Basic => write!(f, "Basic"),
            Self::Standard => write!(f, "Standard"),
            Self::Developer => write!(f, "Developer"),
            Self::QuickConnect => write!(f, "QuickConnect"),
            Self::Premium => write!(f, "Premium"),
        }
    }
}

impl BastionSku {
    pub fn is_standard_or_higher(&self) -> bool {
        matches!(self, Self::Standard | Self::Premium)
    }

    pub fn supports_native_client(&self) -> bool {
        matches!(self, Self::Developer | Self::Standard | Self::Premium)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BastionHost {
    pub id: Option<String>,
    pub name: String,
    pub location: Option<String>,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub properties: Option<BastionHostProperties>,
    pub sku: Option<BastionSkuInfo>,
    pub tags: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BastionHostProperties {
    pub dns_name: Option<String>,
    pub provisioning_state: Option<String>,
    pub enable_tunneling: Option<bool>,
    pub enable_ip_connect: Option<bool>,
    pub ip_configurations: Option<Vec<BastionIpConfiguration>>,
    pub virtual_network: Option<SubResource>,
    pub network_acls: Option<Vec<NetworkAcl>>,
    pub enable_session_recording: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BastionSkuInfo {
    pub name: BastionSku,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BastionIpConfiguration {
    pub name: String,
    pub properties: Option<BastionIpConfigProperties>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BastionIpConfigProperties {
    pub subnet: Option<SubResource>,
    pub public_ip_address: Option<SubResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubResource {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkAcl {
    pub address_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
    pub auth_token: String,
    pub node_id: String,
    pub websocket_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureListResponse<T> {
    pub value: Vec<T>,
    #[serde(rename = "nextLink")]
    pub next_link: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum AuthType {
    Password,
    #[value(name = "ssh-key")]
    SshKey,
    #[value(name = "AAD")]
    Aad,
}

impl fmt::Display for AuthType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Password => write!(f, "password"),
            Self::SshKey => write!(f, "ssh-key"),
            Self::Aad => write!(f, "AAD"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Json,
    Jsonc,
    Table,
    Tsv,
    Yaml,
    Yamlc,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGroup {
    pub id: Option<String>,
    pub name: String,
    pub location: String,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub properties: Option<ResourceGroupProperties>,
    pub tags: Option<HashMap<String, String>>,
    #[serde(rename = "managedBy")]
    pub managed_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceGroupProperties {
    pub provisioning_state: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualMachine {
    pub id: Option<String>,
    pub name: String,
    pub location: Option<String>,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub zones: Option<Vec<String>>,
    pub tags: Option<HashMap<String, String>>,
    pub identity: Option<serde_json::Value>,
    pub plan: Option<serde_json::Value>,
    pub properties: Option<VmProperties>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmProperties {
    pub provisioning_state: Option<String>,
    pub vm_id: Option<String>,
    pub hardware_profile: Option<VmHardwareProfile>,
    pub storage_profile: Option<VmStorageProfile>,
    pub os_profile: Option<VmOsProfile>,
    pub network_profile: Option<VmNetworkProfile>,
    pub priority: Option<String>,
    pub time_created: Option<String>,
    pub instance_view: Option<VmInstanceView>,
}

impl VirtualMachine {
    pub fn to_flattened_value(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();

        if let Some(id) = &self.id {
            map.insert("id".into(), serde_json::Value::String(id.clone()));
        }
        map.insert("name".into(), serde_json::Value::String(self.name.clone()));
        if let Some(loc) = &self.location {
            map.insert("location".into(), serde_json::Value::String(loc.clone()));
        }
        if let Some(t) = &self.resource_type {
            map.insert("type".into(), serde_json::Value::String(t.clone()));
        }
        if let Some(zones) = &self.zones {
            map.insert(
                "zones".into(),
                serde_json::to_value(zones).unwrap_or_default(),
            );
        }
        if let Some(tags) = &self.tags {
            map.insert(
                "tags".into(),
                serde_json::to_value(tags).unwrap_or_default(),
            );
        }
        if let Some(identity) = &self.identity {
            map.insert("identity".into(), identity.clone());
        }
        if let Some(plan) = &self.plan {
            map.insert("plan".into(), plan.clone());
        }

        if let Some(props) = &self.properties {
            if let Ok(serde_json::Value::Object(props_map)) = serde_json::to_value(props) {
                for (k, v) in props_map {
                    map.insert(k, v);
                }
            }
        }

        if let Some(id) = &self.id {
            if let Some(rg) = extract_resource_group(id) {
                map.insert("resourceGroup".into(), serde_json::Value::String(rg));
            }
        }

        serde_json::Value::Object(map)
    }
}

fn extract_resource_group(id: &str) -> Option<String> {
    let lower = id.to_lowercase();
    let idx = lower.find("/resourcegroups/")?;
    let after = &id[idx + "/resourcegroups/".len()..];
    Some(after.split('/').next()?.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmHardwareProfile {
    pub vm_size: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmStorageProfile {
    pub image_reference: Option<serde_json::Value>,
    pub os_disk: Option<serde_json::Value>,
    pub data_disks: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmOsProfile {
    pub computer_name: Option<String>,
    pub admin_username: Option<String>,
    pub linux_configuration: Option<serde_json::Value>,
    pub windows_configuration: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmNetworkProfile {
    pub network_interfaces: Option<Vec<SubResource>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmInstanceView {
    pub statuses: Option<Vec<VmInstanceViewStatus>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmInstanceViewStatus {
    pub code: Option<String>,
    pub display_status: Option<String>,
    pub level: Option<String>,
    pub time: Option<String>,
}
