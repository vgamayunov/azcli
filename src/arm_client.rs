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
}
