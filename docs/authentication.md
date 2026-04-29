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
