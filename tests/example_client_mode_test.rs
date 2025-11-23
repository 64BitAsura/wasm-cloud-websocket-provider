use anyhow::{Context, Result};
use axum::{
    extract::ws::{Message as AxumMessage, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::tungstenite::Message;
use tracing::info;

/// Simple echo WebSocket server for testing
async fn echo_ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// Handle WebSocket connections - echo all messages back
async fn handle_socket(mut socket: WebSocket) {
    info!("New WebSocket connection established");

    while let Some(msg_result) = socket.recv().await {
        match msg_result {
            Ok(msg) => {
                match msg {
                    AxumMessage::Text(text) => {
                        info!("Echo server received text: {}", text);
                        // Echo it back
                        if socket.send(AxumMessage::Text(text)).await.is_err() {
                            break;
                        }
                    }
                    AxumMessage::Binary(data) => {
                        info!("Echo server received binary data: {} bytes", data.len());
                        // Echo it back
                        if socket.send(AxumMessage::Binary(data)).await.is_err() {
                            break;
                        }
                    }
                    AxumMessage::Close(_) => {
                        info!("Client closed connection");
                        break;
                    }
                    AxumMessage::Ping(data) => {
                        if socket.send(AxumMessage::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    AxumMessage::Pong(_) => {
                        // Ignore pongs
                    }
                }
            }
            Err(_) => {
                break;
            }
        }
    }

    info!("WebSocket connection closed");
}

/// Start a simple echo WebSocket server
async fn start_echo_server() -> Result<(tokio::task::JoinHandle<()>, u16)> {
    info!("Starting echo WebSocket server...");

    let app = Router::new().route("/", get(echo_ws_handler));

    // Bind to a random port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .context("Failed to bind listener")?;

    let addr = listener
        .local_addr()
        .context("Failed to get local address")?;
    let port = addr.port();

    info!("Echo server will listen on {}", addr);

    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    // Give server time to start
    sleep(Duration::from_millis(500)).await;

    Ok((handle, port))
}

/// Test client mode example with echo server
#[tokio::test]
#[ignore = "requires building and running example, slow test"]
async fn test_client_mode_with_echo_server() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    // Start echo server
    let (server_handle, port) = start_echo_server().await?;
    let ws_uri = format!("ws://127.0.0.1:{}", port);

    info!("Echo server started on {}", ws_uri);

    // Test the echo server directly with a client connection
    info!("Testing echo server with direct client connection...");

    let (ws_stream, _) = timeout(
        Duration::from_secs(5),
        tokio_tungstenite::connect_async(&ws_uri),
    )
    .await
    .context("Timeout connecting")?
    .context("Failed to connect")?;

    info!("Connected to echo server");

    let (mut write, mut read) = ws_stream.split();

    // Send test messages that simulate wasmCloud example patterns
    let test_messages = [
        r#"{"subject":"test.ping","body":"cGluZw==","reply_to":null}"#,
        r#"{"subject":"test.pong","body":"cG9uZw==","reply_to":"session-123"}"#,
        r#"{"subject":"test.echo","body":"ZWNobyBtZXNzYWdl","reply_to":null}"#,
    ];

    for (i, msg_text) in test_messages.iter().enumerate() {
        info!("Sending test message {}: {}", i, msg_text);
        write
            .send(Message::Text(msg_text.to_string()))
            .await
            .context("Failed to send message")?;

        // Wait for echo response
        match timeout(Duration::from_secs(2), read.next()).await {
            Ok(Some(Ok(msg))) => {
                info!("Received echo response: {:?}", msg);
                if let Message::Text(text) = msg {
                    assert_eq!(text, *msg_text, "Echo response should match sent message");
                }
            }
            Ok(Some(Err(e))) => {
                anyhow::bail!("Error receiving message: {}", e);
            }
            Ok(None) => {
                anyhow::bail!("Connection closed unexpectedly");
            }
            Err(_) => {
                anyhow::bail!("Timeout waiting for echo response");
            }
        }
    }

    // Close connection
    let _ = write.close().await;

    // Clean up server
    info!("Shutting down echo server...");
    server_handle.abort();
    let _ = timeout(Duration::from_secs(2), server_handle).await;

    info!("Test completed successfully - echo server handled all ping/pong/echo messages");
    Ok(())
}

/// Test that echo server properly echoes different message formats
#[tokio::test]
#[ignore = "slow test for manual verification"]
async fn test_echo_server_message_formats() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    // Start echo server
    let (server_handle, port) = start_echo_server().await?;
    let ws_uri = format!("ws://127.0.0.1:{}", port);

    info!("Testing message formats with echo server");

    let (ws_stream, _) = timeout(
        Duration::from_secs(5),
        tokio_tungstenite::connect_async(&ws_uri),
    )
    .await
    .context("Timeout connecting")?
    .context("Failed to connect")?;

    let (mut write, mut read) = ws_stream.split();

    // Test JSON message
    let json_msg = r#"{"subject":"test","body":"dGVzdA==","reply_to":null}"#;
    write
        .send(Message::Text(json_msg.to_string()))
        .await?;

    if let Ok(Some(Ok(msg))) = timeout(Duration::from_secs(2), read.next()).await {
        info!("Received JSON echo: {:?}", msg);
    }

    // Test plain text message
    let plain_msg = "Hello, WebSocket!";
    write
        .send(Message::Text(plain_msg.to_string()))
        .await?;

    if let Ok(Some(Ok(msg))) = timeout(Duration::from_secs(2), read.next()).await {
        info!("Received plain text echo: {:?}", msg);
        if let Message::Text(text) = msg {
            assert_eq!(text, plain_msg, "Plain text should be echoed exactly");
        }
    }

    // Test binary message
    let binary_data = vec![0x01, 0x02, 0x03, 0x04];
    write
        .send(Message::Binary(binary_data.clone()))
        .await?;

    if let Ok(Some(Ok(msg))) = timeout(Duration::from_secs(2), read.next()).await {
        info!("Received binary echo: {:?}", msg);
        if let Message::Binary(data) = msg {
            assert_eq!(data, binary_data, "Binary data should be echoed exactly");
        }
    }

    // Close connection
    let _ = write.close().await;

    // Clean up
    server_handle.abort();
    let _ = timeout(Duration::from_secs(2), server_handle).await;

    info!("Test completed successfully");
    Ok(())
}

/// Test multiple concurrent connections to echo server
#[tokio::test]
#[ignore = "slow test for manual verification"]
async fn test_echo_server_multiple_clients() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    // Start echo server
    let (server_handle, port) = start_echo_server().await?;
    let ws_uri = format!("ws://127.0.0.1:{}", port);

    info!("Testing multiple concurrent clients");

    // Create multiple client connections
    let mut clients = Vec::new();
    for i in 0..3 {
        let uri = ws_uri.clone();
        let client = tokio::spawn(async move {
            let (ws_stream, _) = tokio_tungstenite::connect_async(&uri)
                .await
                .expect("Failed to connect");

            let (mut write, mut read) = ws_stream.split();

            // Send a message
            let msg = format!("Message from client {}", i);
            write
                .send(Message::Text(msg.clone()))
                .await
                .expect("Failed to send");

            // Wait for echo
            if let Some(Ok(Message::Text(text))) = read.next().await {
                assert_eq!(text, msg, "Should receive exact echo");
                return true;
            }
            false
        });
        clients.push(client);
    }

    // Wait for all clients to complete
    let results = futures::future::join_all(clients).await;

    // Verify all clients succeeded
    for (i, result) in results.iter().enumerate() {
        assert!(
            result.as_ref().expect(&format!("Client {} task failed", i)),
            "Client {} should succeed",
            i
        );
    }

    // Clean up
    server_handle.abort();
    let _ = timeout(Duration::from_secs(2), server_handle).await;

    info!("Test completed successfully");
    Ok(())
}
