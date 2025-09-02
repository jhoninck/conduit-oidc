// Matrix Chat System - Rust Matrix Server
// Simplified working version that demonstrates compilation

pub mod auth;
pub mod room;
pub mod federation;
pub mod client_server;
pub mod events;
pub mod state;
pub mod error;
pub mod conduit;

// Re-exports for clean API
pub use auth::{OIDCHandler, AuthenticatedUser, AuthError};
pub use room::{RoomHandler, RoomConfig, RoomError};
pub use federation::{FederationClient, FederationError};
pub use client_server::{ClientServerAPI, ClientError};
pub use events::{MatrixEvent, EventType, EventContent};
pub use state::{RoomState, StateStore, StateError};
pub use error::{MatrixServerError, Result};
pub use conduit::{ConduitServer, ConduitConfig, ConduitError};

use std::sync::Arc;
use axum::{routing::*, Router};

/// Main Matrix server instance
/// Coordinates all components like Synapse's main application
#[derive(Clone)]
pub struct MatrixServer {
    pub auth_handler: Arc<OIDCHandler>,
    pub room_handler: Arc<RoomHandler>,
    pub federation_client: Arc<FederationClient>,
    pub state_store: Arc<dyn StateStore + Send + Sync>,
    pub server_name: String,
}

impl MatrixServer {
    pub async fn new(config: ServerConfig) -> Result<Self> {
        // Initialize core components
        let state_store = Arc::new(
            crate::state::InMemoryStateStore::new()
        );
        
        let auth_handler = Arc::new(
            OIDCHandler::new(config.oidc_config).await?
        );
        
        let room_handler = Arc::new(
            RoomHandler::new(state_store.clone())
        );
        
        let federation_client = Arc::new(
            FederationClient::new(config.federation_config).await?
        );

        Ok(MatrixServer {
            auth_handler,
            room_handler,
            federation_client,
            state_store,
            server_name: config.server_name,
        })
    }

    pub async fn start(&self, bind_addr: &str) -> Result<()> {
        // Start the HTTP server with all endpoints
        let app = self.create_router().await?;
        
        tracing::info!("Starting Matrix server on {}", bind_addr);
        
        let listener = tokio::net::TcpListener::bind(bind_addr).await
            .map_err(|e| MatrixServerError::NetworkError(e.to_string()))?;
            
        axum::serve(listener, app).await
            .map_err(|e| MatrixServerError::NetworkError(e.to_string()))?;
            
        Ok(())
    }

    async fn create_router(&self) -> Result<axum::Router> {
        Ok(Router::new()
            // Client-Server API (/_matrix/client/*)
            .nest("/_matrix/client", self.client_server_routes())
            // Server-Server API (/_matrix/federation/*)
            .nest("/_matrix/federation", self.federation_routes())
            // Health check
            .route("/health", get(|| async { "OK" }))
            .with_state(self.clone()))
    }

    fn client_server_routes(&self) -> Router<MatrixServer> {
        Router::new()
            .route("/v3/login", post(client_server::login))
            .route("/v3/logout", post(client_server::logout))
            .route("/v3/rooms/:room_id/send/:event_type/:txn_id", put(client_server::send_message))
            .route("/v3/rooms/:room_id/messages", get(client_server::get_messages))
            .route("/v3/rooms/:room_id/join", post(client_server::join_room))
            .route("/v3/rooms/:room_id/leave", post(client_server::leave_room))
            .route("/v3/sync", get(client_server::sync))
            .route("/v3/account/whoami", get(client_server::whoami))
            .route("/v3/rooms", get(client_server::list_rooms))
            .route("/v3/support/request", post(client_server::create_support_request))
    }

    fn federation_routes(&self) -> Router<MatrixServer> {
        Router::new()
            .route("/v1/version", get(federation::get_version))
            .route("/v1/query/directory", get(federation::query_directory))
            .route("/v1/event/:event_id", get(federation::get_event))
            .route("/v1/state/:room_id", get(federation::get_room_state))
            .route("/v1/state_ids/:room_id", get(federation::get_room_state_ids))
            .route("/v1/backfill/:room_id", get(federation::backfill_room))
            .route("/v1/get_missing_events/:room_id", post(federation::get_missing_events))
            .route("/v1/event_auth/:room_id/:event_id", get(federation::get_event_auth))
            .route("/v1/query/profile", get(federation::query_profile))
            .route("/v1/make_join/:room_id/:user_id", post(federation::make_join))
            .route("/v1/send_join/:room_id/:event_id", put(federation::send_join))
            .route("/v1/invite/:room_id/:event_id", put(federation::invite))
            .route("/v1/event/:room_id/:event_id", put(federation::send_event))
            .route("/v1/query/directory", get(federation::query_directory))
            .route("/v1/query/profile", get(federation::query_profile))
            .route("/v1/query/keys", post(federation::query_keys))
            .route("/v1/query/client_keys", post(federation::query_client_keys))
            .route("/v1/user/keys/query", post(federation::query_user_keys))
            .route("/v1/user/devices/:user_id", get(federation::get_user_devices))
            .route("/v1/claim/e2e_one_time_key", post(federation::claim_one_time_key))
    }
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub server_name: String,
    pub oidc_config: auth::OIDCConfig,
    pub federation_config: federation::FederationConfig,
}

/// Well-known endpoints for Matrix discovery
pub async fn well_known_server() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "m.server": "matrix.local:8008"
    }))
}

pub async fn well_known_client() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "m.homeserver": {
            "base_url": "https://matrix.local:8008"
        },
        "m.identity_server": {
            "base_url": "https://vector.im"
        }
    }))
}
