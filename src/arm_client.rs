use anyhow::{Context, Result};
use base64::Engine;
use reqwest::Client;
use tracing::debug;

use crate::models::*;

const RESOURCE_GROUP_API_VERSION: &str = "2024-03-01";
const COMPUTE_API_VERSION: &str = "2024-07-01";
const DEPLOYMENT_API_VERSION: &str = "2024-03-01";
const NETWORK_API_VERSION: &str = "2023-11-01";
const SKU_API_VERSION: &str = "2021-07-01";
const DEVTESTLAB_API_VERSION: &str = "2018-09-15";
const DISK_API_VERSION: &str = "2023-04-02";
const RESOURCE_SKU_API_VERSION: &str = "2021-07-01";
const PIM_API_VERSION: &str = "2020-10-01";
const ROLE_DEFINITION_API_VERSION: &str = "2022-04-01";
const ROLE_ASSIGNMENT_API_VERSION: &str = "2022-04-01";
const IMAGE_API_VERSION: &str = "2024-07-01";
const IMAGE_BUILDER_API_VERSION: &str = "2022-07-01";
const SIG_API_VERSION: &str = "2024-03-03";

#[derive(Clone)]
pub struct ArmClient {
    client: Client,
    subscription_id: String,
    access_token: String,
}

impl ArmClient {
    pub fn new(access_token: String, subscription_id: String) -> Self {
        Self {
            client: Client::new(),
            subscription_id,
            access_token,
        }
    }

    pub fn subscription_id(&self) -> &str {
        &self.subscription_id
    }

