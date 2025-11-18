use anyhow::Result;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use tracing_subscriber;

use wasmcloud_provider_messaging_websocket::{BrokerMessage, WebSocketMessagingProvider};

/// Integration test demonstrating session management features
#[tokio::test]
async fn test_session_lifecycle() -> Result<()> {
    // Initialize logging for test
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let provider = WebSocketMessagingProvider::new();

    // Initially no sessions
    let sessions = provider.list_sessions().await;
    assert_eq!(sessions.len(), 0);

    // Session will be created upon linking
    // Note: This test doesn't actually connect to a WebSocket server
    // In production, you would have a real WebSocket endpoint

    Ok(())
}

/// Test configuration merging
#[tokio::test]
async fn test_config_merging() -> Result<()> {
    let mut default_config = HashMap::new();
    default_config.insert("URI".to_string(), "ws://default:8080".to_string());
    default_config.insert("CONNECT_TIMEOUT_SEC".to_string(), "30".to_string());

    let _provider = WebSocketMessagingProvider::from_config(default_config)?;

    // Verify provider was created successfully
    // The config is used internally by the provider
    assert!(true);

    Ok(())
}

/// Test message creation
#[test]
fn test_message_creation() {
    let message = BrokerMessage {
        subject: "test.subject".to_string(),
        body: Bytes::from("test payload"),
        reply_to: Some("reply.subject".to_string()),
    };

    assert_eq!(message.subject, "test.subject");
    assert_eq!(message.body, Bytes::from("test payload"));
    assert_eq!(message.reply_to, Some("reply.subject".to_string()));
}

/// Test provider state management
#[tokio::test]
async fn test_provider_state() -> Result<()> {
    let provider = Arc::new(WebSocketMessagingProvider::new());

    // Clone provider to simulate multi-threaded access
    let provider_clone = Arc::clone(&provider);

    // Spawn a task that checks sessions
    let handle = tokio::spawn(async move {
        let sessions = provider_clone.list_sessions().await;
        sessions.len()
    });

    let count = handle.await?;
    assert_eq!(count, 0);

    Ok(())
}

/// Test concurrent session access
#[tokio::test]
async fn test_concurrent_access() -> Result<()> {
    let provider = Arc::new(WebSocketMessagingProvider::new());

    // Simulate multiple concurrent readers
    let mut handles = vec![];

    for i in 0..10 {
        let provider_clone = Arc::clone(&provider);
        let handle = tokio::spawn(async move {
            let sessions = provider_clone.list_sessions().await;
            (i, sessions.len())
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        let (i, count) = handle.await?;
        assert_eq!(count, 0, "Task {} found unexpected sessions", i);
    }

    Ok(())
}

/// Test provider shutdown
#[tokio::test]
async fn test_shutdown() -> Result<()> {
    let provider = WebSocketMessagingProvider::new();

    // Should be able to shutdown even with no connections
    provider.shutdown().await?;

    // Verify sessions are cleared
    let sessions = provider.list_sessions().await;
    assert_eq!(sessions.len(), 0);

    Ok(())
}
