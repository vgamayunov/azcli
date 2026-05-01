use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use anyhow::{Context, Result, bail};
use base64::Engine;
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use tracing::debug;

use super::cache::{AuthMethod, CachedAccount};
use super::{
    AZURE_CLI_CLIENT_ID, COMMON_TENANT, MANAGEMENT_SCOPE, OAuthErrorResponse,
    OAuthTokenResponse, authorize_endpoint, token_endpoint,
};

pub async fn login(tenant: Option<&str>) -> Result<CachedAccount> {
    let tenant = tenant.unwrap_or(COMMON_TENANT);
    let (code_verifier, code_challenge) = generate_pkce();

    let listener = TcpListener::bind("127.0.0.1:0").context("Failed to bind localhost for redirect")?;
    let port = listener.local_addr()?.port();
    let redirect_uri = format!("http://localhost:{port}");

    let state = uuid::Uuid::new_v4().to_string();

    let auth_url = format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256&prompt=select_account",
        authorize_endpoint(tenant),
        AZURE_CLI_CLIENT_ID,
        urlencoding(&redirect_uri),
        urlencoding(MANAGEMENT_SCOPE),
        urlencoding(&state),
        urlencoding(&code_challenge),
    );

    eprintln!("Opening browser for login...");
    eprintln!("If the browser does not open, visit:\n{auth_url}");

    if open::that(&auth_url).is_err() {
        eprintln!("Failed to open browser automatically.");
    }

    debug!("Waiting for redirect on port {port}...");

    let (stream, _) = listener.accept().context("Failed to accept redirect connection")?;
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;

    let path = request_line
        .split_whitespace()
        .nth(1)
        .context("Invalid HTTP request from redirect")?;

    let full_url = format!("http://localhost:{port}{path}");
    let parsed = url::Url::parse(&full_url).context("Failed to parse redirect URL")?;

    let mut code = None;
    let mut returned_state = None;
    let mut error = None;
    let mut error_description = None;

    for (key, value) in parsed.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.to_string()),
            "state" => returned_state = Some(value.to_string()),
            "error" => error = Some(value.to_string()),
            "error_description" => error_description = Some(value.to_string()),
            _ => {}
        }
    }

    let response_html = if error.is_some() {
        "<html><body><h2>Login failed.</h2><p>You can close this window.</p></body></html>"
    } else {
        "<html><body><h2>Login successful!</h2><p>You can close this window.</p></body></html>"
    };

    let http_response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response_html.len(),
        response_html
    );
    let mut writer = stream;
    let _ = writer.write_all(http_response.as_bytes());
    let _ = writer.flush();

    if let Some(err) = error {
        let desc = error_description.unwrap_or_default();
        bail!("Authentication failed: {err}: {desc}");
    }

    let code = code.context("No authorization code received")?;

    if returned_state.as_deref() != Some(&state) {
        bail!("OAuth state mismatch — possible CSRF attack");
    }

    debug!("Received authorization code, exchanging for token...");

    let client = reqwest::Client::new();
    let resp = client
        .post(token_endpoint(tenant))
        .form(&[
            ("client_id", AZURE_CLI_CLIENT_ID),
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", &redirect_uri),
            ("code_verifier", &code_verifier),
            ("scope", MANAGEMENT_SCOPE),
        ])
        .send()
        .await
        .context("Token exchange request failed")?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        if let Ok(err) = serde_json::from_str::<OAuthErrorResponse>(&body) {
            bail!(
                "Token exchange failed: {}: {}",
                err.error,
                err.error_description.unwrap_or_default()
            );
        }
        bail!("Token exchange failed: {body}");
    }

    let token_resp: OAuthTokenResponse = resp.json().await.context("Failed to parse token response")?;

    let expires_at = token_resp
        .expires_in
        .map(|secs| Utc::now() + Duration::seconds(secs));

    debug!(
        "Interactive login successful (token {} chars, refresh_token={})",
        token_resp.access_token.len(),
        token_resp.refresh_token.is_some()
    );

    Ok(CachedAccount {
        auth_method: AuthMethod::InteractiveBrowser,
        tenant_id: tenant.to_string(),
        subscription_id: None,
        subscription_name: None,
        profile: None,
        access_token: Some(token_resp.access_token),
        refresh_token: token_resp.refresh_token,
        expires_at,
        client_id: None,
        client_secret: None,
        client_certificate_path: None,
        managed_identity_client_id: None,
    })
}

fn generate_pkce() -> (String, String) {
    let verifier_bytes: Vec<u8> = (0..32).map(|_| rand_byte()).collect();
    let verifier = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&verifier_bytes);

    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize());

    (verifier, challenge)
}

fn rand_byte() -> u8 {
    let v = uuid::Uuid::new_v4();
    v.as_bytes()[0]
}

fn urlencoding(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
