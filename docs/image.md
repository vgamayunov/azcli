# az image

Read-only commands for managed images (`Microsoft.Compute/images`) and Image Builder templates (`Microsoft.VirtualMachineImages/imageTemplates`).

Create / update / delete operations are intentionally not implemented in this pass.

## Commands

| Command | Description |
|---|---|
| `image list [-g <rg>]` | List images in subscription or resource group |
| `image show -g <rg> -n <name>` | Show a specific image |
| `image builder list [-g <rg>]` | List Image Builder templates |
| `image builder show -g <rg> -n <name>` | Show a specific Image Builder template |
| `image builder show-runs -g <rg> -n <name> [--output-name <run>]` | List run outputs for a template, or show a single run output |

## Examples

```bash
# All images in the current subscription
azcli image list -o table

# Images in a resource group
azcli image list -g my-rg -o table

# Show one image
azcli image show -g my-rg -n my-image

# Image Builder templates
azcli image builder list -o table
azcli image builder show -g my-rg -n my-template

# Run outputs (artifacts produced by the most recent build)
azcli image builder show-runs -g my-rg -n my-template
azcli image builder show-runs -g my-rg -n my-template --output-name my-output
```

## API versions

| Resource provider | API version |
|---|---|
| `Microsoft.Compute/images` | `2024-07-01` |
| `Microsoft.VirtualMachineImages/imageTemplates` | `2022-07-01` |

## Notes

- `image builder show-runs` corresponds to the ARM `runOutputs` sub-resource on an Image Builder template. Without `--output-name` it lists every run output; with `--output-name` it returns just that one.
- For Shared Image Gallery (SIG) resources use `az sig` (not implemented yet).
