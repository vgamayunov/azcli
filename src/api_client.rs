use anyhow::{Context, Result};
use reqwest::Client;
use tracing::debug;

use crate::models::*;

const API_VERSION: &str = "2024-01-01";

#[derive(Clone)]
pub struct BastionClient {
    client: Client,
    subscription_id: String,
    access_token: String,
}

impl BastionClient {
    pub async fn new() -> Result<Self> {
        let access_token = crate::auth::get_access_token_az_cli().await?;
        let subscription_id = crate::auth::get_subscription_id_az_cli().await?;

        Ok(Self {
            client: Client::new(),
            subscription_id,
            access_token,
        })
    }

    pub fn with_token(access_token: String, subscription_id: String) -> Self {
        Self {
            client: Client::new(),
            subscription_id,
            access_token,
        }
    }

    pub fn subscription_id(&self) -> &str {
        &self.subscription_id
    }

    fn bastion_url(&self, resource_group: &str, bastion_name: &str) -> String {
        format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/bastionHosts/{}?api-version={}",
            self.subscription_id, resource_group, bastion_name, API_VERSION
        )
    }

    fn bastion_list_url(&self, resource_group: Option<&str>) -> String {
        match resource_group {
            Some(rg) => format!(
                "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/bastionHosts?api-version={}",
                self.subscription_id, rg, API_VERSION
            ),
            None => format!(
                "https://management.azure.com/subscriptions/{}/providers/Microsoft.Network/bastionHosts?api-version={}",
                self.subscription_id, API_VERSION
            ),
        }
    }

    pub async fn show(&self, resource_group: &str, bastion_name: &str) -> Result<BastionHost> {
        let url = self.bastion_url(resource_group, bastion_name);
        debug!("GET {url}");

        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .context("Failed to GET bastion host")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("GET bastion host failed ({status}): {body}");
        }

        resp.json().await.context("Failed to parse bastion host response")
    }

    pub async fn list(&self, resource_group: Option<&str>) -> Result<Vec<BastionHost>> {
        let url = self.bastion_list_url(resource_group);
        debug!("GET {url}");

        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .context("Failed to list bastion hosts")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("List bastion hosts failed ({status}): {body}");
        }

        let list: AzureListResponse<BastionHost> = resp.json().await.context("Failed to parse list response")?;
        Ok(list.value)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        resource_group: &str,
        bastion_name: &str,
        location: &str,
        vnet_name: &str,
        public_ip: Option<&str>,
        sku: BastionSku,
        enable_tunneling: bool,
        enable_ip_connect: bool,
        file_copy: bool,
        disable_copy_paste: bool,
        kerberos: bool,
        session_recording: bool,
        shareable_link: bool,
        network_acls_ips: Option<&[String]>,
        zones: Option<&[String]>,
        tags: Option<std::collections::HashMap<String, String>>,
    ) -> Result<BastionHost> {
        let url = self.bastion_url(resource_group, bastion_name);
        debug!("PUT {url}");

        let vnet_id = if vnet_name.starts_with('/') {
            vnet_name.to_string()
        } else {
            format!(
                "/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/virtualNetworks/{}",
                self.subscription_id, resource_group, vnet_name
            )
        };

        let subnet_id = format!("{vnet_id}/subnets/AzureBastionSubnet");

        let mut ip_config_props = serde_json::json!({
            "subnet": { "id": subnet_id }
        });

        if let Some(pip) = public_ip {
            let pip_id = if pip.starts_with('/') {
                pip.to_string()
            } else {
                format!(
                    "/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/publicIPAddresses/{}",
                    self.subscription_id, resource_group, pip
                )
            };
            ip_config_props["publicIPAddress"] = serde_json::json!({ "id": pip_id });
        }

        let mut properties = serde_json::json!({
            "ipConfigurations": [{
                "name": "bastion_ip_config",
                "properties": ip_config_props
            }],
            "virtualNetwork": { "id": vnet_id }
        });

        if enable_tunneling {
            properties["enableTunneling"] = serde_json::json!(true);
        }
        if enable_ip_connect {
            properties["enableIpConnect"] = serde_json::json!(true);
        }
        if file_copy {
            properties["enableFileCopy"] = serde_json::json!(true);
        }
        if disable_copy_paste {
            properties["disableCopyPaste"] = serde_json::json!(true);
        }
        if kerberos {
            properties["enableKerberos"] = serde_json::json!(true);
        }
        if session_recording {
            properties["enableSessionRecording"] = serde_json::json!(true);
        }
        if shareable_link {
            properties["enableShareableLink"] = serde_json::json!(true);
        }
        if let Some(acls) = network_acls_ips {
            let rules: Vec<serde_json::Value> = acls
                .iter()
                .map(|ip| serde_json::json!({ "addressPrefix": ip }))
                .collect();
            properties["networkAcls"] = serde_json::json!({ "ipRules": rules });
        }

        let mut body = serde_json::json!({
            "location": location,
            "sku": { "name": sku.to_string() },
            "properties": properties
        });

        if let Some(z) = zones {
            body["zones"] = serde_json::json!(z);
        }
        if let Some(t) = tags {
            body["tags"] = serde_json::to_value(t)?;
        }

        let resp = self
            .client
            .put(&url)
            .bearer_auth(&self.access_token)
            .json(&body)
            .send()
            .await
            .context("Failed to create bastion host")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Create bastion host failed ({status}): {body}");
        }

        resp.json().await.context("Failed to parse create response")
    }

    pub async fn delete(&self, resource_group: &str, bastion_name: &str) -> Result<()> {
        let url = self.bastion_url(resource_group, bastion_name);
        debug!("DELETE {url}");

        let resp = self
            .client
            .delete(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .context("Failed to delete bastion host")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Delete bastion host failed ({status}): {body}");
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        resource_group: &str,
        bastion_name: &str,
        sku: Option<BastionSku>,
        enable_tunneling: Option<bool>,
        enable_ip_connect: Option<bool>,
        file_copy: Option<bool>,
        disable_copy_paste: Option<bool>,
        kerberos: Option<bool>,
        session_recording: Option<bool>,
        shareable_link: Option<bool>,
        network_acls_ips: Option<&[String]>,
        tags: Option<std::collections::HashMap<String, String>>,
    ) -> Result<BastionHost> {
        let url = self.bastion_url(resource_group, bastion_name);
        debug!("PATCH {url}");

        let mut body = serde_json::json!({});
        let mut properties = serde_json::json!({});
        let mut has_properties = false;

        if let Some(s) = sku {
            body["sku"] = serde_json::json!({ "name": s.to_string() });
        }
        if let Some(v) = enable_tunneling {
            properties["enableTunneling"] = serde_json::json!(v);
            has_properties = true;
        }
        if let Some(v) = enable_ip_connect {
            properties["enableIpConnect"] = serde_json::json!(v);
            has_properties = true;
        }
        if let Some(v) = file_copy {
            properties["enableFileCopy"] = serde_json::json!(v);
            has_properties = true;
        }
        if let Some(v) = disable_copy_paste {
            properties["disableCopyPaste"] = serde_json::json!(v);
            has_properties = true;
        }
        if let Some(v) = kerberos {
            properties["enableKerberos"] = serde_json::json!(v);
            has_properties = true;
        }
        if let Some(v) = session_recording {
            properties["enableSessionRecording"] = serde_json::json!(v);
            has_properties = true;
        }
        if let Some(v) = shareable_link {
            properties["enableShareableLink"] = serde_json::json!(v);
            has_properties = true;
        }
        if let Some(acls) = network_acls_ips {
            let rules: Vec<serde_json::Value> = acls
                .iter()
                .map(|ip| serde_json::json!({ "addressPrefix": ip }))
                .collect();
            properties["networkAcls"] = serde_json::json!({ "ipRules": rules });
            has_properties = true;
        }
        if has_properties {
            body["properties"] = properties;
        }
        if let Some(tags) = tags {
            body["tags"] = serde_json::to_value(tags)?;
        }

        let resp = self
            .client
            .patch(&url)
            .bearer_auth(&self.access_token)
            .json(&body)
            .send()
            .await
            .context("Failed to update bastion host")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Update bastion host failed ({status}): {body}");
        }

        resp.json().await.context("Failed to parse update response")
    }

    pub async fn get_tunnel_token(
        &self,
        bastion_endpoint: &str,
        resource_id: &str,
        resource_port: u16,
        last_token: Option<&str>,
        hostname: Option<&str>,
        node_id: Option<&str>,
    ) -> Result<TokenResponse> {
        let url = format!("https://{bastion_endpoint}/api/tokens");
        debug!("POST {url}");

        let mut form = vec![
            ("resourceId".to_string(), resource_id.to_string()),
            ("protocol".to_string(), "tcptunnel".to_string()),
            ("workloadHostPort".to_string(), resource_port.to_string()),
            ("aztoken".to_string(), self.access_token.clone()),
        ];

        if let Some(t) = last_token {
            form.push(("token".to_string(), t.to_string()));
        }
        if let Some(h) = hostname {
            form.push(("hostname".to_string(), h.to_string()));
        }

        let mut req = self.client.post(&url).form(&form);
        if let Some(nid) = node_id {
            req = req.header("X-Node-Id", nid);
        }

        let resp = req.send().await.context("Failed to get tunnel token")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get tunnel token failed ({status}): {body}");
        }

        resp.json().await.context("Failed to parse token response")
    }

    pub async fn delete_tunnel_token(
        &self,
        bastion_endpoint: &str,
        token: &str,
        node_id: Option<&str>,
    ) -> Result<()> {
        let url = format!("https://{bastion_endpoint}/api/tokens/{token}");
        debug!("DELETE {url}");

        let mut req = self.client.delete(&url);
        if let Some(nid) = node_id {
            req = req.header("X-Node-Id", nid);
        }

        let resp = req.send().await.context("Failed to delete tunnel token")?;

        match resp.status().as_u16() {
            200 | 204 | 404 => Ok(()),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Delete tunnel token failed ({status}): {body}");
            }
        }
    }

    pub async fn get_developer_endpoint(
        &self,
        bastion: &BastionHost,
        resource_id: &str,
        resource_port: u16,
    ) -> Result<String> {
        let props = bastion.properties.as_ref().context("Bastion has no properties")?;
        let dns_name = props.dns_name.as_deref().context("Bastion has no DNS name")?;

        let url = format!("https://{dns_name}/api/connection");
        debug!("POST {url}");

        let body = serde_json::json!({
            "resourceId": resource_id,
            "bastionResourceId": bastion.id,
            "vmPort": resource_port,
            "azToken": self.access_token,
            "connectionType": "nativeclient"
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to get developer endpoint")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get developer endpoint failed ({status}): {body}");
        }

        resp.text().await.context("Failed to read developer endpoint response")
    }

    pub async fn get_rdp_file(
        &self,
        bastion_endpoint: &str,
        resource_id: &str,
        resource_port: u16,
        enable_mfa: bool,
    ) -> Result<String> {
        let url = format!(
            "https://{bastion_endpoint}/api/rdpfile?resourceId={resource_id}&format=rdp&rdpport={resource_port}&enablerdsaad={enable_mfa}"
        );
        debug!("GET {url}");

        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.access_token)
            .header("Accept", "*/*")
            .header("Content-Type", "application/json")
            .send()
            .await
            .context("Failed to get RDP file")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get RDP file failed ({status}): {body}");
        }

        resp.text().await.context("Failed to read RDP file response")
    }
}
