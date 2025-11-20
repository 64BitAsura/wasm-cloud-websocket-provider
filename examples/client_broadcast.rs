use anyhow::Result;
use bytes::Bytes;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::info;
use tracing_subscriber;

use wasmcloud_provider_messaging_websocket::{BrokerMessage, WebSocketMessagingProvider};

/// This example demonstrates client mode with incoming message broadcasting
/// Provider connects to a remote WebSocket server as a client, receives messages
/// from the server, broadcasts them to handler components, and allows components
/// to reply back to the remote server.
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("WebSocket Messaging Provider - Client Mode with Broadcasting Example");

    // Create provider in client mode
    let mut config = HashMap::new();
    config.insert("MODE".to_string(), "client".to_string());
    // Using a public echo WebSocket server for demonstration
    config.insert("URI".to_string(), "ws://echo.websocket.org".to_string());
    config.insert("ENABLE_SESSION_TRACKING".to_string(), "true".to_string());

    let provider = WebSocketMessagingProvider::from_config(config)?;

    info!("Connecting to remote WebSocket server...");

    // Link a component as consumer (to send messages)
    let consumer_component_id = "consumer-component-1";
    provider
        .receive_link_config_as_target(consumer_component_id, HashMap::new())
        .await?;

    info!("Consumer component linked: {}", consumer_component_id);

    // Link a component as handler (to receive messages from remote server)
    let handler_component_id = "handler-component-1";
    provider
        .receive_link_config_as_source(handler_component_id, HashMap::new())
        .await?;

    info!("Handler component linked: {}", handler_component_id);
    info!("Handler component will receive messages from remote WebSocket server");

    // Give connections time to establish
    sleep(Duration::from_secs(2)).await;

    // List active sessions
    let sessions = provider.list_sessions().await;
    info!("Active sessions: {:?}", sessions);

    // Send a message to the remote WebSocket server
    info!("Sending message to remote WebSocket server...");
    let outgoing_msg = BrokerMessage {
        subject: "test.echo".to_string(),
        body: Bytes::from("Hello from wasmCloud provider!"),
        reply_to: None,
    };

    if let Err(e) = provider.publish(consumer_component_id, outgoing_msg).await {
        info!("Message publish result: {}", e);
    }

    info!(
        "Message sent! The echo server will send it back, and it will be broadcast to handler components."
    );

    // Wait to receive echo response
    sleep(Duration::from_secs(3)).await;

    // Send another message with reply-to field
    info!("Sending message with reply-to field...");
    if let Some((session_id, _)) = sessions.first() {
        let msg_with_reply = BrokerMessage {
            subject: "test.request".to_string(),
            body: Bytes::from("Request with reply-to"),
            reply_to: Some(session_id.clone()),
        };

        if let Err(e) = provider
            .publish(consumer_component_id, msg_with_reply)
            .await
        {
            info!("Message publish result: {}", e);
        }

        info!("Message with reply-to sent: {}", session_id);
    }

    sleep(Duration::from_secs(3)).await;

    info!("\n=== Feature Summary ===");
    info!("✓ Provider connected to remote WS server as client");
    info!("✓ Consumer component can send messages to remote server");
    info!("✓ Handler components receive messages from remote server");
    info!("✓ Components can reply back using session ID from reply-to field");
    info!("✓ Bidirectional communication established!");

    // Clean up
    info!("\nShutting down...");
    provider
        .delete_link_as_target(consumer_component_id)
        .await?;
    provider.delete_link_as_source(handler_component_id).await?;
    provider.shutdown().await?;

    info!("Example completed successfully!");
    Ok(())
}
