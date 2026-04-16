# Deployments

Full coverage of all four ARM deployment scopes, plus deployment operations for each scope.

## Resource Group Scope (`deployment group`)

| Command | Description |
|---------|-------------|
| `deployment group list --resource-group RG` | List deployments in a resource group |
| `deployment group show --name NAME --resource-group RG` | Show deployment details |
| `deployment group export --name NAME --resource-group RG` | Export the template used for a deployment |
| `deployment group create --name NAME --resource-group RG -f FILE` | Create or update a deployment |
| `deployment group delete --name NAME --resource-group RG` | Delete a deployment |
| `deployment group validate --resource-group RG -f FILE` | Validate a template without deploying |
| `deployment group what-if --resource-group RG -f FILE` | Preview changes a deployment would make |
| `deployment group cancel --name NAME --resource-group RG` | Cancel a running deployment |
| `deployment group wait --name NAME --resource-group RG --created/--updated/--deleted/--exists` | Poll deployment state |

## Subscription Scope (`deployment sub`)

| Command | Description |
|---------|-------------|
| `deployment sub list` | List deployments at subscription scope |
| `deployment sub show --name NAME` | Show deployment details |
| `deployment sub export --name NAME` | Export the template used for a deployment |
| `deployment sub create --name NAME --location LOC -f FILE` | Create or update a deployment |
| `deployment sub delete --name NAME` | Delete a deployment |
| `deployment sub validate --name NAME --location LOC -f FILE` | Validate a template without deploying |
| `deployment sub what-if --name NAME --location LOC -f FILE` | Preview changes a deployment would make |
| `deployment sub cancel --name NAME` | Cancel a running deployment |
| `deployment sub wait --name NAME --created/--updated/--deleted/--exists` | Poll deployment state |

## Management Group Scope (`deployment mg`)

| Command | Description |
|---------|-------------|
| `deployment mg list --management-group-id MG` | List deployments at management group scope |
| `deployment mg show --name NAME --management-group-id MG` | Show deployment details |
| `deployment mg export --name NAME --management-group-id MG` | Export the template used for a deployment |
| `deployment mg create --name NAME --management-group-id MG --location LOC -f FILE` | Create or update a deployment |
| `deployment mg delete --name NAME --management-group-id MG` | Delete a deployment |
| `deployment mg validate --name NAME --management-group-id MG --location LOC -f FILE` | Validate a template without deploying |
| `deployment mg what-if --name NAME --management-group-id MG --location LOC -f FILE` | Preview changes a deployment would make |
| `deployment mg cancel --name NAME --management-group-id MG` | Cancel a running deployment |
| `deployment mg wait --name NAME --management-group-id MG --created/--updated/--deleted/--exists` | Poll deployment state |

## Tenant Scope (`deployment tenant`)

| Command | Description |
|---------|-------------|
| `deployment tenant list` | List deployments at tenant scope |
| `deployment tenant show --name NAME` | Show deployment details |
| `deployment tenant export --name NAME` | Export the template used for a deployment |
| `deployment tenant create --name NAME --location LOC -f FILE` | Create or update a deployment |
| `deployment tenant delete --name NAME` | Delete a deployment |
| `deployment tenant validate --name NAME --location LOC -f FILE` | Validate a template without deploying |
| `deployment tenant what-if --name NAME --location LOC -f FILE` | Preview changes a deployment would make |
| `deployment tenant cancel --name NAME` | Cancel a running deployment |
| `deployment tenant wait --name NAME --created/--updated/--deleted/--exists` | Poll deployment state |

## Deployment Operations

| Command | Description |
|---------|-------------|
| `deployment operation group list --name NAME --resource-group RG` | List operations for a group deployment |
| `deployment operation group show --name NAME --resource-group RG --operation-id ID` | Show a specific operation |
| `deployment operation sub list --name NAME` | List operations for a subscription deployment |
| `deployment operation sub show --name NAME --operation-id ID` | Show a specific operation |
| `deployment operation mg list --name NAME --management-group-id MG` | List operations for a management group deployment |
| `deployment operation mg show --name NAME --management-group-id MG --operation-id ID` | Show a specific operation |
| `deployment operation tenant list --name NAME` | List operations for a tenant deployment |
| `deployment operation tenant show --name NAME --operation-id ID` | Show a specific operation |