    pub async fn list_resource_groups(&self) -> Result<Vec<ResourceGroup>> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourcegroups?api-version={}",
            self.subscription_id, RESOURCE_GROUP_API_VERSION
        );
        self.arm_list_paginated(url, "list resource groups").await
    }

    pub async fn show_resource_group(&self, name: &str) -> Result<ResourceGroup> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourcegroups/{}?api-version={}",
            self.subscription_id, name, RESOURCE_GROUP_API_VERSION
        );
        debug!("GET {url}");

        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .context("Failed to get resource group")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get resource group failed ({status}): {body}");
        }

        resp.json().await.context("Failed to parse resource group response")
    }

    pub async fn list_vms(&self, resource_group: Option<&str>) -> Result<Vec<VirtualMachine>> {
        let url = match resource_group {
            Some(rg) => format!(
                "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines?api-version={}",
                self.subscription_id, rg, COMPUTE_API_VERSION
            ),
            None => format!(
                "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/virtualMachines?api-version={}",
                self.subscription_id, COMPUTE_API_VERSION
            ),
        };
        self.arm_list_paginated(url, "list virtual machines").await
    }

    pub async fn show_vm(&self, resource_group: &str, name: &str) -> Result<VirtualMachine> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}?$expand=instanceView&api-version={}",
            self.subscription_id, resource_group, name, COMPUTE_API_VERSION
        );
        debug!("GET {url}");

        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .context("Failed to get virtual machine")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get virtual machine failed ({status}): {body}");
        }

        resp.json().await.context("Failed to parse VM response")
    }

    pub async fn start_vm(&self, resource_group: &str, name: &str) -> Result<()> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}/start?api-version={}",
            self.subscription_id, resource_group, name, COMPUTE_API_VERSION
        );
        debug!("POST {url}");

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.access_token)
            .header("Content-Length", "0")
            .send()
            .await
            .context("Failed to start virtual machine")?;

        match resp.status().as_u16() {
            200 | 202 => Ok(()),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Start VM failed ({status}): {body}");
            }
        }
    }

    pub async fn stop_vm(&self, resource_group: &str, name: &str, deallocate: bool) -> Result<()> {
        let action = if deallocate { "deallocate" } else { "powerOff" };
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}/{}?api-version={}",
            self.subscription_id, resource_group, name, action, COMPUTE_API_VERSION
        );
        debug!("POST {url}");

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.access_token)
            .header("Content-Length", "0")
            .send()
            .await
            .with_context(|| format!("Failed to {action} virtual machine"))?;

        match resp.status().as_u16() {
            200 | 202 => Ok(()),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("{action} VM failed ({status}): {body}");
            }
        }
    }

    pub async fn vm_post_action(&self, resource_group: &str, name: &str, action: &str) -> Result<()> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}/{}?api-version={}",
            self.subscription_id, resource_group, name, action, COMPUTE_API_VERSION
        );
        debug!("POST {url}");

        let resp = self.client.post(&url).bearer_auth(&self.access_token)
            .header("Content-Length", "0").send().await
            .with_context(|| format!("Failed to {action} VM"))?;

        match resp.status().as_u16() {
            200 | 202 => Ok(()),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("{action} VM failed ({status}): {body}");
            }
        }
    }

    pub async fn vm_post_action_with_body(&self, resource_group: &str, name: &str, action: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}/{}?api-version={}",
            self.subscription_id, resource_group, name, action, COMPUTE_API_VERSION
        );
        debug!("POST {url}");

        let resp = self.client.post(&url).bearer_auth(&self.access_token)
            .json(&body).send().await
            .with_context(|| format!("Failed to {action} VM"))?;

        match resp.status().as_u16() {
            200 | 202 => resp.json().await.context("Failed to parse response"),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("{action} VM failed ({status}): {body}");
            }
        }
    }

    pub async fn vm_get_instance_view(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}/instanceView?api-version={}",
            self.subscription_id, resource_group, name, COMPUTE_API_VERSION
        );
        debug!("GET {url}");
        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to get VM instance view")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get VM instance view failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse instance view")
    }

    pub async fn get_network_interface(&self, nic_id: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com{nic_id}?api-version={NETWORK_API_VERSION}"
        );
        debug!("GET {url}");
        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to get network interface")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get NIC failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse NIC response")
    }

    pub async fn get_public_ip(&self, pip_id: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com{pip_id}?api-version={NETWORK_API_VERSION}"
        );
        debug!("GET {url}");
        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to get public IP")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get public IP failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse public IP response")
    }

    pub async fn vm_list_sizes(&self, location: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/locations/{}/vmSizes?api-version={}",
            self.subscription_id, location, COMPUTE_API_VERSION
        );
        self.arm_get_paginated(url, "list VM sizes").await
    }

    pub async fn vm_list_skus(&self) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/skus?api-version={}",
            self.subscription_id, SKU_API_VERSION
        );
        self.arm_get_paginated(url, "list compute SKUs").await
    }

    pub async fn vm_list_usage(&self, location: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/locations/{}/usages?api-version={}",
            self.subscription_id, location, COMPUTE_API_VERSION
        );
        self.arm_get_paginated(url, "list VM usage").await
    }

    pub async fn vm_list_resize_options(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}/vmSizes?api-version={}",
            self.subscription_id, resource_group, name, COMPUTE_API_VERSION
        );
        self.arm_get_paginated(url, "list VM resize options").await
    }

    pub async fn vm_create(&self, resource_group: &str, name: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}?api-version={}",
            self.subscription_id, resource_group, name, COMPUTE_API_VERSION
        );
        debug!("PUT {url}");
        let resp = self.client.put(&url).bearer_auth(&self.access_token).json(&body).send().await
            .context("Failed to create VM")?;
        match resp.status().as_u16() {
            200 | 201 => resp.json().await.context("Failed to parse VM create response"),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Create VM failed ({status}): {body}");
            }
        }
    }

    pub async fn vm_delete(&self, resource_group: &str, name: &str, force: bool) -> Result<()> {
        let mut url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}?api-version={}",
            self.subscription_id, resource_group, name, COMPUTE_API_VERSION
        );
        if force {
            url.push_str("&forceDeletion=true");
        }
        debug!("DELETE {url}");
        let resp = self.client.delete(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to delete VM")?;
        match resp.status().as_u16() {
            200 | 202 | 204 => Ok(()),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Delete VM failed ({status}): {body}");
            }
        }
    }

    pub async fn vm_update(&self, resource_group: &str, name: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}?api-version={}",
            self.subscription_id, resource_group, name, COMPUTE_API_VERSION
        );
        debug!("PATCH {url}");
        let resp = self.client.patch(&url).bearer_auth(&self.access_token).json(&body).send().await
            .context("Failed to update VM")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Update VM failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse VM update response")
    }

    pub async fn vm_assess_patches(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}/assessPatches?api-version={}",
            self.subscription_id, resource_group, name, COMPUTE_API_VERSION
        );
        debug!("POST {url}");
        let resp = self.client.post(&url).bearer_auth(&self.access_token)
            .header("Content-Length", "0").send().await
            .context("Failed to assess patches")?;
        match resp.status().as_u16() {
            200 | 202 => {
                if resp.status().as_u16() == 202 {
                    let location = resp.headers().get("Location")
                        .and_then(|v| v.to_str().ok()).map(|s| s.to_string());
                    if let Some(poll_url) = location {
                        loop {
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                            let poll_resp = self.client.get(&poll_url).bearer_auth(&self.access_token).send().await
                                .context("Failed to poll assess patches")?;
                            match poll_resp.status().as_u16() {
                                200 => return poll_resp.json().await.context("Failed to parse assess patches result"),
                                202 => continue,
                                s => {
                                    let body = poll_resp.text().await.unwrap_or_default();
                                    anyhow::bail!("Assess patches poll failed ({s}): {body}");
                                }
                            }
                        }
                    }
                }
                resp.json().await.context("Failed to parse assess patches response")
            }
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Assess patches failed ({status}): {body}");
            }
        }
    }

    pub async fn vm_install_patches(&self, resource_group: &str, name: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}/installPatches?api-version={}",
            self.subscription_id, resource_group, name, COMPUTE_API_VERSION
        );
        debug!("POST {url}");
        let resp = self.client.post(&url).bearer_auth(&self.access_token)
            .json(&body).send().await
            .context("Failed to install patches")?;
        match resp.status().as_u16() {
            200 | 202 => {
                if resp.status().as_u16() == 202 {
                    let location = resp.headers().get("Location")
                        .and_then(|v| v.to_str().ok()).map(|s| s.to_string());
                    if let Some(poll_url) = location {
                        loop {
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                            let poll_resp = self.client.get(&poll_url).bearer_auth(&self.access_token).send().await
                                .context("Failed to poll install patches")?;
                            match poll_resp.status().as_u16() {
                                200 => return poll_resp.json().await.context("Failed to parse install patches result"),
                                202 => continue,
                                s => {
                                    let body = poll_resp.text().await.unwrap_or_default();
                                    anyhow::bail!("Install patches poll failed ({s}): {body}");
                                }
                            }
                        }
                    }
                }
                resp.json().await.context("Failed to parse install patches response")
            }
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Install patches failed ({status}): {body}");
            }
        }
    }

    pub async fn vm_auto_shutdown(&self, resource_group: &str, name: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.DevTestLab/schedules/shutdown-computevm-{}?api-version={}",
            self.subscription_id, resource_group, name, DEVTESTLAB_API_VERSION
        );
        debug!("PUT {url}");
        let resp = self.client.put(&url).bearer_auth(&self.access_token).json(&body).send().await
            .context("Failed to set auto-shutdown")?;
        match resp.status().as_u16() {
            200 | 201 => resp.json().await.context("Failed to parse auto-shutdown response"),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Set auto-shutdown failed ({status}): {body}");
            }
        }
    }

    pub async fn vm_auto_shutdown_delete(&self, resource_group: &str, name: &str) -> Result<()> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.DevTestLab/schedules/shutdown-computevm-{}?api-version={}",
            self.subscription_id, resource_group, name, DEVTESTLAB_API_VERSION
        );
        debug!("DELETE {url}");
        let resp = self.client.delete(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to delete auto-shutdown")?;
        match resp.status().as_u16() {
            200 | 204 => Ok(()),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Delete auto-shutdown failed ({status}): {body}");
            }
        }
    }

    pub async fn get_resource_by_id(&self, resource_id: &str, api_version: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com{resource_id}?api-version={api_version}"
        );
        debug!("GET {url}");
        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to get resource")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get resource failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse resource response")
    }

    pub async fn put_resource_by_id(&self, resource_id: &str, api_version: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com{resource_id}?api-version={api_version}"
        );
        debug!("PUT {url}");
        let resp = self.client.put(&url).bearer_auth(&self.access_token).json(&body).send().await
            .context("Failed to put resource")?;
        match resp.status().as_u16() {
            200 | 201 => resp.json().await.context("Failed to parse resource response"),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Put resource failed ({status}): {body}");
            }
        }
    }

    pub async fn list_vmss(&self, resource_group: Option<&str>) -> Result<Vec<Vmss>> {
        let url = match resource_group {
            Some(rg) => format!(
                "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachineScaleSets?api-version={}",
                self.subscription_id, rg, COMPUTE_API_VERSION
            ),
            None => format!(
                "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/virtualMachineScaleSets?api-version={}",
                self.subscription_id, COMPUTE_API_VERSION
            ),
        };
        self.arm_list_paginated(url, "list VMSS").await
    }

    pub async fn show_vmss(&self, resource_group: &str, name: &str) -> Result<Vmss> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachineScaleSets/{}?api-version={}",
            self.subscription_id, resource_group, name, COMPUTE_API_VERSION
        );
        debug!("GET {url}");

        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to get VMSS")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get VMSS failed ({status}): {body}");
        }

        resp.json().await.context("Failed to parse VMSS response")
    }

    pub async fn list_vmss_instances(&self, resource_group: &str, name: &str, expand: Option<&str>) -> Result<Vec<VmssInstance>> {
        let mut url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachineScaleSets/{}/virtualMachines?api-version={}",
            self.subscription_id, resource_group, name, COMPUTE_API_VERSION
        );
        if let Some(exp) = expand {
            url.push_str(&format!("&$expand={exp}"));
        }
        self.arm_list_paginated(url, "list VMSS instances").await
    }

    pub async fn list_vmss_skus(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachineScaleSets/{}/skus?api-version={}",
            self.subscription_id, resource_group, name, COMPUTE_API_VERSION
        );
        self.arm_get_paginated(url, "list VMSS SKUs").await
    }

    pub async fn list_vmss_instance_public_ips(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachineScaleSets/{}/publicipaddresses?api-version=2018-10-01",
            self.subscription_id, resource_group, name
        );
        self.arm_get_paginated(url, "list VMSS instance public IPs").await
    }

    pub async fn vmss_scale(&self, resource_group: &str, name: &str, capacity: i64) -> Result<()> {
        let vmss = self.show_vmss(resource_group, name).await?;
        let mut sku_value = match &vmss.sku {
            Some(sku) => serde_json::to_value(sku)?,
            None => anyhow::bail!("VMSS has no SKU"),
        };
        sku_value["capacity"] = serde_json::Value::Number(capacity.into());

        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachineScaleSets/{}?api-version={}",
            self.subscription_id, resource_group, name, COMPUTE_API_VERSION
        );
        debug!("PATCH {url}");

        let body = serde_json::json!({ "sku": sku_value });
        let resp = self.client.patch(&url).bearer_auth(&self.access_token).json(&body).send().await
            .context("Failed to scale VMSS")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Scale VMSS failed ({status}): {body}");
        }
        Ok(())
    }

    pub async fn vmss_start(&self, resource_group: &str, name: &str, instance_ids: Option<&[String]>) -> Result<()> {
        self.vmss_action(resource_group, name, "start", instance_ids).await
    }

    pub async fn vmss_stop(&self, resource_group: &str, name: &str, instance_ids: Option<&[String]>) -> Result<()> {
        self.vmss_action(resource_group, name, "poweroff", instance_ids).await
    }

    pub async fn vmss_update_instances(&self, resource_group: &str, name: &str, instance_ids: &[String]) -> Result<()> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachineScaleSets/{}/manualupgrade?api-version={}",
            self.subscription_id, resource_group, name, COMPUTE_API_VERSION
        );
        debug!("POST {url}");

        let body = serde_json::json!({ "instanceIds": instance_ids });
        let resp = self.client.post(&url).bearer_auth(&self.access_token).json(&body).send().await
            .context("Failed to update VMSS instances")?;

        match resp.status().as_u16() {
            200 | 202 => Ok(()),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Update VMSS instances failed ({status}): {body}");
            }
        }
    }

    async fn vmss_action(&self, resource_group: &str, name: &str, action: &str, instance_ids: Option<&[String]>) -> Result<()> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachineScaleSets/{}/{}?api-version={}",
            self.subscription_id, resource_group, name, action, COMPUTE_API_VERSION
        );
        debug!("POST {url}");

        let builder = self.client.post(&url).bearer_auth(&self.access_token);
        let resp = match instance_ids {
            Some(ids) => builder.json(&serde_json::json!({ "instanceIds": ids })).send().await,
            None => builder.header("Content-Length", "0").send().await,
        }.with_context(|| format!("Failed to {action} VMSS"))?;

        match resp.status().as_u16() {
            200 | 202 => Ok(()),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("{action} VMSS failed ({status}): {body}");
            }
        }
    }

    pub fn deployment_base_url_group(&self, resource_group: &str) -> String {
        format!(
            "https://management.azure.com/subscriptions/{}/resourcegroups/{}/providers/Microsoft.Resources/deployments",
            self.subscription_id, resource_group
        )
    }

    pub fn deployment_base_url_sub(&self) -> String {
        format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Resources/deployments",
            self.subscription_id
        )
    }

    pub fn deployment_base_url_mg(mg_id: &str) -> String {
        format!(
            "https://management.azure.com/providers/Microsoft.Management/managementGroups/{}/providers/Microsoft.Resources/deployments",
            mg_id
        )
    }

    pub fn deployment_base_url_tenant() -> String {
        "https://management.azure.com/providers/Microsoft.Resources/deployments".to_string()
    }

    pub async fn deployment_list(&self, base_url: &str) -> Result<serde_json::Value> {
        let url = format!("{base_url}?api-version={DEPLOYMENT_API_VERSION}");
        self.arm_get_paginated(url, "list deployments").await
    }

    pub async fn deployment_show(&self, base_url: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!("{base_url}/{name}?api-version={DEPLOYMENT_API_VERSION}");
        debug!("GET {url}");
        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to get deployment")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get deployment failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse deployment response")
    }

    pub async fn deployment_export(&self, base_url: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!("{base_url}/{name}/exportTemplate?api-version={DEPLOYMENT_API_VERSION}");
        debug!("POST {url}");
        let resp = self.client.post(&url).bearer_auth(&self.access_token)
            .header("Content-Length", "0").send().await
            .context("Failed to export deployment template")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Export deployment template failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse exported template")
    }

    pub async fn deployment_create(&self, base_url: &str, name: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!("{base_url}/{name}?api-version={DEPLOYMENT_API_VERSION}");
        debug!("PUT {url}");
        let resp = self.client.put(&url).bearer_auth(&self.access_token).json(&body).send().await
            .context("Failed to create deployment")?;
        match resp.status().as_u16() {
            200 | 201 => resp.json().await.context("Failed to parse deployment response"),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Create deployment failed ({status}): {body}");
            }
        }
    }

    pub async fn deployment_delete(&self, base_url: &str, name: &str) -> Result<()> {
        let url = format!("{base_url}/{name}?api-version={DEPLOYMENT_API_VERSION}");
        debug!("DELETE {url}");
        let resp = self.client.delete(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to delete deployment")?;
        match resp.status().as_u16() {
            200 | 202 | 204 => Ok(()),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Delete deployment failed ({status}): {body}");
            }
        }
    }

    // ARM returns 200 on valid, 400 on invalid — both are useful JSON
    pub async fn deployment_validate(&self, base_url: &str, name: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!("{base_url}/{name}/validate?api-version={DEPLOYMENT_API_VERSION}");
        debug!("POST {url}");
        let resp = self.client.post(&url).bearer_auth(&self.access_token).json(&body).send().await
            .context("Failed to validate deployment")?;
        resp.json().await.context("Failed to parse validation response")
    }

    pub async fn deployment_what_if(&self, base_url: &str, name: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!("{base_url}/{name}/whatIf?api-version={DEPLOYMENT_API_VERSION}");
        debug!("POST {url}");
        let resp = self.client.post(&url).bearer_auth(&self.access_token).json(&body).send().await
            .context("Failed to execute what-if")?;

        let status_code = resp.status().as_u16();
        if status_code == 202 {
            let location = resp.headers().get("Location")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            if let Some(poll_url) = location {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    debug!("GET {poll_url} (polling what-if)");
                    let poll_resp = self.client.get(&poll_url).bearer_auth(&self.access_token).send().await
                        .context("Failed to poll what-if result")?;
                    let poll_status = poll_resp.status().as_u16();
                    if poll_status == 200 {
                        return poll_resp.json().await.context("Failed to parse what-if result");
                    } else if poll_status == 202 {
                        continue;
                    } else {
                        let body = poll_resp.text().await.unwrap_or_default();
                        anyhow::bail!("What-if polling failed ({poll_status}): {body}");
                    }
                }
            }
            resp.json().await.context("Failed to parse what-if response")
        } else if resp.status().is_success() {
            resp.json().await.context("Failed to parse what-if response")
        } else {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("What-if failed ({status_code}): {body}");
        }
    }

    pub async fn deployment_cancel(&self, base_url: &str, name: &str) -> Result<()> {
        let url = format!("{base_url}/{name}/cancel?api-version={DEPLOYMENT_API_VERSION}");
        debug!("POST {url}");
        let resp = self.client.post(&url).bearer_auth(&self.access_token)
            .header("Content-Length", "0").send().await
            .context("Failed to cancel deployment")?;
        match resp.status().as_u16() {
            200 | 204 => Ok(()),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Cancel deployment failed ({status}): {body}");
            }
        }
    }

    pub async fn deployment_operations_list(&self, base_url: &str, deployment_name: &str) -> Result<serde_json::Value> {
        let url = format!("{base_url}/{deployment_name}/operations?api-version={DEPLOYMENT_API_VERSION}");
        self.arm_get_paginated(url, "list deployment operations").await
    }

    pub async fn deployment_operations_show(&self, base_url: &str, deployment_name: &str, operation_id: &str) -> Result<serde_json::Value> {
        let url = format!("{base_url}/{deployment_name}/operations/{operation_id}?api-version={DEPLOYMENT_API_VERSION}");
        debug!("GET {url}");
        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to get deployment operation")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get deployment operation failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse deployment operation")
    }

    pub async fn list_disks(&self, resource_group: Option<&str>) -> Result<serde_json::Value> {
        let url = match resource_group {
            Some(rg) => format!(
                "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/disks?api-version={}",
                self.subscription_id, rg, DISK_API_VERSION
            ),
            None => format!(
                "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/disks?api-version={}",
                self.subscription_id, DISK_API_VERSION
            ),
        };
        self.arm_get_paginated(url, "list disks").await
    }

    pub async fn show_disk(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/disks/{}?api-version={}",
            self.subscription_id, resource_group, name, DISK_API_VERSION
        );
        debug!("GET {url}");
        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to get disk")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get disk failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse disk response")
    }

    pub async fn list_disk_skus(&self) -> Result<serde_json::Value> {
        // Microsoft.Compute/skus filtered to disks; mirrors `az disk list-skus`.
        let url = format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/skus?api-version={}&$filter=resourceType%20eq%20%27disks%27",
            self.subscription_id, RESOURCE_SKU_API_VERSION
        );
        self.arm_get_paginated(url, "list disk SKUs").await
    }

    pub async fn list_images(&self, resource_group: Option<&str>) -> Result<serde_json::Value> {
        let url = match resource_group {
            Some(rg) => format!(
                "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/images?api-version={}",
                self.subscription_id, rg, IMAGE_API_VERSION
            ),
            None => format!(
                "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/images?api-version={}",
                self.subscription_id, IMAGE_API_VERSION
            ),
        };
        self.arm_get_paginated(url, "list images").await
    }

    pub async fn show_image(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/images/{}?api-version={}",
            self.subscription_id, resource_group, name, IMAGE_API_VERSION
        );
        debug!("GET {url}");
        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to get image")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get image failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse image response")
    }

    pub async fn list_image_templates(&self, resource_group: Option<&str>) -> Result<serde_json::Value> {
        let url = match resource_group {
            Some(rg) => format!(
                "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.VirtualMachineImages/imageTemplates?api-version={}",
                self.subscription_id, rg, IMAGE_BUILDER_API_VERSION
            ),
            None => format!(
                "https://management.azure.com/subscriptions/{}/providers/Microsoft.VirtualMachineImages/imageTemplates?api-version={}",
                self.subscription_id, IMAGE_BUILDER_API_VERSION
            ),
        };
        self.arm_get_paginated(url, "list image templates").await
    }

    pub async fn show_image_template(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.VirtualMachineImages/imageTemplates/{}?api-version={}",
            self.subscription_id, resource_group, name, IMAGE_BUILDER_API_VERSION
        );
        debug!("GET {url}");
        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to get image template")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get image template failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse image template response")
    }

    pub async fn list_image_template_run_outputs(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.VirtualMachineImages/imageTemplates/{}/runOutputs?api-version={}",
            self.subscription_id, resource_group, name, IMAGE_BUILDER_API_VERSION
        );
        self.arm_get_paginated(url, "list image template run outputs").await
    }

    pub async fn show_image_template_run_output(&self, resource_group: &str, name: &str, output_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.VirtualMachineImages/imageTemplates/{}/runOutputs/{}?api-version={}",
            self.subscription_id, resource_group, name, output_name, IMAGE_BUILDER_API_VERSION
        );
        debug!("GET {url}");
        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to get image template run output")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get run output failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse run output response")
    }

    async fn arm_get(&self, url: String, what: &'static str) -> Result<serde_json::Value> {
        debug!("GET {url}");
        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .with_context(|| format!("Failed to {what}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("{what} failed ({status}): {body}");
        }
        resp.json().await.with_context(|| format!("Failed to parse {what} response"))
    }

    async fn arm_get_paginated(&self, url: String, what: &'static str) -> Result<serde_json::Value> {
        let mut all_values: Vec<serde_json::Value> = Vec::new();
        let mut next = Some(url);
        while let Some(u) = next.take() {
            debug!("GET {u}");
            let resp = self.client.get(&u).bearer_auth(&self.access_token).send().await
                .with_context(|| format!("Failed to {what}"))?;
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("{what} failed ({status}): {body}");
            }
            let mut page: serde_json::Value = resp.json().await
                .with_context(|| format!("Failed to parse {what} response"))?;
            if let Some(arr) = page.get_mut("value").and_then(|v| v.as_array_mut()) {
                all_values.append(arr);
            }
            next = page.get("nextLink").and_then(|v| v.as_str()).map(|s| s.to_string());
        }
        Ok(serde_json::json!({ "value": all_values }))
    }

    async fn arm_list_paginated<T: serde::de::DeserializeOwned>(
        &self,
        url: String,
        what: &'static str,
    ) -> Result<Vec<T>> {
        let mut all: Vec<T> = Vec::new();
        let mut next = Some(url);
        while let Some(u) = next.take() {
            debug!("GET {u}");
            let resp = self.client.get(&u).bearer_auth(&self.access_token).send().await
                .with_context(|| format!("Failed to {what}"))?;
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("{what} failed ({status}): {body}");
            }
            let page: AzureListResponse<T> = resp.json().await
                .with_context(|| format!("Failed to parse {what} response"))?;
            all.extend(page.value);
            next = page.next_link;
        }
        Ok(all)
    }

    pub async fn list_galleries(&self, resource_group: Option<&str>) -> Result<serde_json::Value> {
        let url = match resource_group {
            Some(rg) => format!(
                "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/galleries?api-version={}",
                self.subscription_id, rg, SIG_API_VERSION
            ),
            None => format!(
                "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/galleries?api-version={}",
                self.subscription_id, SIG_API_VERSION
            ),
        };
        self.arm_get_paginated(url, "list galleries").await
    }

    pub async fn show_gallery(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/galleries/{}?api-version={}",
            self.subscription_id, resource_group, name, SIG_API_VERSION
        );
        self.arm_get(url, "get gallery").await
    }

    pub async fn list_shared_galleries(&self, location: &str, shared_to_tenant: bool) -> Result<serde_json::Value> {
        let suffix = if shared_to_tenant { "&sharedTo=tenant" } else { "" };
        let url = format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/locations/{}/sharedGalleries?api-version={}{}",
            self.subscription_id, location, SIG_API_VERSION, suffix
        );
        self.arm_get_paginated(url, "list shared galleries").await
    }

    pub async fn show_shared_gallery(&self, location: &str, gallery_unique_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/locations/{}/sharedGalleries/{}?api-version={}",
            self.subscription_id, location, gallery_unique_name, SIG_API_VERSION
        );
        self.arm_get(url, "get shared gallery").await
    }

    pub async fn show_community_gallery(&self, location: &str, public_gallery_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/locations/{}/communityGalleries/{}?api-version={}",
            self.subscription_id, location, public_gallery_name, SIG_API_VERSION
        );
        self.arm_get(url, "get community gallery").await
    }

    pub async fn list_community_galleries(&self, location: Option<&str>, top: usize) -> Result<serde_json::Value> {
        let where_loc = match location {
            Some(loc) => format!(" | where location == '{}'", loc.replace('\'', "''")),
            None => String::new(),
        };
        let query = format!(
            "communitygalleryresources | where type == 'microsoft.compute/locations/communitygalleries'{}",
            where_loc
        );
        let body = serde_json::json!({
            "query": query,
            "options": { "$top": top },
        });
        let url = "https://management.azure.com/providers/Microsoft.ResourceGraph/resources?api-version=2021-03-01";
        debug!("POST {url}\n{body}");
        let resp = self.client.post(url).bearer_auth(&self.access_token).json(&body).send().await
            .context("Failed to query Resource Graph for community galleries")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("List community galleries failed ({status}): {text}");
        }
        let json: serde_json::Value = resp.json().await.context("Failed to parse Resource Graph response")?;
        Ok(json.get("data").cloned().unwrap_or(serde_json::Value::Array(vec![])))
    }

    pub async fn list_gallery_image_definitions(&self, resource_group: &str, gallery_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/galleries/{}/images?api-version={}",
            self.subscription_id, resource_group, gallery_name, SIG_API_VERSION
        );
        self.arm_get_paginated(url, "list gallery image definitions").await
    }

    pub async fn show_gallery_image_definition(&self, resource_group: &str, gallery_name: &str, image_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/galleries/{}/images/{}?api-version={}",
            self.subscription_id, resource_group, gallery_name, image_name, SIG_API_VERSION
        );
        self.arm_get(url, "get gallery image definition").await
    }

    pub async fn list_shared_gallery_image_definitions(&self, location: &str, gallery_unique_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/locations/{}/sharedGalleries/{}/images?api-version={}",
            self.subscription_id, location, gallery_unique_name, SIG_API_VERSION
        );
        self.arm_get_paginated(url, "list shared gallery image definitions").await
    }

    pub async fn show_shared_gallery_image_definition(&self, location: &str, gallery_unique_name: &str, image_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/locations/{}/sharedGalleries/{}/images/{}?api-version={}",
            self.subscription_id, location, gallery_unique_name, image_name, SIG_API_VERSION
        );
        self.arm_get(url, "get shared gallery image definition").await
    }

    pub async fn list_community_gallery_image_definitions(&self, location: &str, public_gallery_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/locations/{}/communityGalleries/{}/images?api-version={}",
            self.subscription_id, location, public_gallery_name, SIG_API_VERSION
        );
        self.arm_get_paginated(url, "list community gallery image definitions").await
    }

    pub async fn show_community_gallery_image_definition(&self, location: &str, public_gallery_name: &str, image_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/locations/{}/communityGalleries/{}/images/{}?api-version={}",
            self.subscription_id, location, public_gallery_name, image_name, SIG_API_VERSION
        );
        self.arm_get(url, "get community gallery image definition").await
    }

    pub async fn list_gallery_image_versions(&self, resource_group: &str, gallery_name: &str, image_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/galleries/{}/images/{}/versions?api-version={}",
            self.subscription_id, resource_group, gallery_name, image_name, SIG_API_VERSION
        );
        self.arm_get_paginated(url, "list gallery image versions").await
    }

    pub async fn show_gallery_image_version(&self, resource_group: &str, gallery_name: &str, image_name: &str, version: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/galleries/{}/images/{}/versions/{}?api-version={}",
            self.subscription_id, resource_group, gallery_name, image_name, version, SIG_API_VERSION
        );
        self.arm_get(url, "get gallery image version").await
    }

    pub async fn list_shared_gallery_image_versions(&self, location: &str, gallery_unique_name: &str, image_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/locations/{}/sharedGalleries/{}/images/{}/versions?api-version={}",
            self.subscription_id, location, gallery_unique_name, image_name, SIG_API_VERSION
        );
        self.arm_get_paginated(url, "list shared gallery image versions").await
    }

    pub async fn show_shared_gallery_image_version(&self, location: &str, gallery_unique_name: &str, image_name: &str, version: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/locations/{}/sharedGalleries/{}/images/{}/versions/{}?api-version={}",
            self.subscription_id, location, gallery_unique_name, image_name, version, SIG_API_VERSION
        );
        self.arm_get(url, "get shared gallery image version").await
    }

    pub async fn list_community_gallery_image_versions(&self, location: &str, public_gallery_name: &str, image_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/locations/{}/communityGalleries/{}/images/{}/versions?api-version={}",
            self.subscription_id, location, public_gallery_name, image_name, SIG_API_VERSION
        );
        self.arm_get_paginated(url, "list community gallery image versions").await
    }

    pub async fn show_community_gallery_image_version(&self, location: &str, public_gallery_name: &str, image_name: &str, version: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/locations/{}/communityGalleries/{}/images/{}/versions/{}?api-version={}",
            self.subscription_id, location, public_gallery_name, image_name, version, SIG_API_VERSION
        );
        self.arm_get(url, "get community gallery image version").await
    }

    pub async fn create_disk(&self, resource_group: &str, name: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/disks/{}?api-version={}",
            self.subscription_id, resource_group, name, DISK_API_VERSION
        );
        debug!("PUT {url}");
        let resp = self.client.put(&url).bearer_auth(&self.access_token).json(&body).send().await
            .context("Failed to create disk")?;
        match resp.status().as_u16() {
            200 | 201 | 202 => resp.json().await.context("Failed to parse create disk response"),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Create disk failed ({status}): {body}");
            }
        }
    }

    pub async fn update_disk(&self, resource_group: &str, name: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/disks/{}?api-version={}",
            self.subscription_id, resource_group, name, DISK_API_VERSION
        );
        debug!("PATCH {url}");
        let resp = self.client.patch(&url).bearer_auth(&self.access_token).json(&body).send().await
            .context("Failed to update disk")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Update disk failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse update disk response")
    }

    pub async fn delete_disk(&self, resource_group: &str, name: &str) -> Result<()> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/disks/{}?api-version={}",
            self.subscription_id, resource_group, name, DISK_API_VERSION
        );
        debug!("DELETE {url}");
        let resp = self.client.delete(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to delete disk")?;
        match resp.status().as_u16() {
            200 | 202 | 204 => Ok(()),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Delete disk failed ({status}): {body}");
            }
        }
    }

    pub async fn disk_grant_access(&self, resource_group: &str, name: &str, access: &str, duration_in_seconds: i64) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/disks/{}/beginGetAccess?api-version={}",
            self.subscription_id, resource_group, name, DISK_API_VERSION
        );
        debug!("POST {url}");
        let body = serde_json::json!({
            "access": access,
            "durationInSeconds": duration_in_seconds,
        });
        let resp = self.client.post(&url).bearer_auth(&self.access_token).json(&body).send().await
            .context("Failed to begin disk access")?;

        let status_code = resp.status().as_u16();
        if status_code == 200 {
            return resp.json().await.context("Failed to parse grant access response");
        }
        if status_code != 202 {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Grant disk access failed ({status_code}): {body}");
        }

        // Poll Azure-AsyncOperation or Location for SAS URL
        let poll_url = resp.headers().get("Azure-AsyncOperation")
            .or_else(|| resp.headers().get("Location"))
            .and_then(|v| v.to_str().ok()).map(|s| s.to_string())
            .context("Grant disk access returned 202 without poll URL")?;

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            debug!("GET {poll_url} (polling grant-access)");
            let poll_resp = self.client.get(&poll_url).bearer_auth(&self.access_token).send().await
                .context("Failed to poll grant access")?;
            match poll_resp.status().as_u16() {
                200 => return poll_resp.json().await.context("Failed to parse grant access result"),
                202 => continue,
                s => {
                    let body = poll_resp.text().await.unwrap_or_default();
                    anyhow::bail!("Grant disk access poll failed ({s}): {body}");
                }
            }
        }
    }

    pub async fn disk_revoke_access(&self, resource_group: &str, name: &str) -> Result<()> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/disks/{}/endGetAccess?api-version={}",
            self.subscription_id, resource_group, name, DISK_API_VERSION
        );
        debug!("POST {url}");
        let resp = self.client.post(&url).bearer_auth(&self.access_token)
            .header("Content-Length", "0").send().await
            .context("Failed to revoke disk access")?;
        match resp.status().as_u16() {
            200 | 202 | 204 => Ok(()),
            status => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Revoke disk access failed ({status}): {body}");
            }
        }
    }

    pub async fn vm_run_command_invoke(&self, resource_group: &str, vm_name: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}/runCommand?api-version={}",
            self.subscription_id, resource_group, vm_name, COMPUTE_API_VERSION
        );
        debug!("POST {url}");
        let resp = self.client.post(&url).bearer_auth(&self.access_token).json(&body).send().await
            .context("Failed to invoke run-command")?;
        let status = resp.status().as_u16();
        if status == 200 {
            return resp.json().await.context("Failed to parse run-command response");
        }
        if status != 202 {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Invoke run-command failed ({status}): {body}");
        }
        let poll_url = resp.headers().get("Azure-AsyncOperation")
            .or_else(|| resp.headers().get("Location"))
            .and_then(|v| v.to_str().ok()).map(|s| s.to_string())
            .context("run-command returned 202 without poll URL")?;
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            debug!("GET {poll_url} (polling run-command)");
            let poll_resp = self.client.get(&poll_url).bearer_auth(&self.access_token).send().await
                .context("Failed to poll run-command")?;
            match poll_resp.status().as_u16() {
                200 => return poll_resp.json().await.context("Failed to parse run-command poll result"),
                202 => continue,
                s => {
                    let body = poll_resp.text().await.unwrap_or_default();
                    anyhow::bail!("run-command poll failed ({s}): {body}");
                }
            }
        }
    }

    pub async fn list_vm_run_commands(&self, resource_group: &str, vm_name: &str, expand_instance_view: bool) -> Result<serde_json::Value> {
        let mut url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}/runCommands?api-version={}",
            self.subscription_id, resource_group, vm_name, COMPUTE_API_VERSION
        );
        if expand_instance_view {
            url.push_str("&$expand=instanceView");
        }
        self.arm_get_paginated(url, "list VM run commands").await
    }

    pub async fn show_vm_run_command(&self, resource_group: &str, vm_name: &str, name: &str, instance_view: bool) -> Result<serde_json::Value> {
        let mut url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}/runCommands/{}?api-version={}",
            self.subscription_id, resource_group, vm_name, name, COMPUTE_API_VERSION
        );
        if instance_view {
            url.push_str("&$expand=instanceView");
        }
        debug!("GET {url}");
        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to show VM run command")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Show VM run command failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse show response")
    }

    pub async fn list_builtin_run_commands(&self, location: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/locations/{}/runCommands?api-version={}",
            self.subscription_id, location, COMPUTE_API_VERSION
        );
        self.arm_get_paginated(url, "list built-in run commands").await
    }

    pub async fn show_builtin_run_command(&self, location: &str, command_id: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/locations/{}/runCommands/{}?api-version={}",
            self.subscription_id, location, command_id, COMPUTE_API_VERSION
        );
        debug!("GET {url}");
        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to show built-in run command")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Show built-in run command failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse show response")
    }

    async fn poll_compute_lro(&self, poll_url: &str, op_name: &str) -> Result<serde_json::Value> {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            debug!("GET {poll_url} (polling {op_name})");
            let resp = self.client.get(poll_url).bearer_auth(&self.access_token).send().await
                .with_context(|| format!("Failed to poll {op_name}"))?;
            match resp.status().as_u16() {
                200 => {
                    return resp.json().await.or_else(|_| Ok(serde_json::json!({})));
                },
                202 => continue,
                204 => return Ok(serde_json::json!({})),
                s => {
                    let body = resp.text().await.unwrap_or_default();
                    anyhow::bail!("{op_name} poll failed ({s}): {body}");
                }
            }
        }
    }

    pub async fn create_vm_run_command(&self, resource_group: &str, vm_name: &str, name: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}/runCommands/{}?api-version={}",
            self.subscription_id, resource_group, vm_name, name, COMPUTE_API_VERSION
        );
        debug!("PUT {url}");
        let resp = self.client.put(&url).bearer_auth(&self.access_token).json(&body).send().await
            .context("Failed to create run command")?;
        let status = resp.status().as_u16();
        if status == 200 || status == 201 {
            if let Some(poll) = resp.headers().get("Azure-AsyncOperation").or_else(|| resp.headers().get("Location"))
                .and_then(|v| v.to_str().ok()).map(|s| s.to_string()) {
                return self.poll_compute_lro(&poll, "create run-command").await;
            }
            return resp.json().await.context("Failed to parse create response");
        }
        if status == 202 {
            let poll_url = resp.headers().get("Azure-AsyncOperation")
                .or_else(|| resp.headers().get("Location"))
                .and_then(|v| v.to_str().ok()).map(|s| s.to_string())
                .context("create returned 202 without poll URL")?;
            return self.poll_compute_lro(&poll_url, "create run-command").await;
        }
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Create run command failed ({status}): {body}");
    }

    pub async fn update_vm_run_command(&self, resource_group: &str, vm_name: &str, name: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}/runCommands/{}?api-version={}",
            self.subscription_id, resource_group, vm_name, name, COMPUTE_API_VERSION
        );
        debug!("PATCH {url}");
        let resp = self.client.patch(&url).bearer_auth(&self.access_token).json(&body).send().await
            .context("Failed to update run command")?;
        let status = resp.status().as_u16();
        if status == 200 {
            if let Some(poll) = resp.headers().get("Azure-AsyncOperation").or_else(|| resp.headers().get("Location"))
                .and_then(|v| v.to_str().ok()).map(|s| s.to_string()) {
                return self.poll_compute_lro(&poll, "update run-command").await;
            }
            return resp.json().await.context("Failed to parse update response");
        }
        if status == 202 {
            let poll_url = resp.headers().get("Azure-AsyncOperation")
                .or_else(|| resp.headers().get("Location"))
                .and_then(|v| v.to_str().ok()).map(|s| s.to_string())
                .context("update returned 202 without poll URL")?;
            return self.poll_compute_lro(&poll_url, "update run-command").await;
        }
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Update run command failed ({status}): {body}");
    }

    pub async fn delete_vm_run_command(&self, resource_group: &str, vm_name: &str, name: &str) -> Result<()> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachines/{}/runCommands/{}?api-version={}",
            self.subscription_id, resource_group, vm_name, name, COMPUTE_API_VERSION
        );
        debug!("DELETE {url}");
        let resp = self.client.delete(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to delete run command")?;
        let status = resp.status().as_u16();
        if status == 200 || status == 204 {
            return Ok(());
        }
        if status == 202 {
            let poll_url = resp.headers().get("Azure-AsyncOperation")
                .or_else(|| resp.headers().get("Location"))
                .and_then(|v| v.to_str().ok()).map(|s| s.to_string())
                .context("delete returned 202 without poll URL")?;
            self.poll_compute_lro(&poll_url, "delete run-command").await?;
            return Ok(());
        }
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Delete run command failed ({status}): {body}");
    }

    /// Extract the current user's Azure AD object ID (oid) from the access token JWT.
    /// Falls back to the `sub` claim if `oid` is absent.
    pub fn principal_id(&self) -> Result<String> {
        let parts: Vec<&str> = self.access_token.split('.').collect();
        if parts.len() != 3 {
            anyhow::bail!("invalid JWT: expected 3 segments, got {}", parts.len());
        }
        let payload = parts[1];
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload)
            .context("failed to base64-decode JWT payload")?;
        let claims: serde_json::Value =
            serde_json::from_slice(&decoded).context("failed to parse JWT payload as JSON")?;
        if let Some(oid) = claims.get("oid").and_then(|v| v.as_str()) {
            return Ok(oid.to_string());
        }
        if let Some(sub) = claims.get("sub").and_then(|v| v.as_str()) {
            return Ok(sub.to_string());
        }
        anyhow::bail!("access token has neither 'oid' nor 'sub' claim");
    }

    pub async fn list_eligible_role_schedules(&self, scope: &str, principal_id: &str) -> Result<serde_json::Value> {
        let filter = format!("principalId eq '{}'", principal_id);
        let encoded = urlencode(&filter);
        let url = format!(
            "https://management.azure.com{}/providers/Microsoft.Authorization/roleEligibilityScheduleInstances?api-version={}&$filter={}",
            scope, PIM_API_VERSION, encoded
        );
        self.arm_get_paginated(url, "list eligible roles").await
    }

    pub async fn list_active_role_schedules(&self, scope: &str, principal_id: &str) -> Result<serde_json::Value> {
        let filter = format!("principalId eq '{}'", principal_id);
        let encoded = urlencode(&filter);
        let url = format!(
            "https://management.azure.com{}/providers/Microsoft.Authorization/roleAssignmentScheduleInstances?api-version={}&$filter={}",
            scope, PIM_API_VERSION, encoded
        );
        self.arm_get_paginated(url, "list active assignments").await
    }

    pub async fn create_role_assignment_schedule_request(&self, scope: &str, request_name: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com{}/providers/Microsoft.Authorization/roleAssignmentScheduleRequests/{}?api-version={}",
            scope, request_name, PIM_API_VERSION
        );
        debug!("PUT {url}");
        let resp = self.client.put(&url).bearer_auth(&self.access_token).json(&body).send().await
            .context("Failed to create role assignment schedule request")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Role assignment schedule request failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse schedule request response")
    }

    pub async fn get_role_definition_by_id(&self, role_definition_id: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com{}?api-version={}",
            role_definition_id, ROLE_DEFINITION_API_VERSION
        );
        debug!("GET {url}");
        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to get role definition")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get role definition failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse role definition response")
    }

    pub async fn list_role_assignments(&self, scope: &str, filter: Option<&str>) -> Result<serde_json::Value> {
        let mut url = format!(
            "https://management.azure.com{}/providers/Microsoft.Authorization/roleAssignments?api-version={}",
            scope, ROLE_ASSIGNMENT_API_VERSION
        );
        if let Some(f) = filter {
            url.push_str("&$filter=");
            url.push_str(&urlencode(f));
        }
        debug!("GET {url}");
        let mut all_values: Vec<serde_json::Value> = Vec::new();
        let mut next = Some(url);
        while let Some(u) = next.take() {
            let resp = self.client.get(&u).bearer_auth(&self.access_token).send().await
                .context("Failed to list role assignments")?;
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("List role assignments failed ({status}): {body}");
            }
            let mut page: serde_json::Value = resp.json().await.context("Failed to parse role assignments response")?;
            if let Some(arr) = page.get_mut("value").and_then(|v| v.as_array_mut()) {
                all_values.append(arr);
            }
            next = page.get("nextLink").and_then(|v| v.as_str()).map(|s| s.to_string());
        }
        Ok(serde_json::json!({ "value": all_values }))
    }

    pub async fn get_role_assignment_by_id(&self, full_id: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com{}?api-version={}",
            full_id, ROLE_ASSIGNMENT_API_VERSION
        );
        debug!("GET {url}");
        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to get role assignment")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get role assignment failed ({status}): {body}");
        }
        resp.json().await.context("Failed to parse role assignment response")
    }

    pub async fn list_role_definitions(&self, scope: &str, filter: Option<&str>) -> Result<serde_json::Value> {
        let mut url = format!(
            "https://management.azure.com{}/providers/Microsoft.Authorization/roleDefinitions?api-version={}",
            scope, ROLE_DEFINITION_API_VERSION
        );
        if let Some(f) = filter {
            url.push_str("&$filter=");
            url.push_str(&urlencode(f));
        }
        debug!("GET {url}");
        let mut all_values: Vec<serde_json::Value> = Vec::new();
        let mut next = Some(url);
        while let Some(u) = next.take() {
            let resp = self.client.get(&u).bearer_auth(&self.access_token).send().await
                .context("Failed to list role definitions")?;
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("List role definitions failed ({status}): {body}");
            }
            let mut page: serde_json::Value = resp.json().await.context("Failed to parse role definitions response")?;
            if let Some(arr) = page.get_mut("value").and_then(|v| v.as_array_mut()) {
                all_values.append(arr);
            }
            next = page.get("nextLink").and_then(|v| v.as_str()).map(|s| s.to_string());
        }
        Ok(serde_json::json!({ "value": all_values }))
    }

    pub async fn list_locations(&self, subscription_id: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/locations?api-version=2022-12-01",
            subscription_id
        );
        self.arm_get_paginated(url, "list locations").await
    }

    pub async fn list_vnets(&self, resource_group: Option<&str>) -> Result<serde_json::Value> {
        let url = match resource_group {
            Some(rg) => format!(
                "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/virtualNetworks?api-version={}",
                self.subscription_id, rg, NETWORK_API_VERSION
            ),
            None => format!(
                "https://management.azure.com/subscriptions/{}/providers/Microsoft.Network/virtualNetworks?api-version={}",
                self.subscription_id, NETWORK_API_VERSION
            ),
        };
        self.arm_get_paginated(url, "list virtual networks").await
    }

    pub async fn show_vnet(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/virtualNetworks/{}?api-version={}",
            self.subscription_id, resource_group, name, NETWORK_API_VERSION
        );
        self.arm_get(url, "get virtual network").await
    }

    pub async fn list_subnets(&self, resource_group: &str, vnet_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/virtualNetworks/{}/subnets?api-version={}",
            self.subscription_id, resource_group, vnet_name, NETWORK_API_VERSION
        );
        self.arm_get_paginated(url, "list subnets").await
    }

    pub async fn show_subnet(&self, resource_group: &str, vnet_name: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/virtualNetworks/{}/subnets/{}?api-version={}",
            self.subscription_id, resource_group, vnet_name, name, NETWORK_API_VERSION
        );
        self.arm_get(url, "get subnet").await
    }

    pub async fn list_vnet_peerings(&self, resource_group: &str, vnet_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/virtualNetworks/{}/virtualNetworkPeerings?api-version={}",
            self.subscription_id, resource_group, vnet_name, NETWORK_API_VERSION
        );
        self.arm_get_paginated(url, "list vnet peerings").await
    }

    pub async fn show_vnet_peering(&self, resource_group: &str, vnet_name: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/virtualNetworks/{}/virtualNetworkPeerings/{}?api-version={}",
            self.subscription_id, resource_group, vnet_name, name, NETWORK_API_VERSION
        );
        self.arm_get(url, "get vnet peering").await
    }

    pub async fn list_nsgs(&self, resource_group: Option<&str>) -> Result<serde_json::Value> {
        let url = match resource_group {
            Some(rg) => format!(
                "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/networkSecurityGroups?api-version={}",
                self.subscription_id, rg, NETWORK_API_VERSION
            ),
            None => format!(
                "https://management.azure.com/subscriptions/{}/providers/Microsoft.Network/networkSecurityGroups?api-version={}",
                self.subscription_id, NETWORK_API_VERSION
            ),
        };
        self.arm_get_paginated(url, "list network security groups").await
    }

    pub async fn show_nsg(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/networkSecurityGroups/{}?api-version={}",
            self.subscription_id, resource_group, name, NETWORK_API_VERSION
        );
        self.arm_get(url, "get network security group").await
    }

    pub async fn list_nsg_rules(&self, resource_group: &str, nsg_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/networkSecurityGroups/{}/securityRules?api-version={}",
            self.subscription_id, resource_group, nsg_name, NETWORK_API_VERSION
        );
        self.arm_get_paginated(url, "list nsg rules").await
    }

    pub async fn show_nsg_rule(&self, resource_group: &str, nsg_name: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/networkSecurityGroups/{}/securityRules/{}?api-version={}",
            self.subscription_id, resource_group, nsg_name, name, NETWORK_API_VERSION
        );
        self.arm_get(url, "get nsg rule").await
    }

    pub async fn list_public_ips(&self, resource_group: Option<&str>) -> Result<serde_json::Value> {
        let url = match resource_group {
            Some(rg) => format!(
                "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/publicIPAddresses?api-version={}",
                self.subscription_id, rg, NETWORK_API_VERSION
            ),
            None => format!(
                "https://management.azure.com/subscriptions/{}/providers/Microsoft.Network/publicIPAddresses?api-version={}",
                self.subscription_id, NETWORK_API_VERSION
            ),
        };
        self.arm_get_paginated(url, "list public IP addresses").await
    }

    pub async fn show_public_ip(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/publicIPAddresses/{}?api-version={}",
            self.subscription_id, resource_group, name, NETWORK_API_VERSION
        );
        self.arm_get(url, "get public IP address").await
    }

    pub async fn list_nics(&self, resource_group: Option<&str>) -> Result<serde_json::Value> {
        let url = match resource_group {
            Some(rg) => format!(
                "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/networkInterfaces?api-version={}",
                self.subscription_id, rg, NETWORK_API_VERSION
            ),
            None => format!(
                "https://management.azure.com/subscriptions/{}/providers/Microsoft.Network/networkInterfaces?api-version={}",
                self.subscription_id, NETWORK_API_VERSION
            ),
        };
        self.arm_get_paginated(url, "list network interfaces").await
    }

    pub async fn show_nic(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/networkInterfaces/{}?api-version={}",
            self.subscription_id, resource_group, name, NETWORK_API_VERSION
        );
        self.arm_get(url, "get network interface").await
    }

    pub async fn list_nic_ip_configs(&self, resource_group: &str, nic_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/networkInterfaces/{}/ipConfigurations?api-version={}",
            self.subscription_id, resource_group, nic_name, NETWORK_API_VERSION
        );
        self.arm_get_paginated(url, "list nic ip configurations").await
    }

    pub async fn show_nic_ip_config(&self, resource_group: &str, nic_name: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/networkInterfaces/{}/ipConfigurations/{}?api-version={}",
            self.subscription_id, resource_group, nic_name, name, NETWORK_API_VERSION
        );
        self.arm_get(url, "get nic ip configuration").await
    }

    pub async fn list_private_endpoints(&self, resource_group: Option<&str>) -> Result<serde_json::Value> {
        let url = match resource_group {
            Some(rg) => format!(
                "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/privateEndpoints?api-version={}",
                self.subscription_id, rg, NETWORK_API_VERSION
            ),
            None => format!(
                "https://management.azure.com/subscriptions/{}/providers/Microsoft.Network/privateEndpoints?api-version={}",
                self.subscription_id, NETWORK_API_VERSION
            ),
        };
        self.arm_get_paginated(url, "list private endpoints").await
    }

    pub async fn show_private_endpoint(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/privateEndpoints/{}?api-version={}",
            self.subscription_id, resource_group, name, NETWORK_API_VERSION
        );
        self.arm_get(url, "get private endpoint").await
    }

    pub async fn list_load_balancers(&self, resource_group: Option<&str>) -> Result<serde_json::Value> {
        let url = match resource_group {
            Some(rg) => format!(
                "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers?api-version={}",
                self.subscription_id, rg, NETWORK_API_VERSION
            ),
            None => format!(
                "https://management.azure.com/subscriptions/{}/providers/Microsoft.Network/loadBalancers?api-version={}",
                self.subscription_id, NETWORK_API_VERSION
            ),
        };
        self.arm_get_paginated(url, "list load balancers").await
    }

    pub async fn show_load_balancer(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers/{}?api-version={}",
            self.subscription_id, resource_group, name, NETWORK_API_VERSION
        );
        self.arm_get(url, "get load balancer").await
    }

    pub async fn list_load_balancer_inbound_nat_rule_port_mappings(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers/{}/inboundNatRulePortMappings?api-version={}",
            self.subscription_id, resource_group, name, NETWORK_API_VERSION
        );
        self.arm_get_paginated(url, "list load balancer inbound NAT rule port mappings").await
    }

    pub async fn list_load_balancer_network_interfaces(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers/{}/networkInterfaces?api-version={}",
            self.subscription_id, resource_group, name, NETWORK_API_VERSION
        );
        self.arm_get_paginated(url, "list load balancer network interfaces").await
    }

    pub async fn list_load_balancer_backend_address_pools(&self, resource_group: &str, lb_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers/{}/backendAddressPools?api-version={}",
            self.subscription_id, resource_group, lb_name, NETWORK_API_VERSION
        );
        self.arm_get_paginated(url, "list load balancer backend address pools").await
    }

    pub async fn show_load_balancer_backend_address_pool(&self, resource_group: &str, lb_name: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers/{}/backendAddressPools/{}?api-version={}",
            self.subscription_id, resource_group, lb_name, name, NETWORK_API_VERSION
        );
        self.arm_get(url, "get load balancer backend address pool").await
    }

    pub async fn list_load_balancer_frontend_ip_configurations(&self, resource_group: &str, lb_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers/{}/frontendIPConfigurations?api-version={}",
            self.subscription_id, resource_group, lb_name, NETWORK_API_VERSION
        );
        self.arm_get_paginated(url, "list load balancer frontend IP configurations").await
    }

    pub async fn show_load_balancer_frontend_ip_configuration(&self, resource_group: &str, lb_name: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers/{}/frontendIPConfigurations/{}?api-version={}",
            self.subscription_id, resource_group, lb_name, name, NETWORK_API_VERSION
        );
        self.arm_get(url, "get load balancer frontend IP configuration").await
    }

    pub async fn list_load_balancer_inbound_nat_pools(&self, resource_group: &str, lb_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers/{}/inboundNatPools?api-version={}",
            self.subscription_id, resource_group, lb_name, NETWORK_API_VERSION
        );
        self.arm_get_paginated(url, "list load balancer inbound NAT pools").await
    }

    pub async fn show_load_balancer_inbound_nat_pool(&self, resource_group: &str, lb_name: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers/{}/inboundNatPools/{}?api-version={}",
            self.subscription_id, resource_group, lb_name, name, NETWORK_API_VERSION
        );
        self.arm_get(url, "get load balancer inbound NAT pool").await
    }

    pub async fn list_load_balancer_inbound_nat_rules(&self, resource_group: &str, lb_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers/{}/inboundNatRules?api-version={}",
            self.subscription_id, resource_group, lb_name, NETWORK_API_VERSION
        );
        self.arm_get_paginated(url, "list load balancer inbound NAT rules").await
    }

    pub async fn show_load_balancer_inbound_nat_rule(&self, resource_group: &str, lb_name: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers/{}/inboundNatRules/{}?api-version={}",
            self.subscription_id, resource_group, lb_name, name, NETWORK_API_VERSION
        );
        self.arm_get(url, "get load balancer inbound NAT rule").await
    }

    pub async fn list_load_balancer_outbound_rules(&self, resource_group: &str, lb_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers/{}/outboundRules?api-version={}",
            self.subscription_id, resource_group, lb_name, NETWORK_API_VERSION
        );
        self.arm_get_paginated(url, "list load balancer outbound rules").await
    }

    pub async fn show_load_balancer_outbound_rule(&self, resource_group: &str, lb_name: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers/{}/outboundRules/{}?api-version={}",
            self.subscription_id, resource_group, lb_name, name, NETWORK_API_VERSION
        );
        self.arm_get(url, "get load balancer outbound rule").await
    }

    pub async fn list_load_balancer_probes(&self, resource_group: &str, lb_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers/{}/probes?api-version={}",
            self.subscription_id, resource_group, lb_name, NETWORK_API_VERSION
        );
        self.arm_get_paginated(url, "list load balancer probes").await
    }

    pub async fn show_load_balancer_probe(&self, resource_group: &str, lb_name: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers/{}/probes/{}?api-version={}",
            self.subscription_id, resource_group, lb_name, name, NETWORK_API_VERSION
        );
        self.arm_get(url, "get load balancer probe").await
    }

    pub async fn list_load_balancer_rules(&self, resource_group: &str, lb_name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers/{}/loadBalancingRules?api-version={}",
            self.subscription_id, resource_group, lb_name, NETWORK_API_VERSION
        );
        self.arm_get_paginated(url, "list load balancer rules").await
    }

    pub async fn show_load_balancer_rule(&self, resource_group: &str, lb_name: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/loadBalancers/{}/loadBalancingRules/{}?api-version={}",
            self.subscription_id, resource_group, lb_name, name, NETWORK_API_VERSION
        );
        self.arm_get(url, "get load balancer rule").await
    }
}

fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}
