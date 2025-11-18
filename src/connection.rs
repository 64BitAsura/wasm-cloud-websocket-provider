use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Configuration for WebSocket connections
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConnectionConfig {
    /// WebSocket server URI (e.g., "ws://localhost:8080" or "wss://example.com/ws")
    #[serde(default = "default_uri")]
    pub uri: String,

    /// Optional authentication token to send with connection
    #[serde(default)]
    pub auth_token: Option<String>,

    /// Timeout for connection in seconds
    #[serde(default = "default_timeout")]
    pub connect_timeout_sec: u64,

    /// Enable session tracking
    #[serde(default = "default_session_tracking")]
    pub enable_session_tracking: bool,

    /// Custom headers to send with WebSocket upgrade request
    #[serde(default)]
    pub custom_headers: HashMap<String, String>,
}

fn default_uri() -> String {
    "ws://127.0.0.1:8080".to_string()
}

fn default_timeout() -> u64 {
    30
}

fn default_session_tracking() -> bool {
    true
}

impl ConnectionConfig {
    /// Create a ConnectionConfig from a HashMap of configuration values
    pub fn from_map(config: &HashMap<String, String>) -> Result<Self> {
        let uri = config.get("URI").cloned().unwrap_or_else(default_uri);

        let auth_token = config.get("AUTH_TOKEN").cloned();

        let connect_timeout_sec = config
            .get("CONNECT_TIMEOUT_SEC")
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(default_timeout);

        let enable_session_tracking = config
            .get("ENABLE_SESSION_TRACKING")
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(default_session_tracking);

        let mut custom_headers = HashMap::new();
        for (key, value) in config.iter() {
            if key.starts_with("HEADER_") {
                let header_name = key.strip_prefix("HEADER_").unwrap();
                custom_headers.insert(header_name.to_string(), value.clone());
            }
        }

        Ok(ConnectionConfig {
            uri,
            auth_token,
            connect_timeout_sec,
            enable_session_tracking,
            custom_headers,
        })
    }

    /// Merge two configs, with values from `other` taking precedence
    pub fn merge(&self, other: &Self) -> Self {
        let mut custom_headers = self.custom_headers.clone();
        custom_headers.extend(other.custom_headers.clone());

        ConnectionConfig {
            uri: if other.uri != default_uri() {
                other.uri.clone()
            } else {
                self.uri.clone()
            },
            auth_token: other.auth_token.clone().or_else(|| self.auth_token.clone()),
            connect_timeout_sec: if other.connect_timeout_sec != default_timeout() {
                other.connect_timeout_sec
            } else {
                self.connect_timeout_sec
            },
            enable_session_tracking: other.enable_session_tracking,
            custom_headers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_map_default() {
        let config = ConnectionConfig::from_map(&HashMap::new()).unwrap();
        assert_eq!(config.uri, "ws://127.0.0.1:8080");
        assert_eq!(config.connect_timeout_sec, 30);
        assert!(config.enable_session_tracking);
    }

    #[test]
    fn test_from_map_custom() {
        let mut map = HashMap::new();
        map.insert("URI".to_string(), "wss://example.com/ws".to_string());
        map.insert("AUTH_TOKEN".to_string(), "secret123".to_string());
        map.insert("CONNECT_TIMEOUT_SEC".to_string(), "60".to_string());
        map.insert(
            "HEADER_Authorization".to_string(),
            "Bearer token".to_string(),
        );

        let config = ConnectionConfig::from_map(&map).unwrap();
        assert_eq!(config.uri, "wss://example.com/ws");
        assert_eq!(config.auth_token, Some("secret123".to_string()));
        assert_eq!(config.connect_timeout_sec, 60);
        assert_eq!(
            config.custom_headers.get("Authorization"),
            Some(&"Bearer token".to_string())
        );
    }

    #[test]
    fn test_merge() {
        let config1 = ConnectionConfig {
            uri: "ws://localhost:8080".to_string(),
            auth_token: Some("token1".to_string()),
            connect_timeout_sec: 30,
            enable_session_tracking: true,
            custom_headers: HashMap::from([("X-Custom".to_string(), "value1".to_string())]),
        };

        let config2 = ConnectionConfig {
            uri: "wss://example.com".to_string(),
            auth_token: None,
            connect_timeout_sec: 60,
            enable_session_tracking: false,
            custom_headers: HashMap::from([("X-Other".to_string(), "value2".to_string())]),
        };

        let merged = config1.merge(&config2);
        assert_eq!(merged.uri, "wss://example.com");
        assert_eq!(merged.auth_token, Some("token1".to_string()));
        assert_eq!(merged.connect_timeout_sec, 60);
        assert!(!merged.enable_session_tracking);
        assert_eq!(merged.custom_headers.len(), 2);
    }
}
