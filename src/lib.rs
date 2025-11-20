use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Context as AnyhowContext, Result};
use axum::extract::ws::Message as AxumMessage;
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, instrument};
use url::Url;

mod connection;
mod server;

use connection::{ConnectionConfig, ConnectionMode};
use server::{start_server, ServerState};

// Re-export for main binary
pub use connection::ConnectionConfig as WsConnectionConfig;

/// Type alias for message handler callback
type MessageHandler = Arc<dyn Fn(String, BrokerMessage) -> Result<()> + Send + Sync>;

/// Message type for internal communication
#[derive(Debug, Clone)]
pub struct BrokerMessage {
    pub subject: String,
    pub body: Bytes,
    pub reply_to: Option<String>,
}

/// Session information for a WebSocket connection
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub connected_at: std::time::SystemTime,
    pub metadata: HashMap<String, String>,
}

/// WebSocket client bundle containing connection and session info
#[derive(Debug)]
pub struct WebSocketClientBundle {
    pub tx: mpsc::UnboundedSender<Message>,
    pub session_info: SessionInfo,
    pub handle: JoinHandle<()>,
}

impl Drop for WebSocketClientBundle {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// WebSocket implementation for wasmcloud:messaging
#[derive(Clone)]
pub struct WebSocketMessagingProvider {
    /// Components that can receive messages (consumers)
    consumer_components: Arc<RwLock<HashMap<String, WebSocketClientBundle>>>,
    /// Components that can handle messages (handlers)
    handler_components: Arc<RwLock<HashMap<String, WebSocketClientBundle>>>,
    /// Default configuration
    default_config: ConnectionConfig,
    /// Session storage for tracking WebSocket connections by session ID
    session_storage: Arc<RwLock<HashMap<String, String>>>, // session_id -> component_id
    /// Server state for server mode
    server_state: Option<Arc<ServerState>>,
    /// Server handle for cleanup
    server_handle: Arc<RwLock<Option<JoinHandle<Result<()>>>>>,
    /// Server address when running in server mode
    server_addr: Arc<RwLock<Option<SocketAddr>>>,
    /// Message handler for broadcasting messages from remote WS server to components (client mode)
    client_message_handler: Arc<RwLock<Option<MessageHandler>>>,
}

impl Default for WebSocketMessagingProvider {
    fn default() -> Self {
        Self {
            consumer_components: Arc::new(RwLock::new(HashMap::new())),
            handler_components: Arc::new(RwLock::new(HashMap::new())),
            default_config: ConnectionConfig::default(),
            session_storage: Arc::new(RwLock::new(HashMap::new())),
            server_state: None,
            server_handle: Arc::new(RwLock::new(None)),
            server_addr: Arc::new(RwLock::new(None)),
            client_message_handler: Arc::new(RwLock::new(None)),
        }
    }
}

impl WebSocketMessagingProvider {
    /// Create a new provider with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create provider from configuration map
    pub fn from_config(config: HashMap<String, String>) -> Result<Self> {
        let default_config = ConnectionConfig::from_map(&config)?;
        Ok(Self {
            default_config,
            ..Default::default()
        })
    }

    /// Start WebSocket server if in server mode
    pub async fn start_server_if_needed(&mut self) -> Result<()> {
        if self.default_config.mode == ConnectionMode::Server {
            info!(
                "Starting WebSocket server mode on {}",
                self.default_config.uri
            );

            // Create a clone of self for the message handler
            let _session_storage = Arc::clone(&self.session_storage);
            let _handler_components = Arc::clone(&self.handler_components);

            // Create server state with message handler
            let server_state = ServerState::new(move |session_id, msg| {
                // This handler will be called when server receives messages from clients
                debug!(
                    "Server received message from session {}: subject={}",
                    session_id, msg.subject
                );
                // In a full implementation, this would invoke the handler component
                // For now, we just log it
                Ok(())
            });

            self.server_state = Some(Arc::new(server_state.clone()));

            // Start server
            let (addr, handle) = start_server(&self.default_config.uri, server_state).await?;

            let mut server_addr = self.server_addr.write().await;
            *server_addr = Some(addr);

            let mut server_handle = self.server_handle.write().await;
            *server_handle = Some(handle);

            info!("WebSocket server started on {}", addr);
        }
        Ok(())
    }

    /// Get server address if running in server mode
    pub async fn get_server_addr(&self) -> Option<SocketAddr> {
        let addr = self.server_addr.read().await;
        *addr
    }

