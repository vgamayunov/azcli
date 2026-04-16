# REST Command

The `rest` command allows arbitrary ARM API requests, similar to `az rest`.

| Command | Description |
|---------|-------------|
| `rest --method METHOD --url URL [--body BODY]` | Execute an arbitrary ARM REST API call |

## Examples

```bash
# List all resources in a resource group
azcli rest --method get \
  --url "/subscriptions/{sub}/resourceGroups/my-rg/resources?api-version=2021-04-01"

# Get a specific resource by ID
azcli rest --method get \
  --url "/subscriptions/{sub}/resourceGroups/my-rg/providers/Microsoft.Compute/virtualMachines/my-vm?api-version=2024-07-01"
```
