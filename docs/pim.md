# Privileged Identity Management (PIM)

Activate, deactivate, and list Azure AD Privileged Identity Management role assignments for the signed-in user.

## Commands

| Command | Description |
|---------|-------------|
| `role pim list` | List eligible and active role assignments at a scope |
| `role pim status` | Show only currently active role assignments |
| `role pim activate -r ROLE [-j JUSTIFICATION] [-d DURATION]` | Self-activate an eligible role |
| `role pim deactivate -r ROLE` | Self-deactivate an active role |

## Flags

| Flag | Description | Default |
|------|-------------|---------|
| `-r, --role` | Role display name (e.g. `Contributor`); case-insensitive | required for activate/deactivate |
| `-j, --justification` | Activation justification | `Activated via azcli` |
| `-d, --duration` | ISO 8601 duration (e.g. `PT1H`, `PT8H`) | `PT8H` |
| `--scope` | Scope to filter by (defaults to current subscription) | `/subscriptions/{id}` |
| `--subscription` | Override subscription ID | from current account |

## Examples

```bash
# Show eligible + active assignments at the subscription scope
azcli role pim list -o table

# Show only currently active assignments
azcli role pim status

# Activate "Contributor" for the default 8 hours
azcli role pim activate -r Contributor -j "Deploying release X"

# Activate for 1 hour at a specific scope
azcli role pim activate -r Reader -d PT1H \
  --scope /subscriptions/00000000-0000-0000-0000-000000000000/resourceGroups/my-rg

# Deactivate
azcli role pim deactivate -r Contributor
```

## Notes

- Role lookup is by display name. If multiple eligibilities/assignments match the same name across scopes, pass `--scope` to disambiguate.
- The principal ID is extracted from the access token's `oid` claim — the same identity used for `az` commands.
- Activation requests return immediately with the `roleAssignmentScheduleRequest` resource. Approval-required roles will surface their status in that response.
- Uses ARM PIM API version `2020-10-01`.
