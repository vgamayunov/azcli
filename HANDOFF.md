# Session State — azcli (Rust Azure CLI)

## Project Goal

Build a Rust equivalent of the Azure CLI, starting with the bastion extension and progressively adding core commands (login, group, vm). Native OAuth2 auth, no Python dependency for auth flow.

## Current Status: WORKING

- **Build**: `cargo build` — 0 errors (14 dead-code warnings from legacy bastion `execute()` fns)
- **Last commit**: `67e0029` — `feat: add resource group and VM commands (list, show, start, stop)`
- **All commands tested end-to-end against live Azure subscription**

## Implemented Commands

```
azcli login [--use-device-code] [--service-principal --client-id --client-secret --tenant] [--identity [--client-id]]
azcli logout
azcli account show
azcli group list [--resource-group RG] [-o table|json|jsonc|tsv|yaml|yamlc|none]
azcli group show --name NAME [-o ...]
azcli vm list [--resource-group RG] [-o ...]
azcli vm show --name NAME --resource-group RG [-o ...]
azcli vm start --name NAME --resource-group RG
azcli vm stop --name NAME --resource-group RG [--no-wait] [--skip-shutdown]
azcli network bastion {create|delete|list|show|update|ssh|rdp|tunnel|wait}
```

Global flags: `-o/--output` (json/jsonc/table/tsv/yaml/yamlc/none), `--subscription`

## Architecture

```
src/
├── main.rs              # CLI entry (clap derive), command routing for all groups
├── api_client.rs        # BastionClient — ARM API + bastion data plane
├── arm_client.rs        # ArmClient — general ARM API client (group, vm methods)
├── auth/
│   ├── mod.rs           # Constants, OAuth structs, az CLI fallback
│   ├── cache.rs         # TokenCache at ~/.azure/azcli_tokens.json
│   ├── interactive.rs   # Browser auth code + PKCE
│   ├── device_code.rs   # Device code polling
│   ├── service_principal.rs  # client_credentials grant
│   ├── managed_identity.rs   # IMDS endpoint
│   └── token_provider.rs     # TokenProvider: wraps all flows, strip_subscription_prefix fix
├── models.rs            # All ARM models (BastionHost, ResourceGroup, VirtualMachine, etc.)
├── output.rs            # Format dispatch (json/jsonc/table/tsv/yaml/yamlc/none)
├── tunnel.rs            # Manual WebSocket tunnel (raw frame I/O for Bastion)
└── commands/
    ├── mod.rs
    ├── group/           # Resource group commands
    │   ├── mod.rs
    │   ├── list.rs
    │   └── show.rs
    ├── vm/              # Virtual machine commands
    │   ├── mod.rs
    │   ├── list.rs
    │   ├── show.rs
    │   ├── start.rs
    │   └── stop.rs
    ├── create.rs        # bastion create
    ├── delete.rs        # bastion delete
    ├── list.rs          # bastion list
    ├── show.rs          # bastion show
    ├── update.rs        # bastion update
    ├── ssh.rs           # bastion ssh
    ├── rdp.rs           # bastion rdp
    ├── tunnel.rs        # bastion tunnel
    └── wait.rs          # bastion wait
```

## Key Technical Decisions

### Auth: Fully manual OAuth2 with reqwest (no azure_identity crate)
- Oracle consultation decided: go manual for full control, smaller binary, no dependency sprawl
- Client ID: `04b07795-8ddb-461a-bbee-02f9e1bf7b46` (Azure CLI well-known public client)
- Scope: `https://management.azure.com/.default`
- Token cache: `~/.azure/azcli_tokens.json` with refresh support
- Bastion commands fall back to `az account get-access-token` when not logged in via azcli

### ARM API: Properties flattening for VM output
The ARM API nests VM fields under `properties` (hardwareProfile, provisioningState, vmId, etc.).
`VirtualMachine` model deserializes the raw structure, then `to_flattened_value()` produces
az-cli-compatible flattened JSON (properties promoted to top level, resourceGroup extracted from id).

### Critical bug fix: subscription ID prefix stripping
ARM `/subscriptions` API returns IDs as `/subscriptions/{guid}`. We stored verbatim, causing
double-prefixed URLs (`/subscriptions//subscriptions/{guid}/...`) → 400 errors.
Fixed with `strip_subscription_prefix()` in `token_provider.rs`.

### Raw WebSocket Implementation (tunnel.rs)
Azure Bastion's WebSocket server is non-compliant (missing upgrade headers, RSV1 bit set without negotiation).
Custom raw frame implementation instead of tungstenite.

### Table output column selection (output.rs)
`pick_table_columns()` checks for preferred fields in order:
name, location, resourceGroup, provisioningState, properties.provisioningState, hardwareProfile.vmSize, sku.name, properties.dnsName.
Falls back to first 6 keys if none match.

## Azure Test Environment
- Bastion: `azurebastion` in RG `access-hub-rg`
- Subscription: `62118f5c-be37-400f-9f20-a8b77a2a7877`
- Target VM: IP `10.1.20.110`, port 22, user `victor.gamayunov`, SSH key `~/.ssh/id_rsa`
- SKU: Standard (uses `/webtunnelv2/` URL pattern)

## Commit History
- `97d6de8` — feat: Rust Azure CLI with bastion extension and -o output format support
- `a0a0b47` — feat: add native OAuth2 login and fix subscription ID prefix from ARM API
- `67e0029` — feat: add resource group and VM commands (list, show, start, stop)

## Known Issues
- 14 dead-code warnings from old bastion `execute()` functions (cosmetic, can clean up)
- `vm stop` defaults to `deallocate` (releases compute billing); `az vm stop` defaults to `powerOff`. Exposed via `--skip-shutdown` flag but semantics differ from az cli where `az vm stop` = powerOff and `az vm deallocate` = deallocate. May want a separate `vm deallocate` command.
- Azure Bastion server intermittently returns proxy-101-then-400 (not a client bug, Python az CLI has same issue)
- No retry logic for WebSocket connection failures

## Dependencies
```toml
clap = "4"                  # CLI framework
tokio = "1"                 # Async runtime
reqwest = "0.12"            # HTTP client (rustls-tls)
serde/serde_json/serde_yaml # Serialization
tracing/tracing-subscriber  # Logging
anyhow/thiserror            # Error handling
tokio-rustls/rustls/webpki-roots  # TLS for WebSocket
base64/uuid/url/which       # Utilities
sha2/dirs/open/chrono       # Auth module deps
```

## What's Left (Future Work)
- `vm deallocate` as separate command (match az cli semantics)
- Pagination support for list APIs (currently returns first page only)
- Retry logic for WebSocket connection failures
- Tests (unit + integration)
- RDP command end-to-end testing
- Developer SKU endpoint support
- `--no-wait` flag for create/update/delete
- Clean up dead code warnings
- Additional az commands (network, storage, etc.)
