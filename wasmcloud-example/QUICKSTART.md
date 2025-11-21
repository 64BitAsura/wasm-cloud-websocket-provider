# Quick Start: Testing the WebSocket Provider Locally

This quick start guide shows you how to test the WebSocket provider without needing a full wasmCloud setup.

## Prerequisites

- Rust installed
- A WebSocket client tool (choose one):
  - `wscat` (recommended): `npm install -g wscat`
  - `websocat`: Download from https://github.com/vi/websocat/releases
  - Browser console or any WebSocket client

## Quick Test (No wasmCloud Required)

### Step 1: Start the Provider in Server Mode

From the repository root:

```bash
cd wasmcloud-example
./test-local.sh
```

This will build and start the WebSocket provider in server mode, listening on `ws://localhost:8080/ws`.

### Step 2: Connect with a WebSocket Client

In another terminal, connect using wscat:

```bash
wscat -c ws://localhost:8080/ws
```

Or using websocat:

```bash
websocat ws://localhost:8080/ws
```

### Step 3: Send Test Messages

Once connected, send a JSON message:

```json
{"subject":"test.echo","body":"SGVsbG8gV29ybGQh","reply_to":null}
```

The `body` field should be base64-encoded. Here's how to encode text:

```bash
echo -n "Hello World!" | base64
# Output: SGVsbG8gV29ybGQh
```

### Step 4: Observe the Logs

You should see in the provider logs:
- Client connection established
- Session ID created
- Message received
- Active session count

## Test with Browser

Create a file `test.html` with this content:

```html
<!DOCTYPE html>
<html>
<head><title>WebSocket Test</title></head>
<body>
    <h1>WebSocket Provider Test</h1>
    <button onclick="connect()">Connect</button>
    <button onclick="send()">Send Message</button>
    <button onclick="disconnect()">Disconnect</button>
    <div><strong>Status:</strong> <span id="status">Not connected</span></div>
    <div><strong>Messages:</strong></div>
    <pre id="output"></pre>
    
    <script>
        let ws;
        const output = document.getElementById('output');
        const status = document.getElementById('status');
        
        function log(msg) {
            output.textContent += new Date().toLocaleTimeString() + ': ' + msg + '\n';
        }
        
        function connect() {
            ws = new WebSocket('ws://localhost:8080/ws');
            
            ws.onopen = () => {
                status.textContent = 'Connected';
                log('✓ Connected to WebSocket server');
            };
            
            ws.onmessage = (e) => {
                log('← Received: ' + e.data);
            };
            
            ws.onclose = () => {
                status.textContent = 'Disconnected';
                log('✗ Disconnected');
            };
            
            ws.onerror = (e) => {
                log('✗ Error: ' + e);
            };
        }
        
        function send() {
            if (ws && ws.readyState === WebSocket.OPEN) {
                const msg = {
                    subject: "test.echo",
                    body: btoa("Hello from browser!"),
                    reply_to: null
                };
                ws.send(JSON.stringify(msg));
                log('→ Sent: ' + JSON.stringify(msg, null, 2));
            } else {
                log('✗ Not connected. Click Connect first.');
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

Open `test.html` in your browser and use the buttons to test the connection.

## Verify Provider Functionality

### Check Active Sessions

The provider tracks all connected WebSocket clients. Watch the logs to see:

```
INFO New WebSocket client connected: session-abc-123
INFO Active WebSocket clients: 1
```

### Multiple Connections

Open multiple WebSocket connections to see session management:

```bash
# Terminal 1
wscat -c ws://localhost:8080/ws

# Terminal 2
wscat -c ws://localhost:8080/ws

# Terminal 3
wscat -c ws://localhost:8080/ws
```

The provider will assign a unique session ID to each connection.

## Next Steps

Once you've verified the provider works:

1. **Build the Component**: See [README.md](README.md#building-the-example) for building the echo component
2. **Deploy with wasmCloud**: Follow the full deployment guide in [README.md](README.md#running-locally)
3. **Integrate with Your App**: Use the provider in your own wasmCloud components

## Troubleshooting

### Port Already in Use

```bash
lsof -i :8080
# Kill the process using the port, or change the port in server_mode.rs
```

### Connection Refused

Make sure the provider is running and listening on the correct address:
```bash
netstat -an | grep 8080
```

### Invalid Message Format

Messages must be valid JSON with these fields:
```json
{
    "subject": "string (required)",
    "body": "base64-encoded string (required)",
    "reply_to": "string or null (optional)"
}
```

## Provider Logs

When running, you should see logs like:

```
INFO WebSocket Messaging Provider - Server Mode Example
INFO Starting WebSocket server...
INFO WebSocket server listening on 127.0.0.1:8080
INFO WebSocket server started on 127.0.0.1:8080
INFO WebSocket server listening on ws://127.0.0.1:8080/ws
```

When a client connects:

```
INFO New WebSocket client connected: session-xxxxx
INFO Active WebSocket clients: 1
```

When receiving messages:

```
DEBUG Received WebSocket message from session-xxxxx
DEBUG Broadcasting message to handler components
```

## Summary

This quick test demonstrates:
- ✅ WebSocket provider starts and listens on a port
- ✅ Clients can connect via WebSocket
- ✅ Provider manages sessions with unique IDs
- ✅ Messages are received and can be routed

The provider is working correctly and ready to integrate with wasmCloud components!
