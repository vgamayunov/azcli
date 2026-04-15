# Session State — azcli (Rust Azure Bastion CLI)

## Project Goal

Build a complete Rust equivalent of the Azure CLI `az network bastion` extension (from Azure/azure-cli-extensions, src/bastion/, v1.4.3). All 9 commands implemented. SSH tunneling works end-to-end.

## Current Status: WORKING

- **Build**: `cargo build` — 0 errors, 0 warnings
- **SSH tunnel**: Tested and working against live Azure Bastion
- **Test command**: `azcli network bastion ssh --name "azurebastion" --resource-group "access-hub-rg" --target-ip-address "10.1.20.110" --username victor.gamayunov --auth-type ssh-key --ssh-key ~/.ssh/id_rsa`

## Architecture

```
src/
├── main.rs           # CLI entry point (clap derive), all 9 command definitions + routing
├── auth.rs           # Azure auth via `az account get-access-token` subprocess
├── api_client.rs     # BastionClient — ARM API + bastion data plane (tokens, RDP, developer endpoint)
├── models.rs         # BastionHost, TokenResponse, BastionSku, AuthType, etc.
├── tunnel.rs         # TunnelServer — TCP listener, manual WebSocket handshake + raw frame I/O
└── commands/
    ├── mod.rs
    ├── create.rs     # PUT bastion host (all feature flags)
    ├── delete.rs     # DELETE bastion host
    ├── list.rs       # LIST by subscription or resource group
    ├── show.rs       # GET single bastion host
    ├── update.rs     # PATCH bastion host (all feature flags)
    ├── ssh.rs        # SSH via bastion tunnel
    ├── rdp.rs        # RDP via tunnel mode, web mode, or AAD
    ├── tunnel.rs     # Generic TCP tunnel through bastion
    └── wait.rs       # Poll provisioning state with configurable interval/timeout
```

## Key Technical Decisions

### Raw WebSocket Implementation (tunnel.rs)
We do NOT use tungstenite. Azure Bastion's WebSocket server has two non-compliance issues:
1. Sometimes omits `Upgrade: websocket` and `Connection: Upgrade` in 101 response
2. Sets RSV1 bit on frames (permessage-deflate indicator) without negotiation

Our implementation:
- Manual TLS connection via `tokio-rustls` + `webpki-roots`
- Raw HTTP upgrade request with `Sec-WebSocket-Key` from UUID v4
- Byte-level HTTP response parsing (scan for `\r\n\r\n`)
- Lenient 101 validation (warns on missing headers, proceeds anyway)
- Custom `WsReader` / `WsWriter` for raw WebSocket frame I/O (ignores RSV bits)
- Detection of HTTP responses masquerading as WebSocket frames (Bastion proxy edge case)

### Auth
Uses `az account get-access-token` subprocess. The access token is passed as an `aztoken` form field to the bastion `/api/tokens` endpoint (NOT as a Bearer header — matches Python implementation).

### Token Flow
1. `az account get-access-token` → ARM bearer token
2. GET bastion host via ARM API → extract `dnsName` (bastion endpoint), SKU
3. POST `https://{bastion_endpoint}/api/tokens` with form data (resourceId, protocol, workloadHostPort, aztoken) → returns authToken, nodeId, websocketToken
4. WebSocket upgrade to `wss://{bastion_endpoint}/webtunnelv2/{websocketToken}?X-Node-Id={nodeId}`
5. Bidirectional TCP ↔ WebSocket binary frame relay

## Azure Test Environment
- Bastion: `azurebastion` in RG `access-hub-rg`
- Subscription: `62118f5c-be37-400f-9f20-a8b77a2a7877`
- Bastion endpoint: `bst-456e5bde-f503-4d2f-8f22-c5846b1a8319.bastion.azure.com`
- Target VM: IP `10.1.20.110`, port 22, user `victor.gamayunov`, SSH key `~/.ssh/id_rsa`
- SKU: Standard (uses `/webtunnelv2/` URL pattern; Developer/QuickConnect use `/omni/webtunnel/`)

## Known Issues

### Azure Bastion Server Instability
The bastion server intermittently:
- Returns HTTP 101 from proxy layer, then immediately sends raw `HTTP/1.1 400 Bad Request` (Tomcat error page) instead of WebSocket frames — proxy upgraded but backend rejected
- Hangs after 101 without sending any data
- Works correctly when the request reaches a healthy backend node

The Python `az` CLI exhibits the same failures during these periods. Not a client bug.

### No Retry Logic Yet
When bastion returns proxy-101-then-400, the connection fails immediately. A retry mechanism with backoff would improve resilience.

## Dependencies
```toml
clap = "4"                  # CLI framework
tokio = "1"                 # Async runtime
reqwest = "0.12"            # HTTP client (rustls-tls)
serde = "1"                 # Serialization
serde_json = "1"
tracing = "0.1"             # Logging
tracing-subscriber = "0.3"
anyhow = "1"                # Error handling
thiserror = "2"
url = "2"                   # URL parsing
base64 = "0.22"             # WebSocket key encoding
uuid = "1"                  # WebSocket key + mask generation
tokio-rustls = "0.26"       # TLS for WebSocket
rustls = "0.23"             # TLS config (aws-lc-rs backend)
webpki-roots = "1"          # Root CA certificates
which = "7"                 # Find ssh/mstsc executables
```

## What's Left (Future Work)
- Retry logic for WebSocket connection failures (with exponential backoff)
- Tests (unit + integration)
- RDP command end-to-end testing (tunnel mode implemented but untested)
- Developer SKU endpoint support (`/api/connection` → `/omni/webtunnel/`)
- `--no-wait` flag for create/update/delete
- Better error messages for common failures (expired token, wrong SKU, etc.)
- Release build optimization

## Reference
- Python source: `azure-cli-extensions` repo, `src/bastion/azext_bastion/`, v1.4.3, commit `a5cf595af28c418938f75ac84f30d90ff5c78ece`
- The `azure-cli/` directory in this repo is a reference clone (not part of the build)
