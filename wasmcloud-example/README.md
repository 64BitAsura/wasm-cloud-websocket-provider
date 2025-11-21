# wasmCloud WebSocket Provider Example

This example demonstrates how to use the `wasm-cloud-websocket-provider` with a wasmCloud component locally.

## Overview

This example includes:
- **Echo Component**: A wasmCloud component that receives WebSocket messages and handles them
- **WebSocket Provider**: The wasm-cloud-websocket-provider in server mode
- **WADM Manifest**: Deployment configuration for wasmCloud

## Architecture

```
┌─────────────────┐
│  WebSocket      │
│  Client         │
│  (wscat/browser)│
└────────┬────────┘
         │
         │ ws://localhost:8080/ws
         │
┌────────▼────────────────────────────────┐
│  WebSocket Provider (Server Mode)       │
│  - Accepts WS connections                │
│  - Manages sessions                      │
│  - Forwards messages to component        │
└────────┬────────────────────────────────┘
         │
         │ wasmcloud:messaging
         │
┌────────▼────────────────────────────────┐
│  Echo Component                          │
│  - Implements messaging handler          │
│  - Processes incoming messages           │
│  - Can reply via session ID              │
└──────────────────────────────────────────┘
```

## Prerequisites

### 1. Install wasmCloud Shell (wash)

```bash
curl -s https://packagecloud.io/install/repositories/wasmcloud/core/script.deb.sh | sudo bash
sudo apt install wash
```

Or on macOS:
```bash
brew install wasmcloud/wasmcloud/wash
```

Or install via cargo:
```bash
cargo install wash-cli
```

### 2. Install WebSocket Client (for testing)

```bash
npm install -g wscat
```

## Building the Example

### Step 1: Build the WebSocket Provider

From the repository root:

```bash
cd /path/to/wasm-cloud-websocket-provider
cargo build --release
```

The provider binary will be at `target/release/websocket-provider`.

### Step 2: Build the Echo Component

```bash
cd wasmcloud-example/echo-component
wash build
```

This will create the WebAssembly component at `build/echo_component_s.wasm`.

## Running Locally

### Option 1: Using Local Development Setup (Recommended)

This approach runs the provider standalone for easier testing and debugging.

#### Step 1: Start wasmCloud Host

In one terminal:

```bash
wash up
```

This starts a local wasmCloud host.

#### Step 2: Build and Start the Provider Locally

In another terminal from the repository root:

```bash
cargo run --release
```

The provider will start in standalone mode. You'll need to configure it via environment or link definitions.

#### Step 3: Deploy Component and Create Links

In another terminal:

```bash
cd wasmcloud-example

# Start the echo component
wash start component file://./echo-component/build/echo_component_s.wasm echo-component

# Create a link between component and provider
# Note: This assumes you've registered the provider with wasmCloud
wash link put echo-component <provider-id> wasmcloud:messaging \
  MODE=server \
  URI=0.0.0.0:8080 \
  ENABLE_SESSION_TRACKING=true
```

### Option 2: Manual Testing (Simpler)

For quick testing without full wasmCloud setup:

#### Step 1: Run the Provider in Server Mode

```bash
cd /path/to/wasm-cloud-websocket-provider
cargo run --example server_mode
```

This starts a WebSocket server on `ws://localhost:8080/ws`.

#### Step 2: Connect with WebSocket Client

In another terminal:

```bash
wscat -c ws://localhost:8080/ws
```

#### Step 3: Send Test Messages

Once connected, send JSON messages:

```json
{"subject": "test.echo", "body": "SGVsbG8gV29ybGQh", "reply_to": null}
```

The `body` field is base64-encoded. "SGVsbG8gV29ybGQh" = "Hello World!"

You can encode messages in bash:
```bash
echo -n "Hello World!" | base64
```

## Testing the Setup

### Using wscat

1. Connect to the WebSocket server:
```bash
wscat -c ws://localhost:8080/ws
```

2. Send a message:
```
{"subject":"test.message","body":"dGVzdCBtZXNzYWdl","reply_to":"session-123"}
```

3. You should see the message being received and processed by the echo component (check logs).

### Using curl (for HTTP upgrade)

```bash
curl -i -N \
  -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Version: 13" \
  -H "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
  http://localhost:8080/ws
```

### Using a Browser Client

Create an HTML file:

