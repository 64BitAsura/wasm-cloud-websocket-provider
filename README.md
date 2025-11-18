# wasm-cloud-websocket-provider

A wasmCloud capability provider that implements the `wasmcloud:messaging` contract using WebSocket as the transport backend. This provider enables WebSocket-based messaging with session management capabilities.

## Features

- **WebSocket Communication**: Connect to WebSocket servers (ws:// or wss://)
- **Session Management**: Track and manage individual WebSocket sessions
- **Session-Specific Messaging**: Send messages to specific WebSocket sessions
- **Bidirectional Communication**: Handle both incoming and outgoing messages
- **Authentication Support**: Optional token-based authentication
- **Custom Headers**: Support for custom headers in WebSocket upgrade requests
- **Connection Pooling**: Manage multiple component connections

## Configuration

The provider can be configured using the following settings when establishing a link:

| Property | Description | Default |
|----------|-------------|---------|
| `URI` | WebSocket server URI (e.g., "ws://localhost:8080" or "wss://example.com/ws") | `ws://127.0.0.1:8080` |
| `AUTH_TOKEN` | Optional authentication token | None |
| `CONNECT_TIMEOUT_SEC` | Connection timeout in seconds | `30` |
| `ENABLE_SESSION_TRACKING` | Enable session tracking for targeted messaging | `true` |
| `HEADER_<name>` | Custom headers (e.g., `HEADER_Authorization`) | None |

## Usage

### Building

```bash
cargo build --release
```

### Running

```bash
cargo run
```

### Testing

```bash
cargo test
```

## Session Management

When session tracking is enabled (default), the provider maintains a mapping of session IDs to component IDs. This allows you to:

1. **Track Active Sessions**: Query all active WebSocket connections
2. **Send to Specific Sessions**: Route messages to a particular WebSocket session
3. **Session Metadata**: Store and retrieve session-specific metadata

### Example: Sending to a Specific Session

```rust
// Get the session ID when a client connects
let sessions = provider.list_sessions().await;

// Send a message to a specific session
provider.send_to_session("session-id-123", BrokerMessage {
    subject: "notification".to_string(),
    body: Bytes::from("Hello!"),
    reply_to: None,
}).await?;
```

## Architecture

This provider is based on the wasmCloud NATS messaging provider but adapted for WebSocket connections:

- **Connection Management**: Each linked component gets its own WebSocket connection
- **Message Routing**: Messages are routed based on component ID and optionally session ID
- **Automatic Reconnection**: (Future enhancement) Handle connection drops gracefully
- **Protocol Flexibility**: Messages can be encoded as JSON or binary

## Differences from NATS Provider

Unlike the NATS messaging provider:
- Uses WebSocket instead of NATS protocol
- Supports session-based message routing
- Does not support pub/sub topics (can be implemented on top)
- No queue groups (single connection per component)

## Future Enhancements

- [ ] Automatic reconnection with exponential backoff
- [ ] Request-reply pattern implementation
- [ ] Message acknowledgment support
- [ ] Compression support
- [ ] Health checks and connection monitoring
- [ ] Metrics and observability integration

## License

Apache-2.0