    /// Send message to a specific WebSocket client (server mode)
    pub async fn send_to_ws_client(&self, session_id: &str, message: BrokerMessage) -> Result<()> {
        if let Some(ref server_state) = self.server_state {
            let msg = self.encode_message_to_axum(&message)?;
            server_state.send_to_client(session_id, msg).await?;
            Ok(())
        } else {
            bail!("Provider is not in server mode")
        }
    }

    /// Broadcast message to all WebSocket clients (server mode)
    pub async fn broadcast_to_clients(&self, message: BrokerMessage) -> Result<()> {
        if let Some(ref server_state) = self.server_state {
            let msg = self.encode_message_to_axum(&message)?;
            server_state.broadcast(msg).await?;
            Ok(())
        } else {
            bail!("Provider is not in server mode")
        }
    }

    /// List all connected WebSocket client sessions (server mode)
    pub async fn list_ws_clients(&self) -> Result<Vec<String>> {
        if let Some(ref server_state) = self.server_state {
            Ok(server_state.list_client_sessions().await)
        } else {
            bail!("Provider is not in server mode")
        }
    }

    /// Encode a broker message into an Axum WebSocket message (for server mode)
    fn encode_message_to_axum(&self, msg: &BrokerMessage) -> Result<AxumMessage> {
        let json = serde_json::json!({
            "subject": msg.subject,
            "body": base64::encode(&msg.body),
            "reply_to": msg.reply_to,
        });
        Ok(AxumMessage::Text(json.to_string()))
    }

    /// Set message handler for client mode - receives messages from remote WS server
    pub async fn set_client_message_handler<F>(&self, handler: F)
    where
        F: Fn(String, BrokerMessage) -> Result<()> + Send + Sync + 'static,
    {
        let mut h = self.client_message_handler.write().await;
        *h = Some(Arc::new(handler));
        info!("Client message handler registered");
    }

    /// Broadcast message to all handler components (called when message arrives from remote WS server)
    #[allow(dead_code)]
    async fn broadcast_to_handlers(&self, session_id: &str, msg: BrokerMessage) -> Result<()> {
        let handlers = self.handler_components.read().await;

        if handlers.is_empty() {
            debug!("No handler components registered to receive messages");
            return Ok(());
        }

        let encoded_msg = self.encode_message(&msg)?;
        let mut broadcast_count = 0;

        for (component_id, bundle) in handlers.iter() {
            if let Err(e) = bundle.tx.send(encoded_msg.clone()) {
                error!(
                    "Failed to broadcast message to component {}: {}",
                    component_id, e
                );
            } else {
                broadcast_count += 1;
                debug!("Broadcasted message to component {}", component_id);
            }
        }

        info!(
            "Broadcasted message from session {} to {} handler components",
            session_id, broadcast_count
        );
        Ok(())
    }

    /// Parse incoming message from remote WebSocket server (static version for async tasks)
    pub fn parse_message_static(text: &str, session_id: &str) -> Result<BrokerMessage> {
        // Try to parse as JSON first
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
            let subject = json
                .get("subject")
                .and_then(|v| v.as_str())
                .unwrap_or("default")
                .to_string();

            let body = if let Some(body_str) = json.get("body").and_then(|v| v.as_str()) {
                Bytes::from(body_str.as_bytes().to_vec())
            } else if let Some(body_arr) = json.get("body").and_then(|v| v.as_array()) {
                let bytes: Vec<u8> = body_arr
                    .iter()
                    .filter_map(|v| v.as_u64().map(|n| n as u8))
                    .collect();
                Bytes::from(bytes)
            } else {
                Bytes::from(text.as_bytes().to_vec())
            };

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
        } else {
            // Plain text message
            Ok(BrokerMessage {
                subject: "message".to_string(),
                body: Bytes::from(text.as_bytes().to_vec()),
                reply_to: Some(session_id.to_string()),
            })
        }
    }

    /// Encode message (static version for async tasks)
    pub fn encode_message_static(msg: &BrokerMessage) -> Result<Message> {
        let json = serde_json::json!({
            "subject": msg.subject,
            "body": base64::encode(&msg.body),
            "reply_to": msg.reply_to,
        });
        Ok(Message::Text(json.to_string()))
    }

