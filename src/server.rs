use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
    routing::get,
    Router,
};
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{BrokerMessage, SessionInfo};

/// Client connection state for server mode
#[derive(Debug)]
pub struct ServerClientConnection {
    pub tx: mpsc::UnboundedSender<Message>,
    #[allow(dead_code)]
    pub session_info: SessionInfo,
}

/// WebSocket server state
#[derive(Clone)]
pub struct ServerState {
    /// Active client connections indexed by session ID
    pub clients: Arc<RwLock<HashMap<String, ServerClientConnection>>>,
    /// Component ID that handles incoming messages
    #[allow(dead_code)]
    pub component_id: Arc<RwLock<Option<String>>>,
    /// Callback for handling incoming messages
    pub message_handler: Arc<dyn Fn(String, BrokerMessage) -> Result<()> + Send + Sync>,
}

impl ServerState {
    pub fn new<F>(message_handler: F) -> Self
    where
        F: Fn(String, BrokerMessage) -> Result<()> + Send + Sync + 'static,
    {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            component_id: Arc::new(RwLock::new(None)),
            message_handler: Arc::new(message_handler),
        }
    }

    /// Set the component that will handle messages from clients
    #[allow(dead_code)]
    pub async fn set_handler_component(&self, component_id: String) {
        let mut comp = self.component_id.write().await;
        *comp = Some(component_id);
    }

    /// Get list of active client sessions
    pub async fn list_client_sessions(&self) -> Vec<String> {
        let clients = self.clients.read().await;
        clients.keys().cloned().collect()
    }

    /// Send message to a specific client session
    pub async fn send_to_client(&self, session_id: &str, msg: Message) -> Result<()> {
        let clients = self.clients.read().await;
        if let Some(client) = clients.get(session_id) {
            client
                .tx
                .send(msg)
                .context("Failed to send message to client")?;
            Ok(())
        } else {
            anyhow::bail!("Client session not found: {}", session_id)
        }
    }

    /// Broadcast message to all connected clients
    pub async fn broadcast(&self, msg: Message) -> Result<()> {
        let clients = self.clients.read().await;
        for (session_id, client) in clients.iter() {
            if let Err(e) = client.tx.send(msg.clone()) {
                warn!("Failed to send to session {}: {}", session_id, e);
            }
        }
        Ok(())
    }

    /// Remove a client session
    async fn remove_client(&self, session_id: &str) {
        let mut clients = self.clients.write().await;
        clients.remove(session_id);
        info!("Client disconnected: {}", session_id);
    }
}

/// Start WebSocket server
pub async fn start_server(
    bind_addr: &str,
    state: ServerState,
) -> Result<(SocketAddr, JoinHandle<Result<()>>)> {
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state.clone());

    // Parse bind address
    let addr: SocketAddr = bind_addr.parse().context("Invalid bind address")?;

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("Failed to bind to address")?;

    let local_addr = listener.local_addr()?;
    info!("WebSocket server listening on {}", local_addr);

    // Spawn server task
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.context("Server error")?;
        Ok(())
    });

    Ok((local_addr, handle))
}

