use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::debug;

const CACHE_FILE: &str = "azcli_tokens.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCache {
    pub accounts: Vec<CachedAccount>,
    pub default_subscription: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAccount {
    pub auth_method: AuthMethod,
    pub tenant_id: String,
    pub subscription_id: Option<String>,
    pub subscription_name: Option<String>,

    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,

    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub client_certificate_path: Option<String>,
    pub managed_identity_client_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    InteractiveBrowser,
    DeviceCode,
    ServicePrincipalSecret,
    ServicePrincipalCertificate,
    ManagedIdentity,
    AzCli,
}

impl TokenCache {
    pub fn load() -> Result<Self> {
        let path = cache_path()?;
        if !path.exists() {
            return Ok(Self {
                accounts: Vec::new(),
                default_subscription: None,
            });
        }

        let data = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read token cache: {}", path.display()))?;
        let cache: Self = serde_json::from_str(&data)
            .with_context(|| format!("Failed to parse token cache: {}", path.display()))?;

        debug!("Loaded token cache with {} accounts", cache.accounts.len());
        Ok(cache)
    }

    pub fn save(&self) -> Result<()> {
        let path = cache_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create cache directory: {}", parent.display())
            })?;
        }

        let data = serde_json::to_string_pretty(self)?;
        fs::write(&path, &data)
            .with_context(|| format!("Failed to write token cache: {}", path.display()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }

        debug!("Saved token cache to {}", path.display());
        Ok(())
    }

    pub fn active_account(&self) -> Option<&CachedAccount> {
        if let Some(ref sub_id) = self.default_subscription {
            self.accounts
                .iter()
                .find(|a| a.subscription_id.as_deref() == Some(sub_id))
        } else {
            self.accounts.first()
        }
    }

    pub fn active_account_mut(&mut self) -> Option<&mut CachedAccount> {
        if let Some(ref sub_id) = self.default_subscription {
            self.accounts
                .iter_mut()
                .find(|a| a.subscription_id.as_deref() == Some(sub_id))
        } else {
            self.accounts.first_mut()
        }
    }

    pub fn set_account(&mut self, account: CachedAccount) {
        if let Some(ref sub_id) = account.subscription_id {
            self.accounts
                .retain(|a| a.subscription_id.as_deref() != Some(sub_id));
            self.default_subscription = Some(sub_id.clone());
        }
        self.accounts.insert(0, account);
    }

    pub fn clear(&mut self) {
        self.accounts.clear();
        self.default_subscription = None;
    }
}

impl CachedAccount {
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(exp) => Utc::now() >= exp,
            None => true,
        }
    }

    pub fn has_refresh_token(&self) -> bool {
        self.refresh_token.is_some()
    }
}

fn cache_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Cannot determine home directory")?;
    Ok(home.join(".azure").join(CACHE_FILE))
}
