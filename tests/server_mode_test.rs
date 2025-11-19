use anyhow::Result;
use bytes::Bytes;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tracing_subscriber;

use wasmcloud_provider_messaging_websocket::{BrokerMessage, WebSocketMessagingProvider};

/// Test server mode initialization
#[tokio::test]
async fn test_server_mode_initialization() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let mut config = HashMap::new();
    config.insert("MODE".to_string(), "server".to_string());
    config.insert("URI".to_string(), "127.0.0.1:0".to_string()); // Use port 0 for random available port

    let mut provider = WebSocketMessagingProvider::from_config(config)?;
    provider.start_server_if_needed().await?;

    // Verify server is running
    let addr = provider.get_server_addr().await;
    assert!(addr.is_some(), "Server should have an address");

    // Clean up
    provider.shutdown().await?;

    Ok(())
}

/// Test client mode (existing functionality)
#[tokio::test]
async fn test_client_mode() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let mut config = HashMap::new();
    config.insert("MODE".to_string(), "client".to_string());
    config.insert("URI".to_string(), "ws://echo.websocket.org".to_string());

    let mut provider = WebSocketMessagingProvider::from_config(config)?;
    provider.start_server_if_needed().await?;

    // In client mode, no server should be started
    let addr = provider.get_server_addr().await;
    assert!(addr.is_none(), "Client mode should not have server address");

    provider.shutdown().await?;

    Ok(())
}

/// Test sending to multiple session types
#[tokio::test]
async fn test_session_routing() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let mut config = HashMap::new();
    config.insert("MODE".to_string(), "server".to_string());
    config.insert("URI".to_string(), "127.0.0.1:0".to_string());

    let mut provider = WebSocketMessagingProvider::from_config(config)?;
    provider.start_server_if_needed().await?;

    // List sessions should work even if empty
    let sessions = provider.list_sessions().await;
    assert_eq!(sessions.len(), 0);

    provider.shutdown().await?;

    Ok(())
}

/// Test broadcast functionality in server mode
#[tokio::test]
async fn test_broadcast_server_mode() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let mut config = HashMap::new();
    config.insert("MODE".to_string(), "server".to_string());
    config.insert("URI".to_string(), "127.0.0.1:0".to_string());

    let mut provider = WebSocketMessagingProvider::from_config(config)?;
    provider.start_server_if_needed().await?;

    // Should be able to broadcast even with no clients
    let msg = BrokerMessage {
        subject: "test.broadcast".to_string(),
        body: Bytes::from("broadcast message"),
        reply_to: None,
    };

    // This should succeed even with no clients
    let result = provider.broadcast_to_clients(msg).await;
    assert!(result.is_ok());

    provider.shutdown().await?;

    Ok(())
}

/// Test listing WebSocket clients in server mode
#[tokio::test]
async fn test_list_ws_clients() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let mut config = HashMap::new();
    config.insert("MODE".to_string(), "server".to_string());
    config.insert("URI".to_string(), "127.0.0.1:0".to_string());

    let mut provider = WebSocketMessagingProvider::from_config(config)?;
    provider.start_server_if_needed().await?;

    // List WS clients should work
    let clients = provider.list_ws_clients().await?;
    assert_eq!(clients.len(), 0);

    provider.shutdown().await?;

    Ok(())
}

/// Test provider shutdown in server mode
#[tokio::test]
async fn test_server_mode_shutdown() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let mut config = HashMap::new();
    config.insert("MODE".to_string(), "server".to_string());
    config.insert("URI".to_string(), "127.0.0.1:0".to_string());

    let mut provider = WebSocketMessagingProvider::from_config(config)?;
    provider.start_server_if_needed().await?;

    let addr = provider.get_server_addr().await;
    assert!(addr.is_some());

    // Shutdown should clean up everything
    provider.shutdown().await?;

    // Give a moment for cleanup
    sleep(Duration::from_millis(100)).await;

    // Verify sessions are cleared
    let sessions = provider.list_sessions().await;
    assert_eq!(sessions.len(), 0);

    Ok(())
}

/// Test reply-to field handling
#[tokio::test]
async fn test_reply_to_handling() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    // Create message with reply-to
    let msg = BrokerMessage {
        subject: "test.request".to_string(),
        body: Bytes::from("request data"),
        reply_to: Some("session-123".to_string()),
    };

    assert_eq!(msg.reply_to, Some("session-123".to_string()));

    // Create message without reply-to
    let msg2 = BrokerMessage {
        subject: "test.publish".to_string(),
        body: Bytes::from("publish data"),
        reply_to: None,
    };

    assert_eq!(msg2.reply_to, None);

    Ok(())
}

/// Test session tracking configuration
#[tokio::test]
async fn test_session_tracking_config() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    // With session tracking enabled (default)
    let mut config1 = HashMap::new();
    config1.insert("MODE".to_string(), "server".to_string());
    config1.insert("URI".to_string(), "127.0.0.1:0".to_string());
    config1.insert("ENABLE_SESSION_TRACKING".to_string(), "true".to_string());

    let mut provider1 = WebSocketMessagingProvider::from_config(config1)?;
    provider1.start_server_if_needed().await?;
    provider1.shutdown().await?;

    // With session tracking disabled
    let mut config2 = HashMap::new();
    config2.insert("MODE".to_string(), "server".to_string());
    config2.insert("URI".to_string(), "127.0.0.1:0".to_string());
    config2.insert("ENABLE_SESSION_TRACKING".to_string(), "false".to_string());

    let mut provider2 = WebSocketMessagingProvider::from_config(config2)?;
    provider2.start_server_if_needed().await?;
    provider2.shutdown().await?;

    Ok(())
}
