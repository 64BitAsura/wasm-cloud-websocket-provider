use anyhow::Result;
use bytes::Bytes;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::info;
use tracing_subscriber;

use wasmcloud_provider_messaging_websocket::{BrokerMessage, WebSocketMessagingProvider};

/// This example demonstrates basic usage of the WebSocket messaging provider
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("WebSocket Messaging Provider - Basic Example");

    // Create provider with custom configuration
    let mut config = HashMap::new();
    config.insert("URI".to_string(), "ws://echo.websocket.org".to_string());
    config.insert("CONNECT_TIMEOUT_SEC".to_string(), "30".to_string());
    config.insert("ENABLE_SESSION_TRACKING".to_string(), "true".to_string());

    let provider = WebSocketMessagingProvider::from_config(config)?;

    // Simulate linking a component
    let component_id = "example-component-1";
    let link_config = HashMap::new();

    info!("Linking component: {}", component_id);
    provider
        .receive_link_config_as_target(component_id, link_config)
        .await?;

    info!("Component linked successfully");

    // List active sessions
    let sessions = provider.list_sessions().await;
    info!("Active sessions: {:?}", sessions);

    // Create a test message
    let message = BrokerMessage {
        subject: "test.echo".to_string(),
        body: Bytes::from("Hello, WebSocket!"),
        reply_to: None,
    };

    // Publish message
    info!("Publishing test message...");
    if let Err(e) = provider.publish(component_id, message).await {
        info!(
            "Note: Message publish result: {} (expected for echo server demo)",
            e
        );
    }

    // Wait a bit to see any responses
    sleep(Duration::from_secs(2)).await;

    // Send a message to a specific session if available
    if let Some((session_id, _)) = sessions.first() {
        info!("Sending message to specific session: {}", session_id);

        let session_msg = BrokerMessage {
            subject: "test.session".to_string(),
            body: Bytes::from("Session-specific message"),
            reply_to: None,
        };

        if let Err(e) = provider.send_to_session(session_id, session_msg).await {
            info!("Session send result: {}", e);
        }
    }

    // Clean up
    info!("Deleting link...");
    provider.delete_link_as_target(component_id).await?;

    info!("Shutting down...");
    provider.shutdown().await?;

    info!("Example completed successfully!");
    Ok(())
}
