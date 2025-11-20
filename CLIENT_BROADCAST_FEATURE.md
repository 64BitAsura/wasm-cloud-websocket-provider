# Client Mode Broadcasting Feature

## Overview

Enhanced the WebSocket provider's client mode to support bidirectional communication with remote WebSocket servers, including automatic message broadcasting to handler components.

## Feature Description

### What Was Added

**Client Mode Message Broadcasting:**
- Provider connects to remote WebSocket server as a client
- Automatically broadcasts incoming messages from remote server to ALL registered handler components
- Components can reply back to the remote server using session IDs from reply-to field
- Supports both JSON and binary message formats

### Use Cases

1. **Real-Time Data Feeds**
   - Subscribe to external data streams (stock prices, weather, IoT sensors)
   - Distribute updates to multiple components for processing

2. **Webhook Receivers**
   - Connect to webhook providers
   - Broadcast webhook events to multiple handlers

3. **IoT Communication**
   - Connect to IoT platforms
   - Distribute sensor data to multiple processing components

4. **Chat Applications**
   - Connect to chat servers
   - Broadcast messages to multiple UI or logging components

## Architecture

### Message Flow

```
Remote WS Server → Provider (Client Mode) → Handler Components
                                          ↓
                                    Reply Messages
                                          ↓
Remote WS Server ← Provider ← Components (using session ID)
```

### Component Roles

1. **Consumer Components**
   - Linked as "target" (`receive_link_config_as_target`)
   - Send messages TO remote WebSocket server
   - Can initiate requests

2. **Handler Components**
   - Linked as "source" (`receive_link_config_as_source`)
   - Receive messages FROM remote WebSocket server
   - Can reply using session IDs

## Implementation Details

### Key Methods

**`parse_message_static(text: &str, session_id: &str) -> Result<BrokerMessage>`**
- Parses incoming messages from remote server
- Supports JSON format with subject, body, and reply-to fields
- Falls back to plain text with automatic reply-to population
- Public API for testing and extension

**`encode_message_static(msg: &BrokerMessage) -> Result<Message>`**
- Encodes BrokerMessage to WebSocket Message
- Uses JSON format with base64-encoded body
- Public API for testing and extension

**Enhanced `connect()` Method**
- Creates bidirectional WebSocket connection
- Spawns async task for message handling
- Forwards incoming messages to all handler components
- Maintains session tracking for replies

### Message Format

**Incoming from Remote Server:**
```json
{
    "subject": "event.type",
    "body": "base64-encoded-data",
    "reply_to": "optional-session-id"
}
```

**Broadcast to Handlers:**
- Same JSON format maintained
- If no reply-to present, session ID is auto-populated
- Multiple handlers receive identical message

**Reply from Components:**
```rust
// Extract session ID from message
let session_id = message.reply_to.unwrap();

// Reply back to remote server
provider.send_to_session(&session_id, reply_message).await?;
```

## Configuration

No new configuration required. Uses existing client mode setup:

```rust
let mut config = HashMap::new();
config.insert("MODE".to_string(), "client".to_string());
config.insert("URI".to_string(), "ws://remote-server:8080".to_string());
config.insert("ENABLE_SESSION_TRACKING".to_string(), "true".to_string());

let provider = WebSocketMessagingProvider::from_config(config)?;
```

## Code Examples

### Basic Setup

```rust
use wasmcloud_provider_messaging_websocket::{
    BrokerMessage, WebSocketMessagingProvider
};
use bytes::Bytes;
use std::collections::HashMap;

// Create provider in client mode
let mut config = HashMap::new();
config.insert("MODE".to_string(), "client".to_string());
config.insert("URI".to_string(), "ws://api.example.com/ws".to_string());

let provider = WebSocketMessagingProvider::from_config(config)?;

// Link consumer (sends messages)
provider.receive_link_config_as_target("my-consumer", HashMap::new()).await?;

// Link handlers (receive messages)
provider.receive_link_config_as_source("handler-1", HashMap::new()).await?;
provider.receive_link_config_as_source("handler-2", HashMap::new()).await?;
```

### Send and Receive

```rust
// Consumer sends message to remote server
let outgoing = BrokerMessage {
    subject: "query.data".to_string(),
    body: Bytes::from("request payload"),
    reply_to: None,
};

provider.publish("my-consumer", outgoing).await?;

// Remote server responds → handler-1 and handler-2 both receive
// (Automatic in background)
```

