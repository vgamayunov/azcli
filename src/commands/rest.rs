use anyhow::{Context, Result};
use reqwest::Client;
use tracing::debug;

const ARM_ENDPOINT: &str = "https://management.azure.com";

pub async fn execute(
    access_token: Option<&str>,
    url: &str,
    method: &str,
    body: Option<&str>,
    headers: Option<&[(String, String)]>,
    uri_parameters: Option<&[(String, String)]>,
    skip_authorization_header: bool,
    subscription_id: Option<&str>,
) -> Result<serde_json::Value> {
    let full_url = resolve_url(url, subscription_id);

    let mut final_url = reqwest::Url::parse(&full_url)
        .with_context(|| format!("Invalid URL: {full_url}"))?;

    if let Some(params) = uri_parameters {
        for (k, v) in params {
            final_url.query_pairs_mut().append_pair(k, v);
        }
    }

    debug!("{} {}", method.to_uppercase(), final_url);

    let client = Client::new();
    let mut builder = match method.to_lowercase().as_str() {
        "get" => client.get(final_url),
        "post" => client.post(final_url),
        "put" => client.put(final_url),
        "patch" => client.patch(final_url),
        "delete" => client.delete(final_url),
        "head" => client.head(final_url),
        other => anyhow::bail!("Unsupported HTTP method: {other}"),
    };

    if !skip_authorization_header {
        if let Some(token) = access_token {
            builder = builder.bearer_auth(token);
        }
    }

    let mut has_content_type = false;
    if let Some(hdrs) = headers {
        for (k, v) in hdrs {
            if k.eq_ignore_ascii_case("content-type") {
                has_content_type = true;
            }
            builder = builder.header(k.as_str(), v.as_str());
        }
    }

    if let Some(body_str) = body {
        let body_content = if body_str.starts_with('@') {
            std::fs::read_to_string(&body_str[1..])
                .with_context(|| format!("Failed to read body file: {}", &body_str[1..]))?
        } else {
            body_str.to_string()
        };

        if !has_content_type && serde_json::from_str::<serde_json::Value>(&body_content).is_ok() {
            builder = builder.header("Content-Type", "application/json");
        }

        builder = builder.body(body_content);
    }

    let resp = builder.send().await.context("Request failed")?;
    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        anyhow::bail!("Request failed ({status}): {body_text}");
    }

    if body_text.is_empty() {
        return Ok(serde_json::Value::Null);
    }

    Ok(serde_json::from_str(&body_text).unwrap_or_else(|_| serde_json::Value::String(body_text)))
}

fn resolve_url(url: &str, subscription_id: Option<&str>) -> String {
    let expanded = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else if url.starts_with('/') {
        format!("{ARM_ENDPOINT}{url}")
    } else {
        format!("{ARM_ENDPOINT}/{url}")
    };

    match subscription_id {
        Some(sub) => expanded.replace("{subscriptionId}", sub),
        None => expanded,
    }
}
