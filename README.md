# azcli

A fast Rust implementation of the Azure CLI with native OAuth2 authentication, Azure Bastion tunneling, and progressive coverage of core `az` commands.

## Quick Start

```bash
# Build
cargo build --release

# Login (interactive browser)
azcli login

# List resource groups
azcli group list -o table

# List VMs
azcli vm list -o table

# Show VM details with power state
azcli vm show --name myvm --resource-group my-rg

# SSH through bastion
azcli network bastion ssh \
  --name mybastion \
  --resource-group my-rg \
  --target-ip-address 10.0.0.4 \
  --username azureuser \
  --auth-type ssh-key \
  --ssh-key ~/.ssh/id_rsa
```

## Prerequisites

- Rust 2024 edition (1.85+)
- Azure subscription

## Authentication

Native OAuth2 implementation — no dependency on `az` CLI for auth.

```bash
# Interactive browser login (default)
azcli login

# Device code flow (for headless/SSH sessions)
azcli login --use-device-code

# Service principal
azcli login --service-principal --tenant TENANT --client-id ID --client-secret SECRET

# Managed identity (from Azure VM/Container)
azcli login --identity

# Check current account
azcli account show

# Logout
azcli logout
```

Tokens are cached at `~/.azure/azcli_tokens.json` with automatic refresh.

## Commands

### Account

| Command | Description |
|---------|-------------|
| `login` | Authenticate (browser, device code, service principal, managed identity) |
| `logout` | Clear cached tokens |
| `account show` | Show current subscription and account info |

### Resource Groups

| Command | Description |
|---------|-------------|
| `group list` | List resource groups in current subscription |
| `group show --name NAME` | Show details of a resource group |

### Virtual Machines

| Command | Description |
|---------|-------------|
| `vm list [--resource-group RG]` | List VMs (all or by resource group) |
| `vm show --name NAME --resource-group RG` | Show VM details with instance view |
| `vm start --name NAME --resource-group RG` | Start a VM |
| `vm stop --name NAME --resource-group RG [--no-wait]` | Power off a VM (keeps allocation) |
| `vm deallocate --name NAME --resource-group RG [--no-wait]` | Deallocate a VM (stops billing) |

### Virtual Machine Scale Sets

| Command | Description |
|---------|-------------|
| `vmss list [--resource-group RG]` | List VMSS (all or by resource group) |
| `vmss show --name NAME --resource-group RG` | Show VMSS details |
| `vmss list-instances --name NAME --resource-group RG [--expand]` | List instances in a VMSS |
| `vmss list-skus --name NAME --resource-group RG` | List available SKUs for a VMSS |
| `vmss list-instance-public-ips --name NAME --resource-group RG` | List public IPs of VMSS instances |
| `vmss list-instance-connection-info --name NAME --resource-group RG` | List NIC info for VMSS instances |
| `vmss scale --name NAME --resource-group RG --new-capacity N` | Scale VMSS to N instances |
| `vmss start --name NAME --resource-group RG [--instance-ids ...]` | Start VMSS instances |
| `vmss stop --name NAME --resource-group RG [--instance-ids ...]` | Stop VMSS instances |
| `vmss update-instances --name NAME --resource-group RG --instance-ids ...` | Manually upgrade instances |
| `vmss wait --name NAME --resource-group RG --created/--updated/--deleted/--exists` | Poll VMSS provisioning state |

### Deployments

All commands under `azcli deployment group <command>` or `azcli deployment operation group <command>`.

| Command | Description |
|---------|-------------|
| `deployment group list --resource-group RG` | List deployments in a resource group |
| `deployment group show --name NAME --resource-group RG` | Show deployment details |
| `deployment group export --name NAME --resource-group RG` | Export the template used for a deployment |
| `deployment group validate --resource-group RG -f FILE` | Validate a template without deploying |
| `deployment group what-if --resource-group RG -f FILE` | Preview changes a deployment would make |
| `deployment group cancel --name NAME --resource-group RG` | Cancel a running deployment |
| `deployment group wait --name NAME --resource-group RG --created/--updated/--deleted/--exists` | Poll deployment state |
| `deployment operation group list --name NAME --resource-group RG` | List operations for a deployment |
| `deployment operation group show --name NAME --resource-group RG --operation-id ID` | Show a specific deployment operation |

### Azure Bastion

All commands under `azcli network bastion <command>`.

| Command | Description |
|---------|-------------|
| `ssh` | SSH to a target VM through bastion (ssh-key, password, or AAD) |
| `rdp` | RDP to a target VM through bastion (tunnel, web, or AAD mode) |
| `tunnel` | Open a generic TCP tunnel through bastion |
| `create` | Create a bastion host with full feature flags |
| `delete` | Delete a bastion host |
| `list` | List bastion hosts by resource group or subscription |
| `show` | Show bastion host details |
| `update` | Update bastion host properties |
| `wait` | Poll bastion provisioning state |

## Output Formats

All commands support `-o/--output` with formats matching `az` CLI:

```bash
azcli vm list -o table         # Human-readable table
azcli vm list -o json          # Pretty JSON (default)
azcli vm list -o jsonc         # Colorized JSON
azcli vm list -o tsv           # Tab-separated values
azcli vm list -o yaml          # YAML
azcli vm list -o none          # Suppress output
```

## Global Flags

| Flag | Description |
|------|-------------|
| `-o, --output` | Output format (json, jsonc, table, tsv, yaml, yamlc, none) |
| `--subscription` | Override subscription ID |

## Usage Examples

### SSH through bastion with port forwarding

```bash
azcli network bastion ssh \
  --name mybastion \
  --resource-group my-rg \
  --target-ip-address 10.0.0.4 \
  --username azureuser \
  --auth-type ssh-key \
  --ssh-key ~/.ssh/id_rsa \
  -- -L 8080:localhost:80
```

### TCP tunnel through bastion

```bash
azcli network bastion tunnel \
  --name mybastion \
  --resource-group my-rg \
  --target-ip-address 10.0.0.4 \
  --resource-port 22 \
  --port 2222

# Then in another terminal:
ssh azureuser@localhost -p 2222
```

### Create a bastion host

```bash
azcli network bastion create \
  --name mybastion \
  --resource-group my-rg \
  --location eastus \
  --vnet-name my-vnet \
  --sku standard \
  --enable-tunneling \
  --enable-ip-connect
```

## Debug Logging

```bash
RUST_LOG=debug azcli vm list
RUST_LOG=azcli=debug azcli network bastion ssh ...
```

## How Bastion Tunneling Works

1. Authenticates via native OAuth2 (or falls back to `az account get-access-token`)
2. Resolves the bastion host through the ARM API
3. Acquires a tunnel token from the bastion data plane (`/api/tokens`)
4. Establishes a WebSocket connection with a manual TLS + HTTP upgrade handshake
5. Relays TCP traffic bidirectionally through WebSocket binary frames
6. Launches the native SSH/RDP client pointing at the local tunnel

The WebSocket layer uses a custom raw frame implementation because Azure Bastion's server has non-standard behavior that off-the-shelf WebSocket clients reject.

### Supported Bastion SKUs

| SKU | Tunnel URL Pattern |
|-----|-------------------|
| Developer / QuickConnect | `wss://{endpoint}/omni/webtunnel/{token}` |
| Basic / Standard / Premium | `wss://{endpoint}/webtunnelv2/{token}?X-Node-Id={nodeId}` |

## License

Private.