### Reply Back to Remote Server

```rust
// Get active session
let sessions = provider.list_sessions().await;
if let Some((session_id, _)) = sessions.first() {
    // Compose reply
    let reply = BrokerMessage {
        subject: "response.data".to_string(),
        body: Bytes::from("processed result"),
        reply_to: None,
    };
    
    // Send reply back to remote server
    provider.send_to_session(session_id, reply).await?;
}
```

### Multiple Handlers

```rust
// All handlers receive the same message independently
provider.receive_link_config_as_source("logger", HashMap::new()).await?;
provider.receive_link_config_as_source("processor", HashMap::new()).await?;
provider.receive_link_config_as_source("analyzer", HashMap::new()).await?;

// When remote server sends message:
// → logger logs it
// → processor processes it
// → analyzer analyzes it
// All happen independently and concurrently
```

## Testing

### Unit Tests

**Message Parsing:**
```rust
let parsed = WebSocketMessagingProvider::parse_message_static(
    r#"{"subject": "test", "body": "data"}"#,
    "session-123"
)?;
assert_eq!(parsed.subject, "test");
```

**Message Encoding:**
```rust
let msg = BrokerMessage { /* ... */ };
let encoded = WebSocketMessagingProvider::encode_message_static(&msg)?;
// Returns WebSocket Message ready to send
```

### Integration Tests

See `tests/client_broadcast_test.rs` for:
- Client mode broadcast scenarios
- Multiple handler coordination
- Reply-back mechanisms
- Session tracking validation

### Example Application

Run the complete example:
```bash
cargo run --example client_broadcast
```

## Performance Considerations

### Scalability
- Each handler component receives messages independently
- No blocking between handlers
- Concurrent message delivery via tokio tasks

### Memory
- Messages are cloned for each handler (small overhead)
- Session tracking uses HashMap (O(1) lookup)
- Unbounded channels for message passing (consider backpressure in production)

### Network
- Single WebSocket connection to remote server
- Efficient message parsing with serde_json
- Binary format support for large payloads

## Security

### Input Validation
- All incoming messages validated and parsed safely
- JSON parsing errors handled gracefully
- Malformed messages logged but don't crash provider

### Session Management
- Unique session IDs generated with UUID v4
- Session tracking optional (can be disabled)
- Automatic cleanup on disconnect

## Limitations

### Current
- No message acknowledgment from handlers to provider
- No guaranteed delivery semantics
- Unbounded channel may grow under extreme load

### Future Enhancements
- Handler-level message acknowledgment
- Configurable message buffering strategies
- Selective handler filtering based on message attributes
- Message routing rules (not all handlers get all messages)

## Migration Guide

### From Previous Version

**No breaking changes!** Existing code continues to work:

```rust
// Old code - still works
provider.receive_link_config_as_target("consumer", config).await?;
provider.publish("consumer", msg).await?;
```

**New capability - opt-in:**

```rust
// New feature - link handlers to receive messages from remote server
provider.receive_link_config_as_source("handler", config).await?;
// Handler now receives messages from remote server automatically
```

## Documentation

- **README.md** - User guide with examples
- **examples/client_broadcast.rs** - Complete working example
- **tests/client_broadcast_test.rs** - Test suite
- **This document** - Detailed feature description

## Comparison: Server Mode vs Client Mode Broadcasting

### Server Mode (Provider as Server)
- Provider listens for incoming WebSocket connections
- External clients connect TO provider
- Broadcast messages TO external clients

### Client Mode Broadcasting (NEW)
- Provider connects TO remote WebSocket server
- Remote server is external
- Broadcast messages FROM remote server to handler components

Both modes can be used simultaneously in different provider instances.

## Summary

This feature completes the bidirectional communication story for the WebSocket provider:
- ✅ Send messages to remote server (existing)
- ✅ Receive messages from remote server (NEW)
- ✅ Broadcast to multiple handlers (NEW)
- ✅ Reply back to remote server (NEW)

The provider now supports all common WebSocket communication patterns in both client and server modes.

---
**Implemented:** 2025-11-20
**Commit:** 5f5eaf6
**Status:** Production Ready