```html
<!DOCTYPE html>
<html>
<head>
    <title>WebSocket Echo Test</title>
</head>
<body>
    <h1>WebSocket Echo Test</h1>
    <button onclick="connect()">Connect</button>
    <button onclick="send()">Send Message</button>
    <button onclick="disconnect()">Disconnect</button>
    <pre id="output"></pre>
    
    <script>
        let ws;
        const output = document.getElementById('output');
        
        function log(msg) {
            output.textContent += msg + '\n';
        }
        
        function connect() {
            ws = new WebSocket('ws://localhost:8080/ws');
            
            ws.onopen = () => log('Connected');
            ws.onmessage = (e) => log('Received: ' + e.data);
            ws.onclose = () => log('Disconnected');
            ws.onerror = (e) => log('Error: ' + e);
        }
        
        function send() {
            if (ws && ws.readyState === WebSocket.OPEN) {
                const msg = {
                    subject: "test.echo",
                    body: btoa("Hello from browser!"),
                    reply_to: null
                };
                ws.send(JSON.stringify(msg));
                log('Sent: ' + JSON.stringify(msg));
            }
        }
        
        function disconnect() {
            if (ws) {
                ws.close();
            }
        }
    </script>
</body>
</html>
```

Save as `websocket-test.html` and open in a browser.

## Verifying the Provider is Working

### Check Provider Logs

When running the provider, you should see:

```
INFO Starting WebSocket Messaging Provider
INFO WebSocket server listening on 0.0.0.0:8080
INFO WebSocket server started at ws://0.0.0.0:8080/ws
```

### Check Component Logs

When the component receives messages:

```
INFO Received message on subject: test.echo
INFO Processing message with reply_to: session-xyz
```

### Monitor Active Sessions

You can check active sessions by examining the provider logs:

```
INFO New WebSocket client connected: session-abc-123
INFO Active sessions: 1
```

## Provider Configuration Options

When creating links, you can configure the provider:

| Property | Description | Default |
|----------|-------------|---------|
| `MODE` | Operation mode: "client" or "server" | `client` |
| `URI` | WebSocket URI (client) or bind address (server) | `ws://127.0.0.1:8080` |
| `ENABLE_SESSION_TRACKING` | Track sessions for targeted messaging | `true` |
| `CONNECT_TIMEOUT_SEC` | Connection timeout (client mode) | `30` |

Example link configuration:
```bash
wash link put echo-component <provider-id> wasmcloud:messaging \
  MODE=server \
  URI=0.0.0.0:8080 \
  ENABLE_SESSION_TRACKING=true
```

## Troubleshooting

### Provider Not Starting

1. Check if port 8080 is already in use:
```bash
lsof -i :8080
# or
netstat -an | grep 8080
```

2. Try a different port:
```bash
# Modify the URI configuration
URI=0.0.0.0:9090
```

### Component Not Receiving Messages

1. Verify the link is created:
```bash
wash get links
```

2. Check provider logs for connection messages

3. Verify the message format is correct (valid JSON with subject, body, reply_to fields)

### WebSocket Connection Fails

1. Ensure the provider is running in server mode
2. Check firewall settings
3. Verify the correct URL format: `ws://host:port/ws` (note the `/ws` path)

### Messages Not Being Processed

1. Verify component is deployed:
```bash
wash get inventory
```

2. Check component logs:
```bash
wash logs echo-component
```

## Advanced Usage

### Multiple Components

You can link multiple components to the same provider:

```bash
wash start component file://./component1.wasm comp1
wash start component file://./component2.wasm comp2

wash link put comp1 <provider-id> wasmcloud:messaging MODE=server URI=0.0.0.0:8080
wash link put comp2 <provider-id> wasmcloud:messaging MODE=server URI=0.0.0.0:8081
```

### Client Mode

To connect to an external WebSocket server:

```bash
wash link put echo-component <provider-id> wasmcloud:messaging \
  MODE=client \
  URI=wss://echo.websocket.org \
  CONNECT_TIMEOUT_SEC=30
```

### Session-Based Messaging

When the provider receives a message with a `reply_to` field, components can use that session ID to send responses back to specific clients.

## Next Steps

- Explore the provider source code in `src/`
- Review configuration options in `CONFIG.md`
- Check out other examples in `examples/`
- Read the full documentation in `README.md`

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

Apache-2.0
