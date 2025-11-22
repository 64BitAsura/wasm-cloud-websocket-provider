# wasm-cloud-websocket-provider

A wasmCloud capability provider that implements the `wasmcloud:messaging` contract using WebSocket as the transport backend. This provider supports both **WebSocket client** and **WebSocket server** modes with comprehensive session management capabilities.

## Features

### WebSocket Client Mode (Connect to WebSocket servers)
- **WebSocket Communication**: Connect to WebSocket servers (ws:// or wss://)
- **Session Management**: Track and manage individual WebSocket sessions
- **Bidirectional Communication**: Handle both incoming and outgoing messages
- **Message Broadcasting**: Receive messages from remote server and broadcast to handler components 🆕
- **Component Reply-Back**: Components can reply to remote server using session IDs 🆕
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

## Quick Start

### Try the wasmCloud Examples 🆕

See complete working examples with wasmCloud components:

#### Server Mode Example
Provider accepts incoming WebSocket connections:

```bash
cd wasmcloud-example
./test-local.sh
```

For detailed instructions, see:
- **[wasmcloud-example/QUICKSTART.md](wasmcloud-example/QUICKSTART.md)** - Quick local testing guide
- **[wasmcloud-example/README.md](wasmcloud-example/README.md)** - Full wasmCloud deployment guide

#### Client Mode Example 🆕
Provider connects to external WebSocket servers:

```bash
cd wasmcloud-example-client
./test-local.sh
```

For detailed instructions, see:
- **[wasmcloud-example-client/QUICKSTART.md](wasmcloud-example-client/QUICKSTART.md)** - Quick local testing guide
- **[wasmcloud-example-client/README.md](wasmcloud-example-client/README.md)** - Full wasmCloud deployment guide

## Usage

### Client Mode (Default)

Connect to an external WebSocket server and enable bidirectional communication:

```rust
let mut config = HashMap::new();
config.insert("MODE".to_string(), "client".to_string());
config.insert("URI".to_string(), "ws://example.com:8080".to_string());

let provider = WebSocketMessagingProvider::from_config(config)?;

// Link consumer component (to send messages)
provider.receive_link_config_as_target("consumer-id", HashMap::new()).await?;

// Link handler component (to receive messages from remote server) 🆕
provider.receive_link_config_as_source("handler-id", HashMap::new()).await?;

// Consumer can send messages to remote server
provider.publish("consumer-id", message).await?;

// Handler components automatically receive messages from remote server
// Components can reply back using session ID from reply-to field
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

### Installation

Pre-built packages are available from [GitHub Releases](https://github.com/64BitAsura/wasm-cloud-websocket-provider/releases). Each release includes the packaged `.crate` file that can be used for distribution.

To use this provider as a dependency in your project, add it to your `Cargo.toml`:

```toml
[dependencies]
wasmcloud-provider-messaging-websocket = { git = "https://github.com/64BitAsura/wasm-cloud-websocket-provider", tag = "v0.1.0" }
```

### Running

#### wasmCloud Examples (Recommended) 🆕

To see the provider in action with real wasmCloud components:

**Server Mode** - Provider accepts WebSocket connections:
```bash
cd wasmcloud-example
./test-local.sh
```

**Client Mode** - Provider connects to WebSocket servers:
```bash
cd wasmcloud-example-client
./test-local.sh
```

See respective README.md files for complete wasmCloud integration guides:
- [wasmcloud-example/README.md](wasmcloud-example/README.md) - Server mode guide
- [wasmcloud-example-client/README.md](wasmcloud-example-client/README.md) - Client mode guide

#### Standalone Examples

##### Client Mode (Default)
```bash
cargo run --example basic_usage
```

##### Client Mode with Broadcasting
```bash
cargo run --example client_broadcast
```

##### Server Mode
```bash
cargo run --example server_mode
```

### Testing

Run the test suite:
```bash
cargo test
```

To run tests that require network access (ignored by default in CI):
```bash
cargo test -- --ignored
```

**Note:** Some integration tests connect to external WebSocket servers and are marked as `#[ignore]` to prevent CI failures. These tests validate real-world network scenarios but are not required for standard development.

## Session Management

### Client Mode Sessions

When session tracking is enabled (default), the provider maintains a mapping of session IDs to component IDs for WebSocket client connections.

#### Client Mode Message Broadcasting 🆕

In client mode, when the provider connects to a remote WebSocket server, it can broadcast incoming messages from that server to all registered handler components:

```rust
// Provider connects to remote WebSocket server
let provider = WebSocketMessagingProvider::from_config(config)?;

// Link consumer (sends messages to remote server)
provider.receive_link_config_as_target("consumer-id", HashMap::new()).await?;

// Link handler(s) (receive messages from remote server)
provider.receive_link_config_as_source("handler-1", HashMap::new()).await?;
provider.receive_link_config_as_source("handler-2", HashMap::new()).await?;

// Consumer sends message to remote server
provider.publish("consumer-id", message).await?;

// Remote server responds → message is broadcast to ALL handler components
// handler-1 and handler-2 both receive the message

// Handlers can reply back to remote server using session ID
let sessions = provider.list_sessions().await;
if let Some((session_id, _)) = sessions.first() {
    provider.send_to_session(session_id, reply_message).await?;
}
```

**Key Features:**
- Messages from remote WebSocket server are automatically broadcast to all handler components
- Components can reply to remote server using session ID from reply-to field
- Supports both JSON and binary message formats
- Multiple handlers can process the same message independently

### Server Mode Sessions

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
