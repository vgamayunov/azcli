# az network

Read-only commands for core Azure networking resources (`Microsoft.Network` v2023-11-01): virtual networks (with subnets and peerings), network security groups (with security rules), public IP addresses, network interfaces (with IP configurations), and private endpoints.

Create / update / delete operations are intentionally not implemented in this pass. `network bastion` (full implementation, including SSH/RDP/tunnel) is documented separately in [bastion.md](bastion.md).

## Commands

### `network vnet`

| Command | Description |
|---|---|
| `network vnet list [-g <rg>]` | List virtual networks in subscription or resource group |
| `network vnet show -g <rg> -n <vnet>` | Show a virtual network |

### `network vnet subnet`

| Command | Description |
|---|---|
| `network vnet subnet list -g <rg> --vnet-name <vnet>` | List subnets in a vnet |
| `network vnet subnet show -g <rg> --vnet-name <vnet> -n <subnet>` | Show a subnet |

### `network vnet peering`

| Command | Description |
|---|---|
| `network vnet peering list -g <rg> --vnet-name <vnet>` | List peerings on a vnet |
| `network vnet peering show -g <rg> --vnet-name <vnet> -n <peering>` | Show a peering |

### `network nsg`

| Command | Description |
|---|---|
| `network nsg list [-g <rg>]` | List network security groups |
| `network nsg show -g <rg> -n <nsg>` | Show a network security group |

### `network nsg rule`

| Command | Description |
|---|---|
| `network nsg rule list -g <rg> --nsg-name <nsg>` | List security rules in an NSG |
| `network nsg rule show -g <rg> --nsg-name <nsg> -n <rule>` | Show a security rule |

### `network public-ip`

| Command | Description |
|---|---|
| `network public-ip list [-g <rg>]` | List public IP addresses |
| `network public-ip show -g <rg> -n <pip>` | Show a public IP address |

### `network nic`

| Command | Description |
|---|---|
| `network nic list [-g <rg>]` | List network interfaces |
| `network nic show -g <rg> -n <nic>` | Show a network interface |

### `network nic ip-config`

| Command | Description |
|---|---|
| `network nic ip-config list -g <rg> --nic-name <nic>` | List IP configurations on a NIC |
| `network nic ip-config show -g <rg> --nic-name <nic> -n <ipconfig>` | Show an IP configuration |

### `network private-endpoint`

| Command | Description |
|---|---|
| `network private-endpoint list [-g <rg>]` | List private endpoints |
| `network private-endpoint show -g <rg> -n <pe>` | Show a private endpoint |

## Output

All commands honor the global `-o/--output` flag. Tables show the most useful columns per resource type (name, location/RG, key state fields, provisioning state). Use `-o json` / `-o jsonc` for the full ARM payload.

## Not Implemented

Out of scope for this read-only pass: route tables, load balancers, application gateways, private link services, private endpoint connections, public IP prefixes, NIC effective routes / NSG, and `vnet list-available-ips`.
