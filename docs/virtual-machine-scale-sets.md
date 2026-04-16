# Virtual Machine Scale Sets

Partial coverage — core VMSS commands are implemented. Subgroups (disk, extension, identity, nic, rolling-upgrade, run-command, etc.) are not yet implemented.

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
