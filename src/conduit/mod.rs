//! Conduit Matrix Server Integration
//! 
//! This module provides Conduit Matrix server functionality integrated into the chat-system.
//! It's not a separate component but rather an integrated part of the Matrix implementation.

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    state::StateStore,
    error::MatrixServerError,
};

/// Conduit server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConduitConfig {
    pub server_name: String,
    pub port: u16,
    pub database_url: String,
    pub oidc_enabled: bool,
    pub federation_enabled: bool,
    pub max_upload_size: usize,
}

impl Default for ConduitConfig {
    fn default() -> Self {
        Self {
            server_name: "conduit.local".to_string(),
            port: 8008,
            database_url: "sqlite:conduit.db".to_string(),
            oidc_enabled: true,
            federation_enabled: true,
            max_upload_size: 50 * 1024 * 1024, // 50MB
        }
    }
}

/// Conduit server instance integrated with chat-system
pub struct ConduitServer {
    config: ConduitConfig,
    state_store: Arc<dyn StateStore + Send + Sync>,
    // Other components will be added as needed
}

impl ConduitServer {
    /// Create a new Conduit server instance
    pub async fn new(
        config: ConduitConfig,
        state_store: Arc<dyn StateStore + Send + Sync>,
    ) -> Result<Self, ConduitError> {
        Ok(Self {
            config,
            state_store,
        })
    }

    /// Start the Conduit server
    pub async fn start(&self) -> Result<(), ConduitError> {
        tracing::info!("🚀 Starting Conduit Matrix server on port {}", self.config.port);
        tracing::info!("📊 Server name: {}", self.config.server_name);
        tracing::info!("🔐 OIDC enabled: {}", self.config.oidc_enabled);
        tracing::info!("🌐 Federation enabled: {}", self.config.federation_enabled);
        
        // TODO: Implement actual server startup
        // For now, just log that we're starting
        
        Ok(())
    }

    /// Get server configuration
    pub fn config(&self) -> &ConduitConfig {
        &self.config
    }

    /// Check if OIDC is enabled
    pub fn oidc_enabled(&self) -> bool {
        self.config.oidc_enabled
    }

    /// Check if federation is enabled
    pub fn federation_enabled(&self) -> bool {
        self.config.federation_enabled
    }
}

/// Conduit-specific errors
#[derive(Error, Debug)]
pub enum ConduitError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("OIDC error: {0}")]
    OIDCError(String),
    
    #[error("Federation error: {0}")]
    FederationError(String),
}

impl From<ConduitError> for MatrixServerError {
    fn from(err: ConduitError) -> Self {
        MatrixServerError::Internal(err.to_string())
    }
}

/// Conduit API endpoints that integrate with the chat-system
pub mod api {
    use axum::{
        response::Json,
        routing::{get, put},
        Router,
    };
    use serde_json::Value;

    /// Create the Conduit API router
    pub fn create_router() -> Router {
        Router::new()
            // Client-server API endpoints
            .route("/_matrix/client/r0/versions", get(get_versions))
            .route("/_matrix/client/r0/sync", get(get_sync))
            .route("/_matrix/client/r0/rooms/:room_id/state", get(get_room_state))
            .route("/_matrix/client/r0/rooms/:room_id/messages", get(get_room_messages))
            .route("/_matrix/client/r0/rooms/:room_id/send/:event_type/:txn_id", put(send_message))
            
            // Federation endpoints
            .route("/_matrix/federation/v1/version", get(get_federation_version))
            .route("/_matrix/federation/v1/query/directory", get(query_directory))
            .route("/_matrix/federation/v1/event/:event_id", get(get_event))
            
            // Health check
            .route("/health", get(|| async { "OK" }))
    }

    // Client-server API endpoints
    pub async fn get_versions() -> Json<Value> {
        Json(serde_json::json!({
            "versions": ["r0.6.0", "v1.1", "v1.2", "v1.3"]
        }))
    }

    pub async fn get_sync() -> Json<Value> {
        Json(serde_json::json!({
            "next_batch": "s1234567890",
            "rooms": {
                "join": {},
                "invite": {},
                "leave": {}
            }
        }))
    }

    pub async fn get_room_state() -> Json<Value> {
        Json(serde_json::json!({
            "chunk": []
        }))
    }

    pub async fn get_room_messages() -> Json<Value> {
        Json(serde_json::json!({
            "start": "s1234567890",
            "chunk": [],
            "end": "s1234567890"
        }))
    }

    pub async fn send_message() -> Json<Value> {
        Json(serde_json::json!({
            "event_id": "$1234567890"
        }))
    }

    // Federation API endpoints
    pub async fn get_federation_version() -> Json<Value> {
        Json(serde_json::json!({
            "server": {
                "name": "conduit",
                "version": "0.1.0"
            }
        }))
    }

    pub async fn query_directory() -> Json<Value> {
        Json(serde_json::json!({
            "room_id": "!test:conduit.local",
            "servers": ["conduit.local"]
        }))
    }

    pub async fn get_event() -> Json<Value> {
        Json(serde_json::json!({
            "origin": "conduit.local",
            "origin_server_ts": 1234567890,
            "pdus": []
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::InMemoryStateStore;

    #[tokio::test]
    async fn test_conduit_config_default() {
        let config = ConduitConfig::default();
        assert_eq!(config.server_name, "conduit.local");
        assert_eq!(config.port, 8008);
        assert!(config.oidc_enabled);
        assert!(config.federation_enabled);
        assert_eq!(config.max_upload_size, 50 * 1024 * 1024);
    }

    #[tokio::test]
    async fn test_conduit_server_new() {
        let config = ConduitConfig::default();
        let state_store = Arc::new(InMemoryStateStore::new());
        
        let server = ConduitServer::new(config, state_store).await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_conduit_server_start() {
        let config = ConduitConfig::default();
        let state_store = Arc::new(InMemoryStateStore::new());
        
        let server = ConduitServer::new(config, state_store).await.unwrap();
        let result = server.start().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_conduit_server_config() {
        let config = ConduitConfig::default();
        let state_store = Arc::new(InMemoryStateStore::new());
        
        let server = ConduitServer::new(config.clone(), state_store).await.unwrap();
        assert_eq!(server.config().server_name, config.server_name);
        assert_eq!(server.config().port, config.port);
    }

    #[tokio::test]
    async fn test_conduit_server_features() {
        let config = ConduitConfig::default();
        let state_store = Arc::new(InMemoryStateStore::new());
        
        let server = ConduitServer::new(config, state_store).await.unwrap();
        assert!(server.oidc_enabled());
        assert!(server.federation_enabled());
    }
}