/// WebSocket upgrade handler
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<ServerState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: ServerState) {
    let session_id = Uuid::new_v4().to_string();
    info!("New WebSocket client connected: {}", session_id);

    let (mut ws_tx, mut ws_rx) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Create session info
    let session_info = SessionInfo {
        session_id: session_id.clone(),
        connected_at: std::time::SystemTime::now(),
        metadata: HashMap::new(),
    };

    // Register client
    {
        let mut clients = state.clients.write().await;
        clients.insert(
            session_id.clone(),
            ServerClientConnection {
                tx: tx.clone(),
                session_info,
            },
        );
    }

    // Clone for the tasks
    let session_id_send = session_id.clone();
    let session_id_recv = session_id.clone();
    let state_recv = state.clone();
    let state_cleanup = state.clone();

    // Spawn task to send messages to client
    let send_handle = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = ws_tx.send(msg).await {
                error!("Failed to send to client {}: {}", session_id_send, e);
                break;
            }
        }
    });

    // Handle incoming messages from client
    let recv_handle = tokio::spawn(async move {
        while let Some(msg_result) = ws_rx.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    debug!("Received text message from {}: {}", session_id_recv, text);

                    // Parse message and forward to handler
                    if let Ok(broker_msg) = parse_broker_message(&text, &session_id_recv) {
                        if let Err(e) =
                            (state_recv.message_handler)(session_id_recv.clone(), broker_msg)
                        {
                            error!("Message handler error: {}", e);
                        }
                    } else {
                        warn!("Failed to parse message from client");
                    }
                }
                Ok(Message::Binary(data)) => {
                    debug!(
                        "Received binary message from {}: {} bytes",
                        session_id_recv,
                        data.len()
                    );

                    // Try to parse as JSON or handle as raw binary
                    if let Ok(text) = String::from_utf8(data.clone()) {
                        if let Ok(broker_msg) = parse_broker_message(&text, &session_id_recv) {
                            if let Err(e) =
                                (state_recv.message_handler)(session_id_recv.clone(), broker_msg)
                            {
                                error!("Message handler error: {}", e);
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("Client {} closed connection", session_id_recv);
                    break;
                }
                Ok(Message::Ping(data)) => {
                    if let Err(e) = tx.send(Message::Pong(data)) {
                        error!("Failed to send pong: {}", e);
                        break;
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    error!("WebSocket error for client {}: {}", session_id_recv, e);
                    break;
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_handle => {
            debug!("Send task completed for {}", session_id);
        }
        _ = recv_handle => {
            debug!("Receive task completed for {}", session_id);
        }
    }

    // Clean up client
    state_cleanup.remove_client(&session_id).await;
}

/// Parse a text message into a BrokerMessage
fn parse_broker_message(text: &str, session_id: &str) -> Result<BrokerMessage> {
    // Try to parse as JSON
    let json: serde_json::Value = serde_json::from_str(text)?;

    let subject = json
        .get("subject")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    let body = if let Some(body_str) = json.get("body").and_then(|v| v.as_str()) {
        // Try to decode from base64/hex
        Bytes::from(body_str.as_bytes().to_vec())
    } else if let Some(body_arr) = json.get("body").and_then(|v| v.as_array()) {
        // Array of bytes
        let bytes: Vec<u8> = body_arr
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n as u8))
            .collect();
        Bytes::from(bytes)
    } else {
        Bytes::from(text.as_bytes().to_vec())
    };

    // If no reply_to is provided, use the session_id so component can reply
    let reply_to = json
        .get("reply_to")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| Some(session_id.to_string()));

    Ok(BrokerMessage {
        subject,
        body,
        reply_to,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_broker_message() {
        let json = r#"{"subject": "test.topic", "body": "hello", "reply_to": "session-123"}"#;
        let msg = parse_broker_message(json, "sess-1").unwrap();
        assert_eq!(msg.subject, "test.topic");
        assert_eq!(msg.reply_to, Some("session-123".to_string()));
    }

    #[test]
    fn test_parse_broker_message_no_reply() {
        let json = r#"{"subject": "test.topic", "body": "hello"}"#;
        let msg = parse_broker_message(json, "sess-1").unwrap();
        assert_eq!(msg.subject, "test.topic");
        assert_eq!(msg.reply_to, Some("sess-1".to_string()));
    }

    #[tokio::test]
    async fn test_server_state() {
        let state = ServerState::new(|_session_id, _msg| Ok(()));
        assert_eq!(state.list_client_sessions().await.len(), 0);

        state.set_handler_component("comp-1".to_string()).await;
        let comp = state.component_id.read().await;
        assert_eq!(*comp, Some("comp-1".to_string()));
    }
}
