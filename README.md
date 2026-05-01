# azcli

A fast Rust implementation of the Azure CLI with native OAuth2 authentication, Azure Bastion tunneling, and progressive coverage of core `az` commands.

## Quick Start

```bash
cargo build --release

azcli login
azcli vm list -o table
azcli group list -o table
```

## Prerequisites

- Rust 2024 edition (1.85+)
- Azure subscription

## Command Coverage

| Command Group | Status | Commands | Details |
|---------------|--------|----------|---------|
| [`login` / `logout` / `account`](docs/authentication.md) | Full | 3 | Native OAuth2 (browser, device code, service principal, managed identity) |
| [`account`](docs/account.md) | Full | 6 | show, list, set, list-locations, get-access-token, clear |
| [`group`](docs/resource-groups.md) | Full | 2 | List and show resource groups |
| [`vm`](docs/virtual-machines.md) | Full (top-level) | 29 | All top-level commands. Subgroups: `vm disk`, `vm nic`, `vm run-command` implemented; extension, identity not yet |
| [`vm disk`](docs/managed-disks.md) | Full | 2 | Attach and detach data disks |
| [`vm nic`](docs/virtual-machines.md#vm-nic) | Full | 5 | List, show, add, remove, set NIC attachments |
| [`vm run-command`](docs/virtual-machines.md#vm-run-command) | Full | 6 | Invoke, list, show, create, update, delete |
| [`disk`](docs/managed-disks.md) | Full | 8 | List, show, list-skus, create, update, delete, grant-access, revoke-access |
| [`image`](docs/image.md) | Read | 5 | list, show; `image builder` list, show, show-runs |
| [`sig`](docs/sig.md) | Read | 18 | Galleries, image definitions, image versions (incl. shared & community) |
| [`vmss`](docs/virtual-machine-scale-sets.md) | Partial | 11 | Core commands. Subgroups not yet implemented |
| [`deployment`](docs/deployments.md) | Full | 44 | All four ARM scopes (group, sub, mg, tenant) + operations |
| [`network bastion`](docs/bastion.md) | Full | 9 | SSH, RDP, tunnel with custom WebSocket implementation |
| [`network`](docs/network.md) | Read | 75+ | **Tier 1 (7 subgroups)**: vnet, nsg, public-ip, nic, private-endpoint, lb, route-table. **Tier 2 (5 subgroups)**: dns, watcher, application-gateway, nat, private-dns. **Tier 3 (4 subgroups)**: vpn-gateway, express-route, traffic-manager, firewall. Includes 33 nested sub-subcommands |
| [`role assignment`](docs/role.md#role-assignment) | Read | 2 | List, show |
| [`role definition`](docs/role.md#role-definition) | Read | 2 | List, show |
| [`role pim`](docs/pim.md) | **NEW** | 4 | List, status, activate, deactivate PIM role assignments |
| [`rest`](docs/rest.md) | Full | 1 | Arbitrary ARM API requests |

## Output Formats

All commands support `-o/--output` with formats matching `az` CLI:

| Format | Description |
|--------|-------------|
| `json` | Pretty JSON (default) |
| `jsonc` | Colorized JSON |
| `table` | Human-readable table |
| `tsv` | Tab-separated values |
| `yaml` / `yamlc` | YAML (plain / colorized) |
| `none` | Suppress output |

## Global Flags

| Flag | Description |
|------|-------------|
| `-o, --output` | Output format |
| `--subscription` | Override subscription. Accepts subscription ID, display name, or profile name |
| `--profile` | Select a named profile (mutually exclusive with `--subscription`) |
| `--query` | JMESPath query string applied to the result before formatting (matches `az --query`) |

### Multi-account profiles

Each `azcli login` caches `(tenant, subscription, refresh_token)`. Tag a login with `--name` to address it later by a friendly name instead of a GUID — useful when juggling multiple tenants/subscriptions (e.g. running `network bastion ssh` against a different account than your default).

```bash
azcli login --name work
azcli login --name personal --tenant <other-tenant-id>

azcli account list -o table             # 'Profile' column shows your names
azcli --profile work account show
azcli --profile personal network bastion ssh --name bastion -g rg --target-resource-id <vm-id>

azcli account set work                  # make 'work' the default
```

`--subscription` is polymorphic and accepts any of: subscription GUID, subscription display name, or profile name. `--profile` is provided as an explicit, unambiguous alternative.

### `--query` examples

```bash
# Project specific fields and rename them
azcli vm list -g my-rg --query "[].{name:name, size:hardwareProfile.vmSize}" -o table

# Filter with starts_with
azcli vm list -g my-rg --query "[?starts_with(name,'web')].name" -o tsv

# Pipe + first element
azcli vmss list-instances -g my-rg -n my-vmss \
  --query "[].{id:instanceId, state:instanceView.statuses[?starts_with(code,'PowerState/')].displayStatus | [0]}" \
  -o table

# Scalars and aggregates
azcli account list-locations --query "length([?metadata.regionType=='Physical'])"
```

JMESPath 1.0 spec, dialect-compatible with the Python `jmespath` library used by `az`.

## Debug Logging

```bash
RUST_LOG=debug azcli vm list
RUST_LOG=azcli=debug azcli network bastion ssh ...
```

## License

Private.
