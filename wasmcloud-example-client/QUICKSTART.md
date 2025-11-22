# Quick Start: Testing the WebSocket Provider in Client Mode

This quick start guide shows you how to test the WebSocket provider in client mode without needing a full wasmCloud setup.

## Prerequisites

- Rust installed
- Internet connection (to reach echo.websocket.org)

## Quick Test (No wasmCloud Required)

### Step 1: Run the Client Mode Example

From the repository root:

```bash
cd /path/to/wasm-cloud-websocket-provider
cargo run --example client_broadcast
```

This will:
1. Connect to a public echo WebSocket server (ws://echo.websocket.org)
2. Create two virtual components (consumer and handler)
3. Send test messages to the server
4. Receive echo responses back
5. Demonstrate bidirectional communication

### Step 2: Observe the Output

You should see output like:

```
INFO WebSocket Messaging Provider - Client Mode with Broadcasting Example
INFO Connecting to remote WebSocket server...
INFO Consumer component linked: consumer-component-1
INFO Handler component linked: handler-component-1
INFO Handler component will receive messages from remote WebSocket server
INFO Active sessions: [("session-xxxxx", "consumer-component-1")]
INFO Sending message to remote WebSocket server...
INFO Message sent! The echo server will send it back, and it will be broadcast to handler components.
INFO Sending message with reply-to field...
INFO Message with reply-to sent: session-xxxxx

=== Feature Summary ===
✓ Provider connected to remote WS server as client
✓ Consumer component can send messages to remote server
✓ Handler components receive messages from remote server
✓ Components can reply back using session ID from reply-to field
✓ Bidirectional communication established!

INFO Shutting down...
INFO Example completed successfully!
```

## What's Happening?

### Provider Setup
- Provider is configured in **CLIENT mode**
- Connects to `ws://echo.websocket.org`
- Session tracking is enabled

### Component Links
- **Consumer component**: Can send messages to the remote server
- **Handler component**: Receives messages from the remote server

### Message Flow
1. Consumer sends a message through the provider
2. Provider forwards it to the echo server
3. Echo server sends it back
4. Provider broadcasts it to handler components
5. Handler receives and processes the message

## Understanding the Code

### Provider Configuration

```rust
let mut config = HashMap::new();
config.insert("MODE".to_string(), "client".to_string());
config.insert("URI".to_string(), "ws://echo.websocket.org".to_string());
config.insert("ENABLE_SESSION_TRACKING".to_string(), "true".to_string());

let provider = WebSocketMessagingProvider::from_config(config)?;
```

### Linking Components

```rust
// Link consumer (for sending messages)
provider.receive_link_config_as_target("consumer-component-1", HashMap::new()).await?;

// Link handler (for receiving messages)
provider.receive_link_config_as_source("handler-component-1", HashMap::new()).await?;
```

### Sending Messages

```rust
let msg = BrokerMessage {
    subject: "test.echo".to_string(),
    body: Bytes::from("Hello from wasmCloud provider!"),
    reply_to: None,
};

provider.publish("consumer-component-1", msg).await?;
```

## Testing with Different Servers

### Using a Local Echo Server

If you want to test with a local server, you can use websocat:

```bash
# Terminal 1: Start echo server
websocat -s 8080
```

Then modify the example to connect to `ws://localhost:8080`:

```rust
config.insert("URI".to_string(), "ws://localhost:8080".to_string());
```

### Using a Custom Server

You can connect to any WebSocket server that accepts JSON messages:

```rust
config.insert("URI".to_string(), "ws://your-server:port/path".to_string());
```

## Building the Component

If you want to build the actual wasmCloud component:

```bash
cd wasmcloud-example-client/client-component
cargo build --target wasm32-wasip1 --release
```

The component will be at:
```
target/wasm32-wasip1/release/client_component.wasm
```

## Component Structure

The client component implements both interfaces:

### Handler Interface (Receives messages)

```rust
impl Guest for Component {
    fn handle_message(msg: HandlerMessage) -> Result<(), String> {
        // Process incoming message from remote server
        // Can reply back using msg.reply_to
        Ok(())
    }
}
```

### Consumer Interface (Sends messages)

```rust
use wasmcloud::messaging::consumer::publish;

// Send message to remote server
let msg = ConsumerMessage {
    subject: "my.subject",
    body: vec![1, 2, 3],
    reply_to: None,
};
publish(&msg)?;
```

## Message Format

Messages are JSON-encoded with the following structure:

```json
{
    "subject": "message.topic",
    "body": "base64-encoded-data",
    "reply_to": "session-id-or-null"
}
```

Example:

```json
{
    "subject": "test.echo",
    "body": "SGVsbG8gV29ybGQh",
    "reply_to": null
}
```

Where `"SGVsbG8gV29ybGQh"` is base64 for "Hello World!".

### Encoding/Decoding

To encode a message in bash:
```bash
echo -n "Hello World!" | base64
# Output: SGVsbG8gV29ybGQh
```

To decode:
```bash
echo "SGVsbG8gV29ybGQh" | base64 -d
# Output: Hello World!
```

## Verify Provider Functionality

### Check Connection

The provider logs should show:

```
INFO Connecting to remote WebSocket server...
INFO Consumer component linked: consumer-component-1
INFO Handler component linked: handler-component-1
```

### Check Sessions

```
INFO Active sessions: [("session-abc-123", "consumer-component-1")]
```

Each WebSocket connection gets a unique session ID.

### Check Message Flow

When messages are sent:
```
INFO Sending message to remote WebSocket server...
INFO Message sent!
```

When messages are received:
```
DEBUG Received WebSocket message
DEBUG Broadcasting message to handler components
```

## Troubleshooting

### Connection Refused

**Problem**: Cannot connect to remote server

```
Error: Connection refused
```

**Solutions**:
1. Check your internet connection
2. Verify the server URL is correct
3. Check if the server is accessible:
   ```bash
   wscat -c ws://echo.websocket.org
   ```

### Timeout

**Problem**: Connection times out

```
Error: Connection timeout
```

**Solutions**:
1. Increase timeout in config:
   ```rust
   config.insert("CONNECT_TIMEOUT_SEC".to_string(), "60".to_string());
   ```
2. Check firewall settings
3. Try a different server

### No Messages Received

**Problem**: Messages are sent but not received back

**Solutions**:
1. Verify the echo server is working:
   ```bash
   wscat -c ws://echo.websocket.org
   # Type a message and press Enter
   ```
2. Check handler component is linked
3. Increase wait time in the example to give server more time to respond

## Next Steps

Once you've verified the provider works in client mode:

1. **Build the Component**: Build the actual wasmCloud component
2. **Deploy with wasmCloud**: Follow the full deployment guide in [README.md](README.md)
3. **Integrate with Your App**: Use the provider with your own components
4. **Connect to Your Server**: Configure the provider to connect to your WebSocket API

## Summary

This quick test demonstrates:
- ✅ Provider connects to remote WebSocket server in client mode
- ✅ Components can send messages through the connection
- ✅ Components can receive messages from the remote server
- ✅ Session tracking works for bidirectional communication
- ✅ Reply-to field enables targeted responses

The provider is working correctly and ready to integrate with wasmCloud components!

## Additional Examples

### Basic Client Usage

For a simpler example without broadcasting:

```bash
cargo run --example basic_usage
```

### Server Mode

To see the provider in server mode (opposite direction):

```bash
cargo run --example server_mode
```

Then connect with:
```bash
wscat -c ws://localhost:8080/ws
```

For more information, see:
- **Full README**: [README.md](README.md)
- **Provider Documentation**: [../../README.md](../../README.md)
- **Configuration Guide**: [../../CONFIG.md](../../CONFIG.md)