    /// Connect to a WebSocket server
    #[instrument(skip(self, config))]
    async fn connect(
        &self,
        config: ConnectionConfig,
        component_id: &str,
    ) -> Result<WebSocketClientBundle> {
        let url = Url::parse(&config.uri)
            .with_context(|| format!("Invalid WebSocket URI: {}", config.uri))?;

        info!("Connecting to WebSocket at {}", url);

        // Create WebSocket connection with timeout
        let ws_stream = tokio::time::timeout(
            Duration::from_secs(config.connect_timeout_sec),
            connect_async(url.clone()),
        )
        .await
        .context("Connection timeout")?
        .context("Failed to connect to WebSocket")?
        .0;

        info!("WebSocket connected successfully");

        // Create channel for sending messages
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

        // Create session info
        let session_id = uuid::Uuid::new_v4().to_string();
        let session_info = SessionInfo {
            session_id: session_id.clone(),
            connected_at: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        };

        // Store session mapping if tracking is enabled
        if config.enable_session_tracking {
            let mut sessions = self.session_storage.write().await;
            sessions.insert(session_id.clone(), component_id.to_string());
            debug!(
                "Session {} registered for component {}",
                session_id, component_id
            );
        }

        // Split WebSocket stream
        let (mut ws_tx, mut ws_rx) = ws_stream.split();

        // Spawn task to handle bidirectional communication
        let component_id = component_id.to_string();
        let session_storage = Arc::clone(&self.session_storage);
        let handler_components = Arc::clone(&self.handler_components);
        let session_id_for_handler = session_id.clone();

        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle outgoing messages
                    Some(msg) = rx.recv() => {
                        if let Err(e) = ws_tx.send(msg).await {
                            error!("Failed to send WebSocket message: {}", e);
                            break;
                        }
                    }
                    // Handle incoming messages from remote WebSocket server
                    Some(msg_result) = ws_rx.next() => {
                        match msg_result {
                            Ok(Message::Text(text)) => {
                                debug!("Received text message from remote server: {}", text);

                                // Parse the message
                                if let Ok(broker_msg) = Self::parse_message_static(&text, &session_id_for_handler) {
                                    // Broadcast to all handler components
                                    let handlers = handler_components.read().await;
                                    for (comp_id, bundle) in handlers.iter() {
                                        let encoded = Self::encode_message_static(&broker_msg);
                                        if let Ok(msg) = encoded {
                                            if let Err(e) = bundle.tx.send(msg) {
                                                error!("Failed to forward message to component {}: {}", comp_id, e);
                                            } else {
                                                debug!("Forwarded message to component {}", comp_id);
                                            }
                                        }
                                    }
                                }
                            }
                            Ok(Message::Binary(data)) => {
                                debug!("Received binary message from remote server: {} bytes", data.len());

                                // Try to convert to text and parse
                                if let Ok(text) = String::from_utf8(data.clone()) {
                                    if let Ok(broker_msg) = Self::parse_message_static(&text, &session_id_for_handler) {
                                        let handlers = handler_components.read().await;
                                        for (comp_id, bundle) in handlers.iter() {
                                            let encoded = Self::encode_message_static(&broker_msg);
                                            if let Ok(msg) = encoded {
                                                if let Err(e) = bundle.tx.send(msg) {
                                                    error!("Failed to forward message to component {}: {}", comp_id, e);
                                                } else {
                                                    debug!("Forwarded message to component {}", comp_id);
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // Handle as raw binary
                                    let broker_msg = BrokerMessage {
                                        subject: "binary.message".to_string(),
                                        body: Bytes::from(data),
                                        reply_to: Some(session_id_for_handler.clone()),
                                    };

                                    let handlers = handler_components.read().await;
                                    for (comp_id, bundle) in handlers.iter() {
                                        let encoded = Self::encode_message_static(&broker_msg);
                                        if let Ok(msg) = encoded {
                                            if let Err(e) = bundle.tx.send(msg) {
                                                error!("Failed to forward message to component {}: {}", comp_id, e);
                                            } else {
                                                debug!("Forwarded binary message to component {}", comp_id);
                                            }
                                        }
                                    }
                                }
                            }
                            Ok(Message::Close(_)) => {
                                info!("WebSocket connection closed");
                                break;
                            }
                            Ok(Message::Ping(data)) => {
                                if let Err(e) = ws_tx.send(Message::Pong(data)).await {
                                    error!("Failed to send pong: {}", e);
                                    break;
                                }
                            }
                            Ok(_) => {}
                            Err(e) => {
                                error!("WebSocket error: {}", e);
                                break;
                            }
                        }
                    }
                    else => break,
                }
            }

            // Cleanup session on disconnect
            let mut sessions = session_storage.write().await;
            sessions.retain(|_, cid| cid != &component_id);
            info!(
                "WebSocket connection handler terminated for component {}",
                component_id
            );
        });

        Ok(WebSocketClientBundle {
            tx,
            session_info,
            handle,
        })
    }

