# azcli

A Rust implementation of the Azure CLI `az network bastion` extension. Provides native client tunneling (SSH, RDP, TCP) through Azure Bastion with full feature parity against the official Python extension.

## Quick Start

```bash
# Build
cargo build --release

# SSH through bastion
azcli network bastion ssh \
  --name mybastion \
  --resource-group my-rg \
  --target-ip-address 10.0.0.4 \
  --username azureuser \
  --auth-type ssh-key \
  --ssh-key ~/.ssh/id_rsa

# TCP tunnel (e.g. for port forwarding)
azcli network bastion tunnel \
  --name mybastion \
  --resource-group my-rg \
  --target-ip-address 10.0.0.4 \
  --resource-port 22 \
  --port 2222
```

## Prerequisites

- Rust 2024 edition (1.85+)
- Azure CLI (`az`) installed and authenticated (`az login`)
- Azure Bastion host with native client tunneling enabled

## Commands

| Command | Description |
|---------|-------------|
| `create` | Create a bastion host with full feature flags (tunneling, IP connect, file copy, Kerberos, session recording, shareable link, network ACLs, zones, tags) |
| `delete` | Delete a bastion host |
| `list` | List bastion hosts by resource group or subscription |
| `show` | Show details of a bastion host |
| `update` | Update bastion host properties (SKU, feature flags, network ACLs, tags) |
| `ssh` | SSH to a target VM through bastion (ssh-key or password auth) |
| `rdp` | RDP to a target VM through bastion (tunnel mode, web mode, or AAD) |
| `tunnel` | Open a generic TCP tunnel through bastion |
| `wait` | Poll bastion provisioning state (created, updated, deleted, exists) |

All commands live under `azcli network bastion <command>`.

## Usage Examples

### SSH with key authentication

```bash
azcli network bastion ssh \
  --name mybastion \
  --resource-group my-rg \
  --target-ip-address 10.0.0.4 \
  --username azureuser \
  --auth-type ssh-key \
  --ssh-key ~/.ssh/id_rsa
```

### SSH with password

```bash
azcli network bastion ssh \
  --name mybastion \
  --resource-group my-rg \
  --target-ip-address 10.0.0.4 \
  --username azureuser \
  --auth-type password
```

### SSH with extra arguments

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

### Update bastion features

```bash
azcli network bastion update \
  --name mybastion \
  --resource-group my-rg \
  --sku premium \
  --enable-tunneling true \
  --session-recording true
```

### Wait for provisioning

```bash
azcli network bastion wait \
  --name mybastion \
  --resource-group my-rg \
  --created \
  --interval 15 \
  --timeout 600
```

### Open a TCP tunnel

```bash
# Opens local port 2222 → target VM port 22 through bastion
azcli network bastion tunnel \
  --name mybastion \
  --resource-group my-rg \
  --target-ip-address 10.0.0.4 \
  --resource-port 22 \
  --port 2222

# Then in another terminal:
ssh azureuser@localhost -p 2222
```

## Debug Logging

```bash
RUST_LOG=debug azcli network bastion ssh ...
RUST_LOG=azcli=debug azcli network bastion ssh ...  # azcli logs only
```

## How It Works

1. Authenticates via `az account get-access-token`
2. Resolves the bastion host through the Azure ARM API
3. Acquires a tunnel token from the bastion data plane (`/api/tokens`)
4. Establishes a WebSocket connection to the bastion endpoint with a manual TLS + HTTP upgrade handshake
5. Relays TCP traffic bidirectionally through WebSocket binary frames
6. Launches the native SSH/RDP client pointing at the local tunnel

The WebSocket layer uses a custom raw frame implementation rather than standard libraries, because Azure Bastion's server has non-standard behavior that off-the-shelf WebSocket clients reject.

## Supported SKUs

| SKU | Tunnel URL Pattern |
|-----|-------------------|
| Developer | `wss://{endpoint}/omni/webtunnel/{token}` |
| QuickConnect | `wss://{endpoint}/omni/webtunnel/{token}` |
| Basic / Standard / Premium | `wss://{endpoint}/webtunnelv2/{token}?X-Node-Id={nodeId}` |

## License

Private.
