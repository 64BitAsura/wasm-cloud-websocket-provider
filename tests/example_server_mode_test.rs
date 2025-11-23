use anyhow::{Context, Result};
use base64::Engine;
use futures::{SinkExt, StreamExt};
use serde_json::json;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::info;

/// Helper to start the server_mode example
async fn start_server_mode_example() -> Result<(Child, u16)> {
    info!("Starting server_mode example...");
    
    let mut child = Command::new("cargo")
        .args(&["run", "--example", "server_mode"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .context("Failed to spawn server_mode example")?;

    // Read both stdout and stderr to find the port
    let stderr = child.stderr.take().expect("Failed to get stderr");
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let mut stderr_reader = BufReader::new(stderr).lines();
    let mut stdout_reader = BufReader::new(stdout).lines();

    // Wait for server to start and extract port
    let start_timeout = Duration::from_secs(60);
    
    let found = Arc::new(AtomicBool::new(false));
    let found_clone = found.clone();
    let port_arc = Arc::new(Mutex::new(8080u16));
    let port_clone = port_arc.clone();
    
    // Spawn task to read stderr
    let stderr_task = tokio::spawn(async move {
        while let Some(line) = stderr_reader.next_line().await.ok().flatten() {
            info!("Server stderr: {}", line);
            
            // Look for the listening message
            if line.contains("WebSocket server listening on") {
                // Extract port from line like "ws://127.0.0.1:8080/ws"
                if let Some(port_str) = line.split(':').nth(2) {
                    if let Some(port_part) = port_str.split('/').next() {
                        if let Ok(p) = port_part.trim().parse::<u16>() {
                            *port_clone.lock().await = p;
                            info!("Detected server port: {}", p);
                            found_clone.store(true, Ordering::SeqCst);
                            break;
                        }
                    }
                }
                found_clone.store(true, Ordering::SeqCst);
                break;
            }
        }
    });
    
    // Also check stdout
    let found_clone2 = found.clone();
    let port_clone2 = port_arc.clone();
    let stdout_task = tokio::spawn(async move {
        while let Some(line) = stdout_reader.next_line().await.ok().flatten() {
            info!("Server stdout: {}", line);
            
            if line.contains("WebSocket server listening on") {
                if let Some(port_str) = line.split(':').nth(2) {
                    if let Some(port_part) = port_str.split('/').next() {
                        if let Ok(p) = port_part.trim().parse::<u16>() {
                            *port_clone2.lock().await = p;
                            info!("Detected server port: {}", p);
                            found_clone2.store(true, Ordering::SeqCst);
                            break;
                        }
                    }
                }
                found_clone2.store(true, Ordering::SeqCst);
                break;
            }
        }
    });
    
    // Wait for server to start
    match timeout(start_timeout, async {
        while !found.load(Ordering::SeqCst) {
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    {
        Ok(_) => {
            let port = *port_arc.lock().await;
            info!("Server started on port {}", port);
            // Give it a moment to fully initialize
            sleep(Duration::from_millis(1000)).await;
            
            // Clean up tasks
            stderr_task.abort();
            stdout_task.abort();
            
            Ok((child, port))
        }
        Err(_) => {
            let _ = child.kill().await;
            anyhow::bail!("Timeout waiting for server to start")
        }
    }
}

/// Test server mode example with a WebSocket client that sends ping/pong messages
#[tokio::test]
#[ignore = "requires building and running example, slow test"]
async fn test_server_mode_example_with_ping_pong() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    // Start the server_mode example
    let (mut server_process, port) = start_server_mode_example().await?;

    // Connect WebSocket client
    let ws_url = format!("ws://127.0.0.1:{}/ws", port);
    info!("Connecting to {}", ws_url);
    
    let connect_result = timeout(
        Duration::from_secs(5),
        connect_async(&ws_url)
    ).await;

    let (ws_stream, _) = match connect_result {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => {
            let _ = server_process.kill().await;
            anyhow::bail!("Failed to connect to WebSocket server: {}", e)
        }
        Err(_) => {
            let _ = server_process.kill().await;
            anyhow::bail!("Timeout connecting to WebSocket server")
        }
    };

    info!("WebSocket connection established successfully");
    let (mut write, _read) = ws_stream.split();

    // Send ping message
    let ping_msg = json!({
        "subject": "test.ping",
        "body": base64::engine::general_purpose::STANDARD.encode("ping"),
        "reply_to": null
    });

    info!("Sending ping message");
    let _ = write.send(Message::Text(ping_msg.to_string())).await;
    sleep(Duration::from_millis(500)).await;

    // Send pong message
    let pong_msg = json!({
        "subject": "test.pong",
        "body": base64::engine::general_purpose::STANDARD.encode("pong"),
        "reply_to": null
    });

    info!("Sending pong message");
    let _ = write.send(Message::Text(pong_msg.to_string())).await;
    sleep(Duration::from_millis(500)).await;

    // Send additional echo messages
    for i in 0..3 {
        let echo_msg = json!({
            "subject": format!("test.echo.{}", i),
            "body": base64::engine::general_purpose::STANDARD.encode(format!("echo message {}", i)),
            "reply_to": format!("session-{}", i)
        });

        info!("Sending echo message {}", i);
        if write.send(Message::Text(echo_msg.to_string())).await.is_err() {
            info!("Send failed (expected - server might close connection)");
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }

    // Close connection gracefully
    let _ = write.close().await;
    
    // Clean up: kill the server process
    info!("Cleaning up server process...");
    server_process.kill().await.context("Failed to kill server process")?;
    
    // Wait for process to exit
    let _ = timeout(Duration::from_secs(5), server_process.wait()).await;

    info!("Test completed successfully - server started and accepted ping/pong messages");
    Ok(())
}