    /// Get a session by session ID
    pub async fn get_session(&self, session_id: &str) -> Option<String> {
        let sessions = self.session_storage.read().await;
        sessions.get(session_id).cloned()
    }

    /// List all active sessions (both component sessions and WS client sessions)
    pub async fn list_sessions(&self) -> Vec<(String, String)> {
        let mut sessions = Vec::new();

        // Get component sessions (client mode)
        let component_sessions = self.session_storage.read().await;
        for (sid, cid) in component_sessions.iter() {
            sessions.push((sid.clone(), format!("component:{}", cid)));
        }
        drop(component_sessions);

        // Get WS client sessions (server mode)
        if let Some(ref server_state) = self.server_state {
            for session_id in server_state.list_client_sessions().await {
                sessions.push((session_id.clone(), format!("ws-client:{}", session_id)));
            }
        }

        sessions
    }

    /// Send a message through a specific session
    /// Works for both client mode (component sessions) and server mode (WS client sessions)
    pub async fn send_to_session(&self, session_id: &str, message: BrokerMessage) -> Result<()> {
        // First, try to find in component sessions (client mode)
        if let Some(component_id) = self.get_session(session_id).await {
            // Try to find the component in either consumer or handler maps
            let consumers = self.consumer_components.read().await;
            if let Some(bundle) = consumers.get(&component_id) {
                let msg = self.encode_message(&message)?;
                bundle.tx.send(msg).context("Failed to send message")?;
                return Ok(());
            }
            drop(consumers);

            let handlers = self.handler_components.read().await;
            if let Some(bundle) = handlers.get(&component_id) {
                let msg = self.encode_message(&message)?;
                bundle.tx.send(msg).context("Failed to send message")?;
                return Ok(());
            }
            drop(handlers);
        }

        // If not found in component sessions, try server mode (WS clients)
        if let Some(ref server_state) = self.server_state {
            let msg = self.encode_message_to_axum(&message)?;
            server_state
                .send_to_client(session_id, msg)
                .await
                .context("Failed to send to WebSocket client")?;
            return Ok(());
        }

        bail!("Session not found: {}", session_id)
    }

    /// Encode a broker message into a WebSocket message
    fn encode_message(&self, msg: &BrokerMessage) -> Result<Message> {
        // Simple JSON encoding for demonstration
        // In production, you might want to use a more efficient binary format
        let json = serde_json::json!({
            "subject": msg.subject,
            "body": base64::encode(&msg.body),
            "reply_to": msg.reply_to,
        });
        Ok(Message::Text(json.to_string()))
    }

    /// Publish a message for a specific component
    #[instrument(skip(self, msg))]
    pub async fn publish(&self, component_id: &str, msg: BrokerMessage) -> Result<()> {
        debug!(
            "Publishing message to component {}: subject={}",
            component_id, msg.subject
        );

        let consumers = self.consumer_components.read().await;
        let bundle = consumers
            .get(component_id)
            .ok_or_else(|| anyhow!("Component not linked: {}", component_id))?;

        let ws_msg = self.encode_message(&msg)?;
        bundle
            .tx
            .send(ws_msg)
            .context("Failed to send message to WebSocket")?;

        Ok(())
    }

    /// Perform a request-reply operation
    #[instrument(skip(self, body))]
    pub async fn request(
        &self,
        component_id: &str,
        subject: String,
        body: Bytes,
        timeout_ms: u32,
    ) -> Result<BrokerMessage> {
        debug!(
            "Request from component {}: subject={}",
            component_id, subject
        );

        let consumers = self.consumer_components.read().await;
        let bundle = consumers
            .get(component_id)
            .ok_or_else(|| anyhow!("Component not linked: {}", component_id))?;

        // Generate a reply subject
        let reply_to = format!("_INBOX.{}", uuid::Uuid::new_v4());

        let msg = BrokerMessage {
            subject,
            body: body.clone(),
            reply_to: Some(reply_to.clone()),
        };

        let ws_msg = self.encode_message(&msg)?;
        bundle
            .tx
            .send(ws_msg)
            .context("Failed to send request to WebSocket")?;

        // TODO: Implement proper request-reply pattern with response waiting
        // For now, return a timeout error as this needs more sophisticated handling
        tokio::time::sleep(Duration::from_millis(timeout_ms as u64)).await;

        Err(anyhow!("Request-reply not fully implemented yet"))
    }

