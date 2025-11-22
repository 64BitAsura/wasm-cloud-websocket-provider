# wasmCloud WebSocket Provider Example - Client Mode

This example demonstrates how to use the `wasm-cloud-websocket-provider` in **CLIENT mode** with a wasmCloud component. In client mode, the provider connects to an external WebSocket server and enables bidirectional communication.

## Overview

This example includes:
- **Client Component**: A wasmCloud component that can both send and receive WebSocket messages
- **WebSocket Provider**: The wasm-cloud-websocket-provider in client mode
- **WADM Manifest**: Deployment configuration for wasmCloud

## Architecture

```
┌─────────────────────────────────────┐
│  Client Component                   │
│  - Sends messages (consumer)        │
│  - Receives messages (handler)      │
│  - Can reply to messages            │
└────────┬──────────────┬─────────────┘
         │              │
         │ consumer     │ handler
         │              │
┌────────▼──────────────▼─────────────┐
│  WebSocket Provider (Client Mode)   │
│  - Connects to remote WS server     │
│  - Manages sessions                 │
│  - Bidirectional communication      │
└────────┬────────────────────────────┘
         │
         │ ws://remote-server/
         │
┌────────▼────────────────────────────┐
│  Remote WebSocket Server            │
│  (e.g., echo.websocket.org)         │
└─────────────────────────────────────┘
```

## Key Features

### Client Mode Capabilities
- **Outbound Messaging**: Send messages to remote WebSocket server
- **Inbound Broadcasting**: Receive messages from remote server
- **Bidirectional Communication**: Components can both send and receive
- **Session Management**: Track WebSocket connections with session IDs
- **Reply-To Support**: Send targeted responses using session IDs

## Prerequisites

### 1. Install Rust and wasm32-wasip1 target

```bash
rustup target add wasm32-wasip1
```

### 2. Install wasmCloud Shell (wash) - Optional

For full wasmCloud deployment:

```bash
# On Linux
curl -s https://packagecloud.io/install/repositories/wasmcloud/core/script.deb.sh | sudo bash
sudo apt install wash

# On macOS
brew install wasmcloud/wasmcloud/wash

# Or via cargo
cargo install wash-cli
```

## Building the Example

### Step 1: Build the WebSocket Provider

From the repository root:

```bash
cd /path/to/wasm-cloud-websocket-provider
cargo build --release
```

The provider binary will be at `target/release/websocket-provider`.

### Step 2: Build the Client Component

```bash
cd wasmcloud-example-client/client-component
cargo build --target wasm32-wasip1 --release
```

This will create the WebAssembly component at `target/wasm32-wasip1/release/client_component.wasm`.

## Running the Example

### Quick Test (No wasmCloud Required)

For quick testing without full wasmCloud setup, use the standalone example:

```bash
cd /path/to/wasm-cloud-websocket-provider
cargo run --example client_broadcast
```

This will:
1. Connect to a public echo WebSocket server (echo.websocket.org)
2. Send a test message
3. Receive the echo response
4. Demonstrate bidirectional communication

### Full wasmCloud Deployment

**Note**: This requires wash CLI and wasmCloud to be installed.

#### Step 1: Start wasmCloud Host

In one terminal:

```bash
wash up
```

This starts a local wasmCloud host.

#### Step 2: Deploy the Application

In another terminal:

```bash
cd wasmcloud-example-client

# Deploy using WADM (if available)
wash app deploy wadm.yaml

# OR manually deploy component and create links
wash start component file://./client-component/target/wasm32-wasip1/release/client_component.wasm client-component

# Create link for sending messages (consumer)
wash link put client-component <provider-id> wasmcloud:messaging \
  MODE=client \
  URI=ws://echo.websocket.org \
  ENABLE_SESSION_TRACKING=true \
  CONNECT_TIMEOUT_SEC=30

# Create link for receiving messages (handler)
wash link put <provider-id> client-component wasmcloud:messaging \
  MODE=client \
  URI=ws://echo.websocket.org \
  ENABLE_SESSION_TRACKING=true
```

#### Step 3: Verify Deployment

```bash
# Check running components
wash get inventory

# Check links
wash get links

# View logs
wash logs client-component
```

## Testing the Setup

### Using the Example Runner

The easiest way to test is using the built-in example:

```bash
cargo run --example client_broadcast
```

You should see output like:

```
INFO WebSocket Messaging Provider - Client Mode with Broadcasting Example
INFO Connecting to remote WebSocket server...
INFO Consumer component linked: consumer-component-1
INFO Handler component linked: handler-component-1
INFO Sending message to remote WebSocket server...
INFO Message sent! The echo server will send it back...
✓ Provider connected to remote WS server as client
✓ Consumer component can send messages to remote server
✓ Handler components receive messages from remote server
✓ Components can reply back using session ID from reply-to field
✓ Bidirectional communication established!
```

### Manual Testing with a Custom WebSocket Server

If you want to test with your own WebSocket server:

1. **Start your WebSocket server** (or use a public one)

2. **Update the configuration** in `wadm.yaml`:
   ```yaml
   URI: "ws://your-server:port"
   ```

3. **Rebuild and redeploy** the component

## How It Works

