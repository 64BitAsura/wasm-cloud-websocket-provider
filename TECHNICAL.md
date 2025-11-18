# Technical Documentation

## Architecture Overview

The WebSocket Messaging Provider is designed to implement the `wasmcloud:messaging` contract using WebSocket as the transport layer. It follows the architectural patterns established by the wasmCloud NATS messaging provider but adapts them for WebSocket connections.

## Core Components

### 1. Connection Management (`connection.rs`)

The `ConnectionConfig` struct manages WebSocket connection parameters:

```rust
pub struct ConnectionConfig {
    pub uri: String,                              // WebSocket server URI
    pub auth_token: Option<String>,               // Authentication token
    pub connect_timeout_sec: u64,                 // Connection timeout
    pub enable_session_tracking: bool,            // Enable session management
    pub custom_headers: HashMap<String, String>,  // Custom HTTP headers
}
```

**Key Features:**
- Configuration from HashMap for wasmCloud integration
- Config merging for combining default and link-specific settings
- Support for ws:// and wss:// protocols

### 2. Provider Implementation (`lib.rs`)

#### WebSocketMessagingProvider

The main provider struct maintains:

```rust
pub struct WebSocketMessagingProvider {
    consumer_components: Arc<RwLock<HashMap<String, WebSocketClientBundle>>>,
    handler_components: Arc<RwLock<HashMap<String, WebSocketClientBundle>>>,
    default_config: ConnectionConfig,
    session_storage: Arc<RwLock<HashMap<String, String>>>,
}
```

- **consumer_components**: Components that publish/request messages
- **handler_components**: Components that receive messages
- **session_storage**: Maps session IDs to component IDs

#### WebSocketClientBundle

Each WebSocket connection is wrapped in a bundle:

```rust
pub struct WebSocketClientBundle {
    pub tx: mpsc::UnboundedSender<Message>,  // Send channel
    pub session_info: SessionInfo,            // Session metadata
    pub handle: JoinHandle<()>,               // Background task handle
}
```

The bundle includes:
- A channel for sending messages to the WebSocket
- Session information (ID, connection time, metadata)
- A task handle for cleanup on drop

### 3. Message Flow

#### Outgoing Messages (Component → WebSocket)

1. Component calls `publish()` or `request()`
2. Provider looks up the component's WebSocket bundle
3. Message is encoded (JSON format with base64 body)
4. Message is sent through the unbounded channel
5. Background task sends to WebSocket

#### Incoming Messages (WebSocket → Component)

1. Background task receives WebSocket message
2. Message is parsed and decoded
3. Handler component is invoked via wRPC (in full integration)
4. Response is sent back through WebSocket if needed

### 4. Session Management

Sessions enable targeted message delivery:

```rust
// Store session mapping
session_storage.insert(session_id, component_id);

// Send to specific session
provider.send_to_session(session_id, message).await?;

// List all sessions
let sessions = provider.list_sessions().await;
```

## Thread Safety

All shared state uses `Arc<RwLock<T>>` for thread-safe access:
- Multiple readers can access simultaneously
- Writers get exclusive access
- No data races or deadlocks

## Error Handling

The provider uses `anyhow::Result` for error propagation:
- Connection errors are reported during linking
- Message send errors are logged but don't crash the provider
- Cleanup errors are logged during shutdown

## Integration with wasmCloud

### Link Lifecycle

1. **receive_link_config_as_target**: Component wants to publish messages
   - Provider creates WebSocket connection
   - Stores in consumer_components map
   - Returns success/failure

2. **receive_link_config_as_source**: Component wants to receive messages
   - Provider creates WebSocket connection
   - Stores in handler_components map
   - Sets up message forwarding

3. **delete_link**: Component unlinks
   - Provider removes from appropriate map
   - WebSocketClientBundle dropped
   - Background task aborted
   - WebSocket connection closed

### Message Format

Messages are currently encoded as JSON:

```json
{
    "subject": "topic.name",
    "body": "base64-encoded-payload",
    "reply_to": "optional-reply-subject"
}
```

This format is flexible and can be changed to use more efficient encodings like MessagePack or Protocol Buffers.

## Performance Considerations

### Concurrency

- Each component gets its own WebSocket connection
- Background tasks run independently
- Lock contention is minimized by using read locks when possible

### Memory Usage

- Unbounded channels could grow under heavy load
- Consider implementing backpressure in production
- Session storage grows with connected clients

### Network

- WebSocket uses TCP with lower overhead than HTTP per message
- TLS (wss://) adds encryption overhead
- Consider compression for large messages

## Future Enhancements

### Planned Features

1. **Request-Reply Pattern**: Full implementation with timeout and response matching
2. **Reconnection Logic**: Automatic reconnection with exponential backoff
3. **Message Acknowledgment**: Ensure reliable delivery
4. **Compression**: WebSocket per-message deflate
5. **Metrics**: Connection count, message throughput, error rates
6. **Health Checks**: Periodic ping/pong to detect dead connections

### Possible Optimizations

- Connection pooling for multiple components to same server
- Binary message format (MessagePack/ProtoBuf)
- Zero-copy message passing where possible
- Batch message sending

## Testing Strategy

### Unit Tests

- Configuration parsing and merging
- Provider state management
- Message encoding/decoding

### Integration Tests

- Session lifecycle
- Concurrent access patterns
- Shutdown behavior
- Error conditions

### Manual Testing

- Connect to echo.websocket.org
- Test with real wasmCloud components
- Load testing with many concurrent connections
- Network failure scenarios

## Security Considerations

### Transport Security

- Use wss:// for encrypted connections
- Validate server certificates
- Support custom CA certificates

### Authentication

- Token-based authentication in headers
- JWT support (future)
- Client certificates (future)

### Input Validation

- Validate message size limits
- Sanitize subject names
- Rate limiting (future)

## Debugging

### Logging

Enable detailed logging:
```bash
RUST_LOG=debug cargo run
```

Log levels:
- `error`: Connection failures, message send errors
- `warn`: Unexpected conditions
- `info`: Connection lifecycle, link events
- `debug`: Message flow, session management
- `trace`: Detailed internal state (if needed)

### Common Issues

**Connection Timeout:**
- Check network connectivity
- Verify WebSocket server is running
- Increase `CONNECT_TIMEOUT_SEC`

**Message Not Sent:**
- Check component is linked
- Verify session exists
- Look for channel errors in logs

**Session Not Found:**
- Session may have disconnected
- Check session tracking is enabled
- Verify component ID mapping

## References

- [wasmCloud Documentation](https://wasmcloud.com)
- [WebSocket Protocol (RFC 6455)](https://tools.ietf.org/html/rfc6455)
- [tokio-tungstenite Documentation](https://docs.rs/tokio-tungstenite)
- [wasmCloud Messaging Interface](https://github.com/wasmCloud/interfaces)
