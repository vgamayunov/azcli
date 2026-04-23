# account

Subscription, token, and local cache management. All operations are read-only against Azure ARM; `set` and `clear` mutate only the local token cache at `~/.azure/azcli_tokens.json`.

## Commands

### `azcli account show [-n <name-or-id>]`

Print the active subscription, or one matching `--name`. Output includes `id`, `name`, `tenantId`, `authMethod`, `isDefault`, and `tokenExpiresAt`.

```bash
azcli account show
azcli account show -n MySubscriptionName
azcli account show -n 62118f5c-be37-400f-9f20-a8b77a2a7877
```

### `azcli account list`

Enumerate every subscription accessible to the signed-in identity across **all tenants** (parity with `az account list`). For each tenant returned by `https://management.azure.com/tenants`, azcli exchanges the cached refresh token for a per-tenant access token, lists subscriptions, and merges results. Tenants that fail token exchange (e.g. require fresh MFA) are skipped with a warning.

Output schema matches `az account list` exactly: `cloudName`, `homeTenantId`, `id`, `isDefault`, `managedByTenants`, `name`, `state`, `tenantDefaultDomain`, `tenantDisplayName`, `tenantId`, `user.{name,type}`. The `user.name` field is decoded from the access-token JWT (`preferred_username`/`upn` claim); `user.type` is `"user"` for interactive flows and `"servicePrincipal"` for SP auth.

```bash
azcli account list -o table
azcli account list -o json | jq '.[] | select(.tenantDisplayName == "Microsoft")'
```

### `azcli account set -n <name-or-id>`

Change the default subscription. Accepts the subscription GUID, full ARM id (`/subscriptions/<guid>`), or display name. Updates the local cache; the next ARM call uses the new default.

```bash
azcli account set -n 62118f5c-be37-400f-9f20-a8b77a2a7877
azcli account set -n MySubscriptionName
```

### `azcli account list-locations [--subscription-id <id>]`

List Azure ARM regions available to a subscription. Defaults to the active subscription.

```bash
azcli account list-locations -o table
```

### `azcli account get-access-token`

Print a bearer token for `https://management.azure.com`. Output schema matches `az account get-access-token` (`accessToken`, `expiresOn`, `subscription`, `tenant`, `tokenType`).

```bash
TOKEN=$(azcli account get-access-token --query accessToken -o tsv)
curl -H "Authorization: Bearer $TOKEN" https://management.azure.com/subscriptions?api-version=2022-12-01
```

### `azcli account clear`

Remove all cached accounts and tokens. Equivalent to `azcli logout`. Does not affect Azure-side sessions.

```bash
azcli account clear
```

## Notes

- The local cache stores subscription metadata, refresh tokens, and access tokens. `set` rewrites the cache atomically.
- `account list` always queries ARM live across all tenants, so it reflects the current set of accessible subscriptions even if the cache is stale. First call may take several seconds for accounts with many tenants; tenant queries run concurrently.
- Token refresh is automatic on expiry for service principal and managed identity flows; interactive/device-code flows fall back to `az` CLI when refresh is unavailable.
