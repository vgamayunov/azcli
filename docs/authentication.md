# Authentication

Native OAuth2 implementation — no dependency on `az` CLI for auth.

## Login Methods

```bash
# Interactive browser login (default)
azcli login

# Device code flow (for headless/SSH sessions)
azcli login --use-device-code

# Service principal
azcli login --service-principal --tenant TENANT --client-id ID --client-secret SECRET

# Managed identity (from Azure VM/Container)
azcli login --identity

# Check current account
azcli account show

# Logout
azcli logout
```

## Named Profiles (Multi-Account)

Tag a login with `--name` so it can be addressed by a friendly name later:

```bash
azcli login --name work
azcli login --name personal --tenant <other-tenant-id>

azcli --profile work vm list
azcli --profile personal network bastion ssh ...
azcli account list -o table             # shows the Profile column
azcli account set work                  # set 'work' as default
```

`--profile <name>` and `--subscription <id|name|profile>` are mutually exclusive global flags. `--subscription` is polymorphic and also accepts profile names; `--profile` exists as an unambiguous alternative.

## Token Cache

Tokens are cached at `~/.azure/azcli_tokens.json` with automatic refresh.

## Default Tenant & Subscription on Re-login

When `azcli login` (or `azcli login --use-device-code`) is invoked without `--tenant`, azcli reuses the tenant from the previous cached login. After login completes, if the previously active subscription is still accessible in that tenant, it is restored as the default. Pass `--tenant` explicitly to switch tenants.

## Commands

| Command | Description |
|---------|-------------|
| `login` | Authenticate (browser, device code, service principal, managed identity) |
| `logout` | Clear cached tokens |
| `account show` | Show current subscription and account info |
