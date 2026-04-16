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

## Commands

| Command | Description |
|---------|-------------|
| `login` | Authenticate (browser, device code, service principal, managed identity) |
| `logout` | Clear cached tokens |
| `account show` | Show current subscription and account info |
