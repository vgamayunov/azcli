# Managed Disks

Two command groups cover managed disks:

- `disk` — top-level managed disk operations (8 commands)
- `vm disk` — attach/detach data disks to a VM (2 commands)

## `disk` — Top-Level

### Query & Info

| Command | Description |
|---------|-------------|
| `disk list [--resource-group RG]` | List managed disks (all or by resource group) |
| `disk show --name NAME --resource-group RG` | Show managed disk details |
| `disk list-skus [--location LOC] [--zone]` | List available disk SKUs (optionally filtered by location / zonal availability) |

### Lifecycle

| Command | Description |
|---------|-------------|
| `disk create --name NAME --resource-group RG [--location LOC] [--size-gb N] [--sku SKU] [--source SRC] [--zone Z] [--hyper-v-generation GEN] [--os-type TYPE]` | Create a managed disk (empty or from a source) |
| `disk update --name NAME --resource-group RG [--size-gb N] [--sku SKU]` | Update disk size or SKU |
| `disk delete --name NAME --resource-group RG` | Delete a managed disk |

### SAS Access

| Command | Description |
|---------|-------------|
| `disk grant-access --name NAME --resource-group RG [--access-level LEVEL] [--duration-in-seconds N]` | Grant time-bound SAS access (default: `Read`, 3600s). Polls the long-running operation and returns the SAS URL |
| `disk revoke-access --name NAME --resource-group RG` | Revoke an existing SAS access |

## `vm disk` — VM Data Disk Operations

| Command | Description |
|---------|-------------|
| `vm disk attach --vm-name VM --resource-group RG --name DISK [--new] [--size-gb N] [--sku SKU] [--lun N] [--caching MODE] [--enable-write-accelerator]` | Attach an existing disk to a VM, or create and attach a new disk with `--new`. LUN auto-assigned if omitted |
| `vm disk detach --vm-name VM --resource-group RG --name DISK [--force-detach]` | Detach a data disk from a VM. `--force-detach` issues a force-detach on a stuck disk |

## Notes

- Disk API version: `2023-04-02` (`Microsoft.Compute/disks`)
- SKU listing uses `Microsoft.Compute/skus` (api-version `2021-07-01`) filtered to `resourceType eq 'disks'`
- `grant-access` follows the standard ARM long-running-operation pattern (202 → poll `Azure-AsyncOperation` / `Location` → 200 with SAS URL)
- `vm disk attach` / `detach` are implemented as PATCH on the VM's `storageProfile.dataDisks`. Force-detach sets `toBeDetached: true` and `detachOption: ForceDetach`
