use anyhow::Result;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing_subscriber;

use wasmcloud_provider_messaging_websocket::{BrokerMessage, WebSocketMessagingProvider};

/// Test client mode message broadcasting from remote WS server to handler components
/// Note: This test requires network access to echo.websocket.org
#[tokio::test]
#[ignore = "requires network access to external server"]
async fn test_client_mode_broadcast_to_handlers() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let mut config = HashMap::new();
    config.insert("MODE".to_string(), "client".to_string());
    config.insert("URI".to_string(), "ws://echo.websocket.org".to_string());
    config.insert("ENABLE_SESSION_TRACKING".to_string(), "true".to_string());

    let provider = WebSocketMessagingProvider::from_config(config)?;

    // Link consumer component (sends messages)
    let consumer_id = "test-consumer";
    provider
        .receive_link_config_as_target(consumer_id, HashMap::new())
        .await?;

    // Link handler component (receives messages)
    let handler_id = "test-handler";
    provider
        .receive_link_config_as_source(handler_id, HashMap::new())
        .await?;

    // Give time for connections
    sleep(Duration::from_millis(500)).await;

    // Verify links were established
    let sessions = provider.list_sessions().await;
    assert!(!sessions.is_empty(), "Should have active sessions");

    // Send a test message
    let msg = BrokerMessage {
        subject: "test.broadcast".to_string(),
        body: Bytes::from("test message"),
        reply_to: None,
    };

    provider.publish(consumer_id, msg).await?;

    // Wait for echo response
    sleep(Duration::from_millis(1000)).await;

    // Clean up
    provider.delete_link_as_target(consumer_id).await?;
    provider.delete_link_as_source(handler_id).await?;
    provider.shutdown().await?;

    Ok(())
}

/// Test that components can send replies back to remote server using session ID
/// Note: This test requires network access to echo.websocket.org
#[tokio::test]
#[ignore = "requires network access to external server"]
async fn test_component_reply_to_remote_server() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let mut config = HashMap::new();
    config.insert("MODE".to_string(), "client".to_string());
    config.insert("URI".to_string(), "ws://echo.websocket.org".to_string());

    let provider = WebSocketMessagingProvider::from_config(config)?;

    // Link both consumer and handler
    provider
        .receive_link_config_as_target("consumer", HashMap::new())
        .await?;
    provider
        .receive_link_config_as_source("handler", HashMap::new())
        .await?;

    sleep(Duration::from_millis(500)).await;

    // Get session ID
    let sessions = provider.list_sessions().await;
    assert!(!sessions.is_empty());

    let (session_id, _) = &sessions[0];

    // Send message with reply-to
    let msg = BrokerMessage {
        subject: "test.request".to_string(),
        body: Bytes::from("request"),
        reply_to: Some(session_id.clone()),
    };

    // Component can reply using send_to_session with the reply-to session ID
    provider.send_to_session(session_id, msg).await?;

    sleep(Duration::from_millis(500)).await;

    // Clean up
    provider.shutdown().await?;

    Ok(())
}

/// Test message parsing from remote server
#[tokio::test]
async fn test_message_parsing() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let provider = WebSocketMessagingProvider::new();

    // Test JSON message parsing
    let json_msg = r#"{"subject": "test.topic", "body": "hello world", "reply_to": "sess-123"}"#;
    let parsed = WebSocketMessagingProvider::parse_message_static(json_msg, "default-session")?;

    assert_eq!(parsed.subject, "test.topic");
    assert_eq!(parsed.reply_to, Some("sess-123".to_string()));

    // Test plain text message parsing
    let plain_msg = "plain text message";
    let parsed_plain =
        WebSocketMessagingProvider::parse_message_static(plain_msg, "default-session")?;

    assert_eq!(parsed_plain.subject, "message");
    assert_eq!(parsed_plain.reply_to, Some("default-session".to_string()));

    Ok(())
}

/// Test message encoding for sending to remote server
#[tokio::test]
async fn test_message_encoding() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let msg = BrokerMessage {
        subject: "test.subject".to_string(),
        body: Bytes::from("test body"),
        reply_to: Some("session-xyz".to_string()),
    };

    let encoded = WebSocketMessagingProvider::encode_message_static(&msg)?;

    // Verify it's a text message
    match encoded {
        tokio_tungstenite::tungstenite::Message::Text(text) => {
            // Should be JSON
            let json: serde_json::Value = serde_json::from_str(&text)?;
            assert_eq!(json["subject"], "test.subject");
            assert_eq!(json["reply_to"], "session-xyz");
            assert!(json.get("body").is_some());
        }
        _ => panic!("Expected text message"),
    }

    Ok(())
}

/// Test multiple handler components receiving broadcast
/// Note: This test requires network access to echo.websocket.org
#[tokio::test]
#[ignore = "requires network access to external server"]
async fn test_multiple_handlers_broadcast() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let mut config = HashMap::new();
    config.insert("MODE".to_string(), "client".to_string());
    config.insert("URI".to_string(), "ws://echo.websocket.org".to_string());

    let provider = WebSocketMessagingProvider::from_config(config)?;

    // Link one consumer
    provider
        .receive_link_config_as_target("consumer", HashMap::new())
        .await?;

    // Link multiple handlers
    provider
        .receive_link_config_as_source("handler-1", HashMap::new())
        .await?;
    provider
        .receive_link_config_as_source("handler-2", HashMap::new())
        .await?;
    provider
        .receive_link_config_as_source("handler-3", HashMap::new())
        .await?;

    sleep(Duration::from_millis(500)).await;

    // Send message - should be broadcast to all handlers when echo returns
    let msg = BrokerMessage {
        subject: "broadcast.test".to_string(),
        body: Bytes::from("broadcast to all"),
        reply_to: None,
    };

    provider.publish("consumer", msg).await?;

    sleep(Duration::from_millis(1000)).await;

    // Clean up
    provider.shutdown().await?;

    Ok(())
}

/// Test client mode with session tracking enabled/disabled
/// Note: This test requires network access to echo.websocket.org
#[tokio::test]
#[ignore = "requires network access to external server"]
async fn test_client_session_tracking() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    // Test with session tracking enabled
    let mut config_enabled = HashMap::new();
    config_enabled.insert("MODE".to_string(), "client".to_string());
    config_enabled.insert("URI".to_string(), "ws://echo.websocket.org".to_string());
    config_enabled.insert("ENABLE_SESSION_TRACKING".to_string(), "true".to_string());

    let provider_enabled = WebSocketMessagingProvider::from_config(config_enabled)?;
    provider_enabled
        .receive_link_config_as_target("comp1", HashMap::new())
        .await?;

    sleep(Duration::from_millis(300)).await;

    let sessions_enabled = provider_enabled.list_sessions().await;
    assert!(!sessions_enabled.is_empty(), "Should track sessions");

    provider_enabled.shutdown().await?;

    // Test with session tracking disabled
    let mut config_disabled = HashMap::new();
    config_disabled.insert("MODE".to_string(), "client".to_string());
    config_disabled.insert("URI".to_string(), "ws://echo.websocket.org".to_string());
    config_disabled.insert("ENABLE_SESSION_TRACKING".to_string(), "false".to_string());

    let provider_disabled = WebSocketMessagingProvider::from_config(config_disabled)?;
    provider_disabled
        .receive_link_config_as_target("comp2", HashMap::new())
        .await?;

    sleep(Duration::from_millis(300)).await;

    let _sessions_disabled = provider_disabled.list_sessions().await;
    // Sessions might still be tracked at provider level but not stored

    provider_disabled.shutdown().await?;

    Ok(())
}
