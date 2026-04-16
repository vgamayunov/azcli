use anyhow::{Context, Result};
use reqwest::Client;
use tracing::debug;

use crate::models::*;

const RESOURCE_GROUP_API_VERSION: &str = "2024-03-01";
const COMPUTE_API_VERSION: &str = "2024-07-01";

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
        debug!("GET {url}");

        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .context("Failed to list resource groups")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("List resource groups failed ({status}): {body}");
        }

        let list: AzureListResponse<ResourceGroup> =
            resp.json().await.context("Failed to parse resource group list")?;
        Ok(list.value)
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
        debug!("GET {url}");

        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .context("Failed to list virtual machines")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("List virtual machines failed ({status}): {body}");
        }

        let list: AzureListResponse<VirtualMachine> =
            resp.json().await.context("Failed to parse VM list")?;
        Ok(list.value)
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
        debug!("GET {url}");

        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to list VMSS")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("List VMSS failed ({status}): {body}");
        }

        let list: AzureListResponse<Vmss> = resp.json().await.context("Failed to parse VMSS list")?;
        Ok(list.value)
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
        debug!("GET {url}");

        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to list VMSS instances")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("List VMSS instances failed ({status}): {body}");
        }

        let list: AzureListResponse<VmssInstance> = resp.json().await.context("Failed to parse VMSS instances")?;
        Ok(list.value)
    }

    pub async fn list_vmss_skus(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachineScaleSets/{}/skus?api-version={}",
            self.subscription_id, resource_group, name, COMPUTE_API_VERSION
        );
        debug!("GET {url}");

        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to list VMSS SKUs")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("List VMSS SKUs failed ({status}): {body}");
        }

        resp.json().await.context("Failed to parse VMSS SKUs")
    }

    pub async fn list_vmss_instance_public_ips(&self, resource_group: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/virtualMachineScaleSets/{}/publicipaddresses?api-version=2018-10-01",
            self.subscription_id, resource_group, name
        );
        debug!("GET {url}");

        let resp = self.client.get(&url).bearer_auth(&self.access_token).send().await
            .context("Failed to list VMSS instance public IPs")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("List VMSS instance public IPs failed ({status}): {body}");
        }

        resp.json().await.context("Failed to parse VMSS public IPs")
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
}
