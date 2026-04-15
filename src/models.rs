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
