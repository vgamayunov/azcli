# role assignment / role definition

Read-only access to Azure RBAC role assignments and definitions. For PIM (eligible/active assignments), see [pim.md](pim.md).

## role assignment

### `azcli role assignment list`

Lists role assignments at a scope.

```bash
azcli role assignment list -o table
azcli role assignment list --assignee <object-id> --all
azcli role assignment list -g <resource-group> --role Reader
azcli role assignment list --scope /subscriptions/<sub-id>/resourceGroups/<rg>
```

| Flag | Description |
|------|-------------|
| `--assignee <oid>` | Filter by principal object ID (user, group, SP, MI). UPN/display-name not supported — use object ID. |
| `--role <name-or-guid>` | Filter by role display name or role definition GUID. Display-name match is client-side. |
| `--scope <arm-id>` | Full ARM scope to query. Defaults to subscription. |
| `-g, --resource-group <name>` | Shorthand for resource-group scope. |
| `--include-groups` | Include assignments inherited via group membership (uses `assignedTo()` filter). |
| `--all` | List assignments at every scope reachable from the query scope (omits `atScope()`). |

Defaults match `az role assignment list`: when neither `--all` nor `--scope` is given, results are scoped to the current subscription using `atScope()`.

### `azcli role assignment show`

```bash
azcli role assignment show --ids <full-arm-id>
azcli role assignment show -n <assignment-guid> --scope <arm-scope>
```

`--ids` is the full assignment ARM ID. Otherwise pass `--name` (the assignment GUID) plus `--scope`.

## role definition

### `azcli role definition list`

```bash
azcli role definition list -o table
azcli role definition list --custom-role-only
azcli role definition list -n Contributor
```

| Flag | Description |
|------|-------------|
| `-n, --name <name>` | Filter by role display name (server-side `roleName eq` filter). |
| `--scope <arm-id>` | Scope to query for assignable definitions. Defaults to subscription. |
| `--custom-role-only` | Show only `CustomRole` definitions (filters built-ins). |

### `azcli role definition show`

```bash
azcli role definition show -n Reader
azcli role definition show -n acdd72a7-3385-48ef-bd42-f606fba81ae7
```

Accepts either the role display name or the role-definition GUID. Display-name lookups list and filter on `roleName eq '<name>'`; an exact GUID match short-circuits the list.

## Notes

- All operations are read-only.
- `--assignee` accepts object IDs only. UPN, display name, SP appId, and group name resolution would require Microsoft Graph and is not implemented.
- `--role` display-name filtering happens client-side after the ARM call returns. Pass a GUID to skip the post-filter.
- Role names in output are resolved from `roleDefinitionId` via a per-invocation cache.
