# Quick Start Guide

Get started with the WebSocket Messaging Provider in 5 minutes!

## Prerequisites

- Rust 1.70 or later
- A WebSocket server to connect to (we'll use echo.websocket.org for this guide)

## Installation

Clone the repository:

```bash
git clone https://github.com/64BitAsura/wasm-cloud-websocket-provider.git
cd wasm-cloud-websocket-provider
```

## Build

```bash
cargo build --release
```

## Run Tests

```bash
cargo test
```

## Run the Example

```bash
cargo run --example basic_usage
```

You should see output similar to:

```
INFO WebSocket Messaging Provider - Basic Example
INFO Linking component: example-component-1
INFO Component linked successfully
INFO Active sessions: [("550e8400-e29b-41d4-a716-446655440000", "example-component-1")]
INFO Publishing test message...
INFO Deleting link...
INFO Shutting down...
INFO Example completed successfully!
```

## Create Your First Integration

### Step 1: Add to your Cargo.toml

```toml
[dependencies]
wasmcloud-provider-messaging-websocket = { path = "../wasm-cloud-websocket-provider" }
bytes = "1.5"
tokio = { version = "1.35", features = ["full"] }
```

### Step 2: Create a provider instance

```rust
use wasmcloud_provider_messaging_websocket::WebSocketMessagingProvider;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure the provider
    let mut config = HashMap::new();
    config.insert("URI".to_string(), "ws://localhost:8080".to_string());
    
    let provider = WebSocketMessagingProvider::from_config(config)?;
    
    // ... use the provider
    
    Ok(())
}
```

### Step 3: Link a component

```rust
// Link a component to the provider
let component_id = "my-component";
let link_config = HashMap::new();

provider.receive_link_config_as_target(component_id, link_config).await?;
```

### Step 4: Send messages

```rust
use bytes::Bytes;
use wasmcloud_provider_messaging_websocket::BrokerMessage;

let message = BrokerMessage {
    subject: "hello.world".to_string(),
    body: Bytes::from("Hello, WebSocket!"),
    reply_to: None,
};

provider.publish(component_id, message).await?;
```

### Step 5: Clean up

```rust
// Unlink the component
provider.delete_link_as_target(component_id).await?;

// Shutdown the provider
provider.shutdown().await?;
```

## Working with Sessions

### List Active Sessions

```rust
let sessions = provider.list_sessions().await;
for (session_id, component_id) in sessions {
    println!("Session: {} -> Component: {}", session_id, component_id);
}
```

### Send to Specific Session

```rust
let message = BrokerMessage {
    subject: "notification".to_string(),
    body: Bytes::from("Session-specific message"),
    reply_to: None,
};

provider.send_to_session("session-id-here", message).await?;
```

## Common Configurations

### Connect to Echo Server

```rust
let mut config = HashMap::new();
config.insert("URI".to_string(), "ws://echo.websocket.org".to_string());
let provider = WebSocketMessagingProvider::from_config(config)?;
```

### Secure Connection (WSS)

```rust
let mut config = HashMap::new();
config.insert("URI".to_string(), "wss://secure.example.com/ws".to_string());
config.insert("AUTH_TOKEN".to_string(), "your-token".to_string());
let provider = WebSocketMessagingProvider::from_config(config)?;
```

### With Custom Headers

```rust
let mut config = HashMap::new();
config.insert("URI".to_string(), "wss://api.example.com/ws".to_string());
config.insert("HEADER_Authorization".to_string(), "Bearer token".to_string());
config.insert("HEADER_X-API-Key".to_string(), "api-key-123".to_string());
let provider = WebSocketMessagingProvider::from_config(config)?;
```

## Troubleshooting

### Connection Timeout

If you're seeing connection timeouts:

```rust
// Increase the timeout
config.insert("CONNECT_TIMEOUT_SEC".to_string(), "60".to_string());
```

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run --example basic_usage
```

### Check WebSocket Server

Make sure your WebSocket server is running and accessible:

```bash
# Test with wscat (install with: npm install -g wscat)
wscat -c ws://localhost:8080
```

## Next Steps

- Read [README.md](README.md) for more features
- Check [CONFIG.md](CONFIG.md) for configuration options
- Review [TECHNICAL.md](TECHNICAL.md) for architecture details
- See [examples/](examples/) for more code examples
- Contribute! See [CONTRIBUTING.md](CONTRIBUTING.md)

## Getting Help

- Open an issue on GitHub
- Check existing issues for solutions
- Review the documentation

## License

Apache-2.0 - See [LICENSE](LICENSE) for details