    /// Handle a new link configuration (component linking to this provider)
    #[instrument(skip(self, config))]
    pub async fn receive_link_config_as_target(
        &self,
        source_id: &str,
        config: HashMap<String, String>,
    ) -> Result<()> {
        info!("Receiving link config for source component: {}", source_id);

        let config = if config.is_empty() {
            self.default_config.clone()
        } else {
            let new_config = ConnectionConfig::from_map(&config)?;
            self.default_config.merge(&new_config)
        };

        let bundle = self.connect(config, source_id).await?;

        let mut components = self.consumer_components.write().await;
        components.insert(source_id.to_string(), bundle);

        info!("Successfully linked component: {}", source_id);
        Ok(())
    }

    /// Handle link configuration when provider is the source
    #[instrument(skip(self, config))]
    pub async fn receive_link_config_as_source(
        &self,
        target_id: &str,
        config: HashMap<String, String>,
    ) -> Result<()> {
        info!("Receiving link config for target component: {}", target_id);

        let config = if config.is_empty() {
            self.default_config.clone()
        } else {
            let new_config = ConnectionConfig::from_map(&config)?;
            self.default_config.merge(&new_config)
        };

        let bundle = self.connect(config, target_id).await?;

        let mut components = self.handler_components.write().await;
        components.insert(target_id.to_string(), bundle);

        info!("Successfully linked component: {}", target_id);
        Ok(())
    }

    /// Handle link deletion (component unlinking from provider)
    #[instrument(skip(self))]
    pub async fn delete_link_as_target(&self, source_id: &str) -> Result<()> {
        info!("Deleting link for source component: {}", source_id);

        let mut components = self.consumer_components.write().await;
        if let Some(bundle) = components.remove(source_id) {
            // The bundle will be dropped here, aborting the task and closing the connection
            debug!(
                "Removed WebSocket connection for component {} (session: {})",
                source_id, bundle.session_info.session_id
            );
        }

        Ok(())
    }

    /// Handle link deletion when provider is the source
    #[instrument(skip(self))]
    pub async fn delete_link_as_source(&self, target_id: &str) -> Result<()> {
        info!("Deleting link for target component: {}", target_id);

        let mut components = self.handler_components.write().await;
        if let Some(bundle) = components.remove(target_id) {
            debug!(
                "Removed WebSocket connection for component {} (session: {})",
                target_id, bundle.session_info.session_id
            );
        }

        Ok(())
    }

    /// Shutdown the provider, closing all connections
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down WebSocket messaging provider");

        // Stop server if running
        let mut server_handle = self.server_handle.write().await;
        if let Some(handle) = server_handle.take() {
            handle.abort();
            info!("WebSocket server stopped");
        }

        let mut consumers = self.consumer_components.write().await;
        consumers.clear();

        let mut handlers = self.handler_components.write().await;
        handlers.clear();

        let mut sessions = self.session_storage.write().await;
        sessions.clear();

        info!("WebSocket messaging provider shutdown complete");
        Ok(())
    }
}

// uuid is used in the implementation via uuid::Uuid::new_v4()

// Base64 encoding for message payload
mod base64 {
    use bytes::Bytes;

    pub fn encode(data: &Bytes) -> String {
        data.iter()
            .flat_map(|&b| {
                let hex = format!("{:02x}", b);
                hex.chars().collect::<Vec<_>>()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = WebSocketMessagingProvider::new();
        assert!(provider.consumer_components.try_read().is_ok());
    }

    #[test]
    fn test_provider_from_config() {
        let mut config = HashMap::new();
        config.insert("URI".to_string(), "ws://localhost:9090".to_string());

        let provider = WebSocketMessagingProvider::from_config(config).unwrap();
        assert_eq!(provider.default_config.uri, "ws://localhost:9090");
    }

    #[tokio::test]
    async fn test_session_management() {
        let provider = WebSocketMessagingProvider::new();

        // Initially no sessions
        let sessions = provider.list_sessions().await;
        assert_eq!(sessions.len(), 0);

        // Session tracking is tested through the connect method
    }
}
