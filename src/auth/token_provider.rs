use anyhow::{Context, Result, bail};
use chrono::{Duration, Utc};
use tracing::{debug, info, warn};

use super::cache::{AuthMethod, CachedAccount, TokenCache};
use super::{
    AZURE_CLI_CLIENT_ID, MANAGEMENT_SCOPE, OAuthErrorResponse, OAuthTokenResponse,
    list_subscriptions, token_endpoint,
};

pub struct TokenProvider {
    cache: TokenCache,
    subscription_override: Option<String>,
}

impl TokenProvider {
    pub fn load(subscription_override: Option<String>) -> Result<Self> {
        let cache = TokenCache::load()?;
        Ok(Self {
            cache,
            subscription_override,
        })
    }

    pub fn cache_default_subscription(&self) -> Option<String> {
        self.subscription_override
            .clone()
            .or_else(|| self.cache.default_subscription.clone())
    }

    pub fn cache(&self) -> &TokenCache {
        &self.cache
    }

    pub fn cache_mut(&mut self) -> &mut TokenCache {
        &mut self.cache
    }

    pub fn save_cache(&self) -> Result<()> {
        self.cache.save()
    }

    pub async fn get_access_token(&mut self) -> Result<String> {
        let sub_override = self.subscription_override.clone();

        let account = if let Some(ref sub_id) = sub_override {
            self.cache
                .accounts
                .iter()
                .find(|a| a.subscription_id.as_deref() == Some(sub_id))
                .or(self.cache.accounts.first())
        } else {
            self.cache.active_account()
        };

        let account = match account {
            Some(a) => a.clone(),
            None => return self.fallback_az_cli().await,
        };

        if !account.is_expired() {
            if let Some(ref token) = account.access_token {
                debug!("Using cached access token ({} chars)", token.len());
                return Ok(token.clone());
            }
        }

        if account.has_refresh_token() {
            debug!("Access token expired, attempting refresh...");
            match self.refresh_token(&account).await {
                Ok(token) => return Ok(token),
                Err(e) => {
                    warn!("Token refresh failed: {e:#}. Trying re-acquisition...");
                }
            }
        }

        match account.auth_method {
            AuthMethod::ServicePrincipalSecret => {
                if let (Some(cid), Some(secret)) =
                    (&account.client_id, &account.client_secret)
                {
                    let new_account =
                        super::service_principal::login(&account.tenant_id, cid, secret).await?;
                    let token = new_account
                        .access_token
                        .clone()
                        .context("No token from service principal re-auth")?;
                    self.update_account(new_account)?;
                    return Ok(token);
                }
            }
            AuthMethod::ManagedIdentity => {
                let new_account = super::managed_identity::login(
                    account.managed_identity_client_id.as_deref(),
                )
                .await?;
                let token = new_account
                    .access_token
                    .clone()
                    .context("No token from managed identity re-auth")?;
                self.update_account(new_account)?;
                return Ok(token);
            }
            _ => {}
        }

        warn!("Cached credentials expired and cannot be refreshed. Falling back to az CLI.");
        self.fallback_az_cli().await
    }

    pub fn get_subscription_id(&self) -> Result<String> {
        if let Some(ref sub_id) = self.subscription_override {
            return Ok(sub_id.clone());
        }

        if let Some(ref sub_id) = self.cache.default_subscription {
            return Ok(sub_id.clone());
        }

        if let Some(account) = self.cache.active_account() {
            if let Some(ref sub_id) = account.subscription_id {
                return Ok(sub_id.clone());
            }
        }

        bail!(
            "No subscription configured. Run 'azcli login' or pass --subscription."
        );
    }

    pub async fn get_subscription_id_or_fallback(&mut self) -> Result<String> {
        match self.get_subscription_id() {
            Ok(id) => Ok(id),
            Err(_) => super::get_subscription_id_az_cli().await,
        }
    }

    pub async fn login_interactive(&mut self, tenant: Option<&str>) -> Result<()> {
        let mut account = super::interactive::login(tenant).await?;
        self.resolve_subscription(&mut account).await?;
        self.cache.set_account(account);
        self.cache.save()?;
        self.print_login_info();
        Ok(())
    }

    pub async fn login_device_code(&mut self, tenant: Option<&str>) -> Result<()> {
        let mut account = super::device_code::login(tenant).await?;
        self.resolve_subscription(&mut account).await?;
        self.cache.set_account(account);
        self.cache.save()?;
        self.print_login_info();
        Ok(())
    }

