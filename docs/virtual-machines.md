# Virtual Machines

All 29 `az vm` top-level commands are implemented. Subgroups: [`vm disk`](managed-disks.md) and [`vm nic`](#vm-nic) are implemented; extension, identity, run-command, etc. are not yet implemented.

## Query & Info

| Command | Description |
|---------|-------------|
| `vm list [--resource-group RG]` | List VMs (all or by resource group) |
| `vm show --name NAME --resource-group RG` | Show VM details with instance view |
| `vm get-instance-view --name NAME --resource-group RG` | Get instance view (statuses, agent, disks) |
| `vm list-ip-addresses [--name NAME] [--resource-group RG]` | List public and private IP addresses |
| `vm list-sizes --location LOC` | List available VM sizes in a location |
| `vm list-skus [--location LOC] [--resource-type TYPE] [--size SIZE] [--zone]` | List compute SKUs with filters |
| `vm list-usage --location LOC` | List compute usage/quota for a location |
| `vm list-vm-resize-options --name NAME --resource-group RG` | List available resize options for a VM |

## Lifecycle

| Command | Description |
|---------|-------------|
| `vm create --name NAME --resource-group RG --image IMAGE [--size SIZE] [--location LOC] [--admin-username USER] [--admin-password PWD] [--ssh-key-value KEY] [--subnet ID]` | Create a VM |
| `vm delete --name NAME --resource-group RG [--force-deletion] [--no-wait]` | Delete a VM |
| `vm update --name NAME --resource-group RG --set KEY=VAL ...` | Update VM properties via dot-path assignments |
| `vm resize --name NAME --resource-group RG --size SIZE` | Resize a VM |

## Power Management

| Command | Description |
|---------|-------------|
| `vm start --name NAME --resource-group RG` | Start a VM |
| `vm stop --name NAME --resource-group RG [--no-wait]` | Power off a VM (keeps allocation) |
| `vm deallocate --name NAME --resource-group RG [--no-wait]` | Deallocate a VM (stops billing) |
| `vm restart --name NAME --resource-group RG [--no-wait]` | Restart a VM |

## Maintenance & Recovery

| Command | Description |
|---------|-------------|
| `vm redeploy --name NAME --resource-group RG [--no-wait]` | Redeploy a VM to a new host |
| `vm reimage --name NAME --resource-group RG [--no-wait]` | Reimage a VM |
| `vm reapply --name NAME --resource-group RG [--no-wait]` | Reapply VM configuration |
| `vm perform-maintenance --name NAME --resource-group RG` | Perform maintenance on a VM |
| `vm simulate-eviction --name NAME --resource-group RG` | Simulate eviction of a Spot VM |

## Imaging & Disks

| Command | Description |
|---------|-------------|
| `vm generalize --name NAME --resource-group RG` | Generalize a VM for imaging |
| `vm capture --name NAME --resource-group RG --vhd-name-prefix PREFIX [--storage-container NAME] [--overwrite]` | Capture a generalized VM |
| `vm convert --name NAME --resource-group RG` | Convert unmanaged disks to managed |

## Patching

| Command | Description |
|---------|-------------|
| `vm assess-patches --name NAME --resource-group RG` | Assess available patches |
| `vm install-patches --name NAME --resource-group RG --maximum-duration DUR --reboot-setting SETTING` | Install patches on a VM |

## Scheduling & Networking

| Command | Description |
|---------|-------------|
| `vm auto-shutdown --name NAME --resource-group RG [--time HHMM] [--off] [--email ADDR] [--webhook URL]` | Configure auto-shutdown |
| `vm open-port --name NAME --resource-group RG --port PORT [--priority N] [--nsg-name NAME]` | Open inbound port on VM's NSG |

## Polling

| Command | Description |
|---------|-------------|
| `vm wait --name NAME --resource-group RG [--created] [--updated] [--deleted] [--exists]` | Wait for a VM condition |

## vm nic

Manage NIC attachments on a VM. All operations work on `networkProfile.networkInterfaces` of the VM.

| Command | Description |
|---------|-------------|
| `vm nic list --vm-name VM --resource-group RG` | List NICs attached to a VM |
| `vm nic show --vm-name VM --resource-group RG --nic NIC` | Show full NIC details (verifies NIC is attached, then GETs the NIC resource). `NIC` may be a name (resolved in `RG`) or a full resource ID |
| `vm nic add --vm-name VM --resource-group RG --nics NIC [NIC ...] [--primary-nic NAME]` | Append NICs to the VM (skipping any already attached). If no primary is set, the first entry becomes primary |
| `vm nic remove --vm-name VM --resource-group RG --nics NIC [NIC ...] [--primary-nic NAME]` | Detach NICs from the VM |
| `vm nic set --vm-name VM --resource-group RG --nics NIC [NIC ...] [--primary-nic NAME]` | Replace the entire NIC list on the VM |

### Notes

- Exactly one NIC must be marked primary. If `--primary-nic` is not provided and no existing entry is primary, the first NIC is marked primary automatically
- `add` / `remove` / `set` issue a VM PATCH on `networkProfile.networkInterfaces` and typically require the VM to be deallocated
- NIC names are case-insensitive and resolved in the VM's resource group when a bare name is supplied
