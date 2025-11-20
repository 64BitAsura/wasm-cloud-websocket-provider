use anyhow::Result;
use bytes::Bytes;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::info;
use tracing_subscriber;

use wasmcloud_provider_messaging_websocket::{BrokerMessage, WebSocketMessagingProvider};

/// This example demonstrates the WebSocket server mode
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("WebSocket Messaging Provider - Server Mode Example");

    // Create provider in server mode
    let mut config = HashMap::new();
    config.insert("MODE".to_string(), "server".to_string());
    config.insert("URI".to_string(), "127.0.0.1:8080".to_string());
    config.insert("ENABLE_SESSION_TRACKING".to_string(), "true".to_string());

    let mut provider = WebSocketMessagingProvider::from_config(config)?;

    // Start WebSocket server
    info!("Starting WebSocket server...");
    provider.start_server_if_needed().await?;

    if let Some(addr) = provider.get_server_addr().await {
        info!("WebSocket server listening on ws://{}/ws", addr);
        info!("You can connect using a WebSocket client to test the server");
        info!("Example: websocat ws://{}/ws", addr);
    }

    // Simulate running for a while
    info!("Server is running. Press Ctrl+C to stop.");

    // In a loop, demonstrate server capabilities
    for i in 0..30 {
        sleep(Duration::from_secs(2)).await;

        // List connected clients
        let clients = provider.list_ws_clients().await?;
        if !clients.is_empty() {
            info!("Connected clients ({} total): {:?}", clients.len(), clients);

            // Broadcast a message to all clients
            let broadcast_msg = BrokerMessage {
                subject: "server.broadcast".to_string(),
                body: Bytes::from(format!("Broadcast message #{}", i)),
                reply_to: None,
            };

            if let Err(e) = provider.broadcast_to_clients(broadcast_msg).await {
                info!("Failed to broadcast: {}", e);
            } else {
                info!("Broadcasted message to all clients");
            }

            // Send to specific session if available
            if let Some(session_id) = clients.first() {
                let specific_msg = BrokerMessage {
                    subject: "server.direct".to_string(),
                    body: Bytes::from(format!("Direct message to session: {}", session_id)),
                    reply_to: Some(session_id.clone()),
                };

                if let Err(e) = provider.send_to_session(session_id, specific_msg).await {
                    info!("Failed to send to session: {}", e);
                } else {
                    info!("Sent message to session: {}", session_id);
                }
            }
        } else {
            info!("No clients connected yet. Waiting for connections...");
        }

        // List all sessions (includes both WS clients and component sessions)
        let all_sessions = provider.list_sessions().await;
        if !all_sessions.is_empty() {
            info!("All sessions: {:?}", all_sessions);
        }
    }

    // Shutdown
    info!("Shutting down server...");
    provider.shutdown().await?;

    info!("Server stopped. Example completed!");
    Ok(())
}