    pub async fn login_service_principal(
        &mut self,
        tenant: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Result<()> {
        let mut account =
            super::service_principal::login(tenant, client_id, client_secret).await?;
        self.resolve_subscription(&mut account).await?;
        self.cache.set_account(account);
        self.cache.save()?;
        self.print_login_info();
        Ok(())
    }

    pub async fn login_managed_identity(&mut self, client_id: Option<&str>) -> Result<()> {
        let mut account = super::managed_identity::login(client_id).await?;
        self.resolve_subscription(&mut account).await?;
        self.cache.set_account(account);
        self.cache.save()?;
        self.print_login_info();
        Ok(())
    }

    pub fn show_account(&self) -> Result<()> {
        let account = self
            .cache
            .active_account()
            .context("Not logged in. Run 'azcli login'.")?;

        let sub_display = account
            .subscription_name
            .as_deref()
            .unwrap_or("(unknown)");
        let sub_id = account.subscription_id.as_deref().unwrap_or("(none)");

        eprintln!("Logged in as: {:?}", account.auth_method);
        eprintln!("Tenant:       {}", account.tenant_id);
        eprintln!("Subscription: {sub_display} ({sub_id})");

        if let Some(ref exp) = account.expires_at {
            let now = Utc::now();
            if *exp > now {
                let remaining = *exp - now;
                eprintln!("Token expires: in {} minutes", remaining.num_minutes());
            } else {
                eprintln!("Token expires: EXPIRED (will refresh on next use)");
            }
        }

        Ok(())
    }

    pub fn logout(&mut self) -> Result<()> {
        self.cache.clear();
        self.cache.save()?;
        info!("Logged out. Token cache cleared.");
        Ok(())
    }

    async fn refresh_token(&mut self, account: &CachedAccount) -> Result<String> {
        let refresh_token = account
            .refresh_token
            .as_deref()
            .context("No refresh token")?;

        let client = reqwest::Client::new();
        let resp = client
            .post(token_endpoint(&account.tenant_id))
            .form(&[
                ("client_id", AZURE_CLI_CLIENT_ID),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("scope", MANAGEMENT_SCOPE),
            ])
            .send()
            .await
            .context("Token refresh request failed")?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            if let Ok(err) = serde_json::from_str::<OAuthErrorResponse>(&body) {
                bail!(
                    "Token refresh failed: {}: {}",
                    err.error,
                    err.error_description.unwrap_or_default()
                );
            }
            bail!("Token refresh failed: {body}");
        }

        let token_resp: OAuthTokenResponse =
            resp.json().await.context("Failed to parse refresh response")?;

        let new_token = token_resp.access_token.clone();

        if let Some(acct) = self.cache.active_account_mut() {
            acct.access_token = Some(token_resp.access_token);
            if let Some(rt) = token_resp.refresh_token {
                acct.refresh_token = Some(rt);
            }
            acct.expires_at = token_resp
                .expires_in
                .map(|secs| Utc::now() + Duration::seconds(secs));
        }

        self.cache.save()?;
        debug!("Token refreshed successfully");
        Ok(new_token)
    }

    async fn resolve_subscription(&self, account: &mut CachedAccount) -> Result<()> {
        let token = match account.access_token {
            Some(ref t) => t.clone(),
            None => return Ok(()),
        };

        match list_subscriptions(&token).await {
            Ok(subs) => {
                if subs.is_empty() {
                    warn!("No subscriptions found for this account");
                    return Ok(());
                }

                if subs.len() == 1 {
                    let sub = &subs[0];
                    account.subscription_id = Some(strip_subscription_prefix(&sub.id));
                    account.subscription_name = sub.display_name.clone();
                    if let Some(ref tid) = sub.tenant_id {
                        account.tenant_id = tid.clone();
                    }
                    return Ok(());
                }

                eprintln!("\nAvailable subscriptions:");
                for (i, sub) in subs.iter().enumerate() {
                    let name = sub.display_name.as_deref().unwrap_or("(unnamed)");
                    let state = sub.state.as_deref().unwrap_or("");
                    let sub_id = strip_subscription_prefix(&sub.id);
                    eprintln!("  [{}] {} ({}) - {}", i + 1, name, sub_id, state);
                }

                let sub = &subs[0];
                account.subscription_id = Some(strip_subscription_prefix(&sub.id));
                account.subscription_name = sub.display_name.clone();
                if let Some(ref tid) = sub.tenant_id {
                    account.tenant_id = tid.clone();
                }

                eprintln!(
                    "\nDefault subscription: {} ({})",
                    sub.display_name.as_deref().unwrap_or("(unnamed)"),
                    strip_subscription_prefix(&sub.id)
                );
            }
            Err(e) => {
                warn!("Could not list subscriptions: {e:#}");
            }
        }

        Ok(())
    }

    fn update_account(&mut self, account: CachedAccount) -> Result<()> {
        self.cache.set_account(account);
        self.cache.save()
    }

    fn print_login_info(&self) {
        if let Some(account) = self.cache.active_account() {
            let sub_display = account
                .subscription_name
                .as_deref()
                .unwrap_or("(unknown)");
            let sub_id = account.subscription_id.as_deref().unwrap_or("(none)");
            eprintln!(
                "\nLogged in successfully.\nSubscription: {sub_display} ({sub_id})"
            );
        }
    }

    async fn fallback_az_cli(&self) -> Result<String> {
        debug!("No cached credentials, falling back to az CLI");
        super::get_access_token_az_cli().await
    }
}

/// ARM API returns subscription IDs as `/subscriptions/{guid}`.
/// Strip the prefix to get the bare GUID for URL construction.
fn strip_subscription_prefix(id: &str) -> String {
    id.strip_prefix("/subscriptions/")
        .unwrap_or(id)
        .to_string()
}
