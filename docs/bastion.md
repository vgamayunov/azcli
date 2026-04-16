# Azure Bastion

Full coverage of all `az network bastion` commands, plus native SSH/RDP tunneling through bastion with a custom WebSocket implementation.

## Commands

| Command | Description |
|---------|-------------|
| `network bastion ssh` | SSH to a target VM through bastion (ssh-key, password, or AAD) |
| `network bastion rdp` | RDP to a target VM through bastion (tunnel, web, or AAD mode) |
| `network bastion tunnel` | Open a generic TCP tunnel through bastion |
| `network bastion create` | Create a bastion host with full feature flags |
| `network bastion delete` | Delete a bastion host |
| `network bastion list` | List bastion hosts by resource group or subscription |
| `network bastion show` | Show bastion host details |
| `network bastion update` | Update bastion host properties |
| `network bastion wait` | Poll bastion provisioning state |

## Examples

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
