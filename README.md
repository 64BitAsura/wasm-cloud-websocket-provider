# wasm-cloud-websocket-provider

A wasmCloud capability provider that implements the `wasmcloud:messaging` contract using WebSocket as the transport backend. This provider supports both **WebSocket client** and **WebSocket server** modes with comprehensive session management capabilities.

## Features

### WebSocket Client Mode (Connect to WebSocket servers)
- **WebSocket Communication**: Connect to WebSocket servers (ws:// or wss://)
- **Session Management**: Track and manage individual WebSocket sessions
- **Bidirectional Communication**: Handle both incoming and outgoing messages
- **Authentication Support**: Optional token-based authentication
- **Custom Headers**: Support for custom headers in WebSocket upgrade requests

### WebSocket Server Mode (Accept WebSocket connections) 🆕
- **WebSocket Server**: Listen for and accept incoming WebSocket connections
- **Multi-Client Support**: Handle multiple concurrent client connections
- **Session Tracking**: Track all connected clients with unique session IDs
- **Targeted Messaging**: Send messages to specific clients using session IDs
- **Broadcast Capability**: Send messages to all connected clients
- **Reply-To Support**: Clients receive reply-to field to enable request-response patterns

### Common Features
- **Dual Mode Operation**: Switch between client and server mode via configuration
- **Session-Specific Messaging**: Send messages to specific WebSocket sessions
- **Connection Pooling**: Manage multiple component connections
- **Reply-To Routing**: Automatic routing based on reply-to field

## Configuration

The provider can be configured using the following settings when establishing a link:

| Property | Description | Default | Applies To |
|----------|-------------|---------|------------|
| `MODE` | Operation mode: "client" or "server" | `client` | Both |
| `URI` | WebSocket server URI (client mode) or bind address (server mode)<br/>Examples: "ws://localhost:8080" or "0.0.0.0:8080" | `ws://127.0.0.1:8080` | Both |
| `AUTH_TOKEN` | Optional authentication token | None | Client |
| `CONNECT_TIMEOUT_SEC` | Connection timeout in seconds | `30` | Client |
| `ENABLE_SESSION_TRACKING` | Enable session tracking for targeted messaging | `true` | Both |
| `HEADER_<name>` | Custom headers (e.g., `HEADER_Authorization`) | None | Client |

## Usage

### Client Mode (Default)

Connect to an external WebSocket server:

```rust
let mut config = HashMap::new();
config.insert("MODE".to_string(), "client".to_string());
config.insert("URI".to_string(), "ws://example.com:8080".to_string());

let provider = WebSocketMessagingProvider::from_config(config)?;
```

### Server Mode (New!) 🆕

Start a WebSocket server to accept incoming connections:

```rust
let mut config = HashMap::new();
config.insert("MODE".to_string(), "server".to_string());
config.insert("URI".to_string(), "0.0.0.0:8080".to_string());

let mut provider = WebSocketMessagingProvider::from_config(config)?;
provider.start_server_if_needed().await?;

// Server is now listening for WebSocket connections at ws://0.0.0.0:8080/ws
```

### Building

```bash
cargo build --release
```

### Running

#### Client Mode (Default)
```bash
cargo run --example basic_usage
```

#### Server Mode
```bash
cargo run --example server_mode
```

### Testing

```bash
cargo test
```

## Session Management

### Client Mode Sessions

When session tracking is enabled (default), the provider maintains a mapping of session IDs to component IDs for WebSocket client connections.

### Server Mode Sessions (New!) 🆕

In server mode, the provider tracks all connected WebSocket clients:

```rust
// List all connected WebSocket clients
let clients = provider.list_ws_clients().await?;

// Send message to a specific client
provider.send_to_session("client-session-id", BrokerMessage {
    subject: "notification".to_string(),
    body: Bytes::from("Hello!"),
    reply_to: None,
}).await?;

// Broadcast message to all clients
provider.broadcast_to_clients(BrokerMessage {
    subject: "announcement".to_string(),
    body: Bytes::from("System update"),
    reply_to: None,
}).await?;
```

### Reply-To Field Support

The provider automatically includes reply-to fields in messages to enable request-response patterns:

**Client → Server:**
```json
{
    "subject": "request.data",
    "body": "...",
    "reply_to": "session-abc-123"
}
```

**Component can reply back using the session ID:**
```rust
provider.send_to_session("session-abc-123", BrokerMessage {
    subject: "response.data",
    body: Bytes::from("Response payload"),
    reply_to: None,
}).await?;
```

## Architecture

This provider implements the wasmCloud messaging interface with WebSocket as the transport:

### Client Mode
- **Connection Management**: Each linked component gets its own WebSocket connection to a server
- **Message Routing**: Messages are routed based on component ID and optionally session ID
- **Outbound Focus**: Primarily for components that need to send messages to WebSocket servers

### Server Mode (New!) 🆕
- **Multi-Client Handling**: Accepts multiple incoming WebSocket connections
- **Session Isolation**: Each client gets a unique session ID for targeted messaging
- **Inbound Focus**: Ideal for components that need to receive messages from WebSocket clients
- **Broadcast Support**: Can send messages to all connected clients simultaneously

### Message Flow

#### Client Mode (Component → WebSocket Server)
1. Component publishes message via provider
2. Provider encodes message as JSON/binary
3. Message sent through WebSocket connection
4. Server receives and processes message

#### Server Mode (WebSocket Client → Component)
1. Client connects to provider's WebSocket server
2. Client sends message with optional reply-to
3. Provider parses and routes to handler component
4. Component processes and can reply using reply-to session ID

## Differences from NATS Provider

Unlike the NATS messaging provider:
- Uses WebSocket instead of NATS protocol
- Supports both client and server modes
- Session-based message routing for direct client communication
- Does not support pub/sub topics (can be implemented on top)
- No queue groups (each component/client has dedicated connection)
- Built-in HTTP upgrade handling in server mode

## Future Enhancements

- [x] WebSocket server mode for accepting connections
- [x] Session-based message routing
- [x] Broadcast to all connected clients
- [x] Reply-to field support
- [ ] Automatic reconnection with exponential backoff (client mode)
- [ ] Request-reply pattern implementation with timeout matching
- [ ] Message acknowledgment support
- [ ] Compression support (WebSocket per-message deflate)
- [ ] Health checks and connection monitoring
- [ ] Metrics and observability integration
- [ ] TLS/SSL certificate configuration for server mode
- [ ] Authentication middleware for server mode
- [ ] Rate limiting per client session

## Security

See [SECURITY.md](SECURITY.md) for security audit information and best practices.

### Security Features Implemented

- ✅ Session isolation with unique IDs
- ✅ Safe message parsing with validation
- ✅ Thread-safe concurrent access
- ✅ Automatic resource cleanup
- ✅ Updated dependencies to latest versions

### Known Security Issues

- ⚠️ Transitive dependency vulnerability in tokio-tar (upstream issue)
- ⚠️ Unmaintained paste dependency (upstream issue)

Both issues are in dependencies of wasmcloud-provider-sdk and do not affect this provider's functionality. See SECURITY.md for details.

## License

Apache-2.0
