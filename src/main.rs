use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

use wasmcloud_provider_messaging_websocket::WebSocketMessagingProvider;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Starting WebSocket Messaging Provider");

    // Create provider instance
    let provider = WebSocketMessagingProvider::new();

    // In a real wasmCloud integration, this would:
    // 1. Register with wasmCloud host
    // 2. Listen for link configurations
    // 3. Handle component invocations
    // For now, this is a standalone demonstration

    info!("WebSocket Messaging Provider initialized");
    info!("Waiting for shutdown signal...");

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    info!("Shutdown signal received");
    provider.shutdown().await?;

    Ok(())
}
