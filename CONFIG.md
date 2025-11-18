# WebSocket Provider Configuration Examples

## Basic Configuration

```json
{
  "URI": "ws://localhost:8080"
}
```

## Secure WebSocket (WSS)

```json
{
  "URI": "wss://secure.example.com/ws",
  "AUTH_TOKEN": "your-auth-token-here",
  "CONNECT_TIMEOUT_SEC": "60"
}
```

## With Custom Headers

```json
{
  "URI": "wss://api.example.com/websocket",
  "HEADER_Authorization": "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "HEADER_X-API-Key": "your-api-key",
  "HEADER_X-Client-ID": "my-client-123"
}
```

## With Session Tracking Disabled

```json
{
  "URI": "ws://stream.example.com",
  "ENABLE_SESSION_TRACKING": "false"
}
```

## Complete Configuration

```json
{
  "URI": "wss://websocket.example.com:8443/ws",
  "AUTH_TOKEN": "secret-token-123",
  "CONNECT_TIMEOUT_SEC": "45",
  "ENABLE_SESSION_TRACKING": "true",
  "HEADER_Authorization": "Bearer token",
  "HEADER_X-Custom-Header": "custom-value"
}
```

## Environment Variables

You can also configure via environment variables:

```bash
export WS_URI="ws://localhost:8080"
export WS_AUTH_TOKEN="my-token"
export WS_CONNECT_TIMEOUT_SEC="30"
```

## wasmCloud Link Configuration

When using with wasmCloud, configuration is provided through link definitions:

```yaml
# wadm.yaml example
links:
  - name: websocket-link
    from: my-component
    to: websocket-provider
    config:
      - name: URI
        value: "wss://api.example.com/ws"
      - name: AUTH_TOKEN
        value: "${WEBSOCKET_TOKEN}"  # From secrets
      - name: CONNECT_TIMEOUT_SEC
        value: "60"
```

## Common Configurations by Use Case

### Echo Server Testing

```json
{
  "URI": "ws://echo.websocket.org",
  "CONNECT_TIMEOUT_SEC": "10"
}
```

### Production API

```json
{
  "URI": "wss://production-api.example.com/v1/websocket",
  "AUTH_TOKEN": "${SECRET_TOKEN}",
  "CONNECT_TIMEOUT_SEC": "60",
  "ENABLE_SESSION_TRACKING": "true",
  "HEADER_X-Service-Name": "wasmcloud-messaging"
}
```

### Local Development

```json
{
  "URI": "ws://localhost:3000/ws",
  "CONNECT_TIMEOUT_SEC": "5",
  "ENABLE_SESSION_TRACKING": "true"
}
```

### High-Latency Connection

```json
{
  "URI": "wss://remote-server.example.com/ws",
  "CONNECT_TIMEOUT_SEC": "120",
  "ENABLE_SESSION_TRACKING": "true"
}
```

## Configuration Tips

1. **Always use WSS in production** for encrypted connections
2. **Set appropriate timeouts** based on your network conditions
3. **Use secrets management** for sensitive tokens (never commit tokens)
4. **Enable session tracking** if you need to send messages to specific clients
5. **Add custom headers** for API keys, authentication, or tracking
6. **Test with echo servers** before integrating with real APIs

## Troubleshooting

### Connection Issues

If you're having trouble connecting:

1. Verify the URI is correct (ws:// or wss://)
2. Check if the server is running and accessible
3. Increase the connection timeout
4. Check firewall/proxy settings
5. Verify authentication credentials

### Authentication Failures

If authentication is failing:

1. Verify the token is correct and not expired
2. Check the header format (Bearer, API Key, etc.)
3. Confirm the server expects the authentication method you're using
4. Review server logs for authentication errors

## Next Steps

- See [README.md](README.md) for usage examples
- See [TECHNICAL.md](TECHNICAL.md) for architecture details
- See [examples/](examples/) for code examples
