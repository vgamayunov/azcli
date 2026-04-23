use anyhow::Result;

use crate::auth::token_provider::TokenProvider;

pub fn execute(provider: &mut TokenProvider) -> Result<serde_json::Value> {
    let count = provider.cache().accounts.len();
    provider.cache_mut().clear();
    provider.save_cache()?;
    Ok(serde_json::json!({
        "cleared": count,
        "message": format!("removed {} cached account(s)", count),
    }))
}