### Message Flow: Sending Messages

1. Component wants to send a message
2. Component calls the `consumer` interface
3. Provider sends message through WebSocket connection
4. Remote server receives the message

### Message Flow: Receiving Messages

1. Remote server sends a message
2. Provider receives message through WebSocket
3. Provider broadcasts to all handler components
4. Component processes message via `handler` interface
5. Component can reply using the session ID

### Bidirectional Communication Example

```rust
// Component receives message from remote server
impl Guest for Component {
    fn handle_message(msg: HandlerMessage) -> Result<(), String> {
        // Process the incoming message
        process_message(&msg.body);
        
        // Send a response back if reply_to is present
        if let Some(session_id) = msg.reply_to {
            let response = ConsumerMessage {
                subject: format!("response.{}", msg.subject),
                body: generate_response(&msg.body),
                reply_to: Some(session_id),
            };
            
            // Publish response through consumer interface
            publish(&response)?;
        }
        
        Ok(())
    }
}
```

## Configuration Options

When creating links, you can configure the provider:

| Property | Description | Default |
|----------|-------------|---------|
| `MODE` | Operation mode (must be "client") | `client` |
| `URI` | WebSocket server URI to connect to | `ws://127.0.0.1:8080` |
| `AUTH_TOKEN` | Optional authentication token | None |
| `CONNECT_TIMEOUT_SEC` | Connection timeout in seconds | `30` |
| `ENABLE_SESSION_TRACKING` | Track sessions for targeted messaging | `true` |
| `HEADER_<name>` | Custom headers (e.g., `HEADER_Authorization`) | None |

### Example Configurations

#### Connecting to a Secure WebSocket Server

```yaml
properties:
  MODE: "client"
  URI: "wss://secure-server.example.com/ws"
  CONNECT_TIMEOUT_SEC: "60"
  HEADER_Authorization: "Bearer your-token-here"
```

#### Connecting with Authentication

```yaml
properties:
  MODE: "client"
  URI: "ws://api.example.com/ws"
  AUTH_TOKEN: "your-auth-token"
  ENABLE_SESSION_TRACKING: "true"
```

## Troubleshooting

### Provider Not Connecting

1. Verify the WebSocket server is accessible:
   ```bash
   wscat -c ws://echo.websocket.org
   ```

2. Check provider logs:
   ```bash
   wash logs <provider-id>
   ```

3. Verify the URI format is correct:
   - Should start with `ws://` or `wss://`
   - Should include protocol, host, and port if non-standard
   - Example: `ws://example.com:8080/path`

### Component Not Receiving Messages

1. Verify the handler link is created:
   ```bash
   wash get links
   ```

2. Check that the component exports the handler interface:
   ```bash
   # In wit/world.wit
   export wasmcloud:messaging/handler@0.2.0;
   ```

3. Ensure the remote server is sending messages

### Messages Not Being Sent

1. Verify the consumer link is created:
   ```bash
   wash get links
   ```

2. Check that the component imports the consumer interface:
   ```bash
   # In wit/world.wit
   import wasmcloud:messaging/consumer@0.2.0;
   ```

3. Verify the WebSocket connection is established (check provider logs)

### Connection Timeouts

1. Increase the timeout in configuration:
   ```yaml
   CONNECT_TIMEOUT_SEC: "60"
   ```

2. Check network connectivity and firewall rules

3. Verify the server is responding to connections

## Comparison with Server Mode

| Feature | Client Mode | Server Mode |
|---------|-------------|-------------|
| **Connection** | Connects to remote server | Accepts incoming connections |
| **Use Case** | Integrate with existing WS services | Provide WS API to clients |
| **Message Flow** | Component → Server → Component | Client → Component |
| **Session Management** | Track outbound connections | Track inbound clients |
| **Typical Scenario** | Consume external APIs | Serve multiple clients |

## Advanced Usage

### Multiple Remote Servers

You can connect to multiple WebSocket servers by creating multiple provider instances with different configurations:

```bash
# Connect to server 1
wash link put component provider1 wasmcloud:messaging URI=ws://server1.com

# Connect to server 2
wash link put component provider2 wasmcloud:messaging URI=ws://server2.com
```

### Custom Message Handling

Implement sophisticated message routing in your component:

```rust
impl Guest for Component {
    fn handle_message(msg: HandlerMessage) -> Result<(), String> {
        match msg.subject.as_str() {
            "notification" => handle_notification(&msg)?,
            "request" => handle_request(&msg)?,
            "event" => handle_event(&msg)?,
            _ => return Err("Unknown message type".to_string()),
        }
        Ok(())
    }
}
```

### Session-Based Communication

Use session IDs for targeted communication:

```rust
// Store session ID from incoming message
let session_id = msg.reply_to.clone();

// Later, send a response to that specific session
let response = ConsumerMessage {
    subject: "targeted.response",
    body: data,
    reply_to: session_id,
};
publish(&response)?;
```

## Next Steps

- Explore the component source code in `client-component/src/lib.rs`
- Review configuration options in `../../CONFIG.md`
- Check out the server mode example in `../wasmcloud-example/`
- Read the full provider documentation in `../../README.md`

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## License

Apache-2.0
