# Azure Bastion Intermittent Tunnel Failures — Support Report

## Observed Behavior

1. **Symptom**: Intermittent SSH tunnel failures through Azure Bastion native client (Standard SKU). The same bastion host + target VM works correctly on retry — failures are non-deterministic.

2. **Failure pattern — "Proxy 101 then Backend 400"**:
   - Client sends a valid WebSocket upgrade to `wss://{bastion_endpoint}/webtunnelv2/{token}?X-Node-Id={nodeId}`
   - The **front-end proxy** (likely Azure Front Door or an L7 load balancer) returns `HTTP/1.1 101 Switching Protocols` — the upgrade appears to succeed
   - **Immediately after**, instead of WebSocket frames, the client receives a raw `HTTP/1.1 400 Bad Request` with a **Tomcat error page** body — this comes from the **backend node**, not the proxy
   - This means the proxy layer upgraded the connection, but the backend Tomcat instance rejected the forwarded request

3. **Second failure mode**: Sometimes the 101 response arrives but the connection **hangs** — no WebSocket frames, no error, no data at all. Eventually times out.

## Environment Details

| Field | Value |
|---|---|
| Bastion name | `azurebastion` |
| Resource group | `access-hub-rg` |
| Subscription | `62118f5c-be37-400f-9f20-a8b77a2a7877` |
| Bastion endpoint | `bst-456e5bde-f503-4d2f-8f22-c5846b1a8319.bastion.azure.com` |
| SKU | Standard |
| Target VM IP | `10.1.20.110` |
| Target port | 22 (SSH) |
| Protocol | Native client tunneling (`/webtunnelv2/`) |
| WebSocket URL pattern | `wss://{endpoint}/webtunnelv2/{token}?X-Node-Id={nodeId}` |

## Technical Details for Investigation

- The 101 response from the proxy often **omits** the standard `Upgrade: websocket` and `Connection: Upgrade` headers — this is consistent behavior even on successful connections, suggesting the proxy strips them
- The backend 400 response contains a Tomcat error page, indicating the backend is Java-based (Apache Tomcat) and the request routing from proxy → backend is failing intermittently
- The `X-Node-Id` header directs the connection to a specific backend node — the failures may correlate with unhealthy nodes in the backend pool
- Token acquisition (`POST /api/tokens`) always succeeds — the failure is only on the subsequent WebSocket upgrade

## Suggested Server-Side Investigation

1. Check backend node health for `bst-456e5bde-f503-4d2f-8f22-c5846b1a8319` — are some nodes returning 400 to the proxy?
2. Check if the L7 proxy is routing WebSocket upgrades to nodes that haven't completed initialization (Tomcat not ready)
3. The `X-Node-Id` from the token response may be pointing to an unhealthy node — does the token service validate node health before assigning?

## Reproduction

```bash
az network bastion ssh \
  --name azurebastion \
  --resource-group access-hub-rg \
  --target-ip-address 10.1.20.110 \
  --auth-type ssh-key \
  --ssh-key ~/.ssh/id_rsa \
  --username victor.gamayunov
```

Retry few times during failure windows — the issue is intermittent and appears to depend on which backend node handles the connection.
