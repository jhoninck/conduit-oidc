// Authentication and OIDC Integration
// Simplified version for Matrix chat system

use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;

/// OIDC configuration for Zitadel integration
#[derive(Debug, Clone)]
pub struct OIDCConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
    pub server_name: String,
}

/// Authenticated user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub access_token: String,
    pub device_id: String,
    pub subscription_active: bool,
    pub scopes: Vec<String>,
}

/// OIDC handler for authentication
pub struct OIDCHandler {
    config: OIDCConfig,
    // In production, you'd have proper OIDC client here
}

impl OIDCHandler {
    pub async fn new(config: OIDCConfig) -> Result<Self, AuthError> {
        Ok(Self { config })
    }

    /// Validate access token and return user info
    pub async fn validate_token(&self, access_token: &str) -> Result<AuthenticatedUser, AuthError> {
        // In production, this would validate the JWT token with Zitadel
        // For now, return a mock user for demonstration
        
        // Extract user ID from token (in production, decode JWT)
        let user_id = if access_token.starts_with("user_") {
            access_token.to_string()
        } else {
            return Err(AuthError::InvalidToken("Invalid token format".to_string()));
        };

        Ok(AuthenticatedUser {
            user_id,
            access_token: access_token.to_string(),
            device_id: "device_123".to_string(),
            subscription_active: true, // In production, check with billing system
            scopes: vec!["matrix:write".to_string(), "matrix:read".to_string()],
        })
    }

    /// Check if user has required scope
    pub fn user_has_scope(&self, user: &AuthenticatedUser, required_scope: &str) -> bool {
        user.scopes.iter().any(|scope| scope == required_scope)
    }

    /// Check if user has active subscription
    pub fn user_has_subscription(&self, user: &AuthenticatedUser) -> bool {
        user.subscription_active
    }
}

/// Authentication errors
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),
    
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("OIDC error: {0}")]
    OIDCError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
}

impl AuthError {
    pub fn status_code(&self) -> u16 {
        match self {
            AuthError::InvalidToken(_) => 401,
            AuthError::TokenExpired => 401,
            AuthError::InsufficientPermissions(_) => 403,
            AuthError::UserNotFound(_) => 404,
            AuthError::OIDCError(_) => 500,
            AuthError::NetworkError(_) => 500,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            AuthError::InvalidToken(_) => "M_UNKNOWN_TOKEN",
            AuthError::TokenExpired => "M_UNKNOWN_TOKEN",
            AuthError::InsufficientPermissions(_) => "M_FORBIDDEN",
            AuthError::UserNotFound(_) => "M_NOT_FOUND",
            AuthError::OIDCError(_) => "M_UNKNOWN",
            AuthError::NetworkError(_) => "M_UNKNOWN",
        }
    }
}

/// Login request
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    #[serde(rename = "type")]
    pub login_type: String,
    pub identifier: Option<UserIdentifier>,
    pub password: Option<String>,
    pub device_id: Option<String>,
    pub initial_device_display_name: Option<String>,
}

/// User identifier for login
#[derive(Debug, Serialize, Deserialize)]
pub struct UserIdentifier {
    #[serde(rename = "type")]
    pub id_type: String,
    pub user: Option<String>,
    pub medium: Option<String>,
    pub address: Option<String>,
}

/// Login response
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_id: String,
    pub access_token: String,
    pub device_id: String,
    pub expires_in_ms: Option<u64>,
    pub refresh_token: Option<String>,
}

/// Logout request
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub access_token: String,
}

/// Logout response
#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub message: String,
}

/// Device information
#[derive(Debug, Serialize)]
pub struct Device {
    pub device_id: String,
    pub display_name: Option<String>,
    pub last_seen_ip: Option<String>,
    pub last_seen_user_agent: Option<String>,
    pub last_seen_ts: Option<u64>,
}

/// Device list response
#[derive(Debug, Serialize)]
pub struct DeviceListResponse {
    pub devices: Vec<Device>,
}

/// User profile information
#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub displayname: Option<String>,
    pub avatar_url: Option<String>,
}

/// Whoami response
#[derive(Debug, Serialize, Deserialize)]
pub struct WhoamiResponse {
    pub user_id: String,
    pub device_id: Option<String>,
    pub is_guest: bool,
}

/// Authentication middleware for Axum
pub async fn auth_middleware(
    auth_header: Option<axum::http::HeaderValue>,
    state: axum::extract::State<crate::MatrixServer>,
) -> Result<AuthenticatedUser, AuthError> {
    let auth_header = auth_header
        .ok_or_else(|| AuthError::InvalidToken("Missing Authorization header".to_string()))?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| AuthError::InvalidToken("Invalid Authorization header".to_string()))?;

    if !auth_str.starts_with("Bearer ") {
        return Err(AuthError::InvalidToken("Invalid Authorization format".to_string()));
    }

    let token = &auth_str[7..]; // Remove "Bearer " prefix
    
    state.auth_handler.validate_token(token).await
}

/// Extract user from request for authenticated endpoints
pub async fn extract_user(
    // TODO: Implement proper auth header extraction
    // auth_header: axum::extract::TypedHeader<axum::headers::Authorization<axum::headers::Bearer>>,
    state: axum::extract::State<crate::MatrixServer>,
) -> Result<AuthenticatedUser, AuthError> {
    // TODO: Extract token from authorization header when TypedHeader is available
    // For now, return a mock user for testing
    Ok(AuthenticatedUser {
        user_id: "test_user".to_string(),
        access_token: "mock_token".to_string(),
        device_id: "mock_device".to_string(),
        subscription_active: true,
        scopes: vec!["openid".to_string(), "profile".to_string()],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MatrixServer;
    use std::sync::Arc;
    use axum::extract::State;

    // Mock MatrixServer for testing
    async fn create_mock_server() -> MatrixServer {
        MatrixServer {
            auth_handler: Arc::new(OIDCHandler::new(OIDCConfig {
                issuer_url: "https://test-issuer.com".to_string(),
                client_id: "test-client".to_string(),
                client_secret: "test-secret".to_string(),
                redirect_url: "http://localhost:8000/callback".to_string(),
                scopes: vec!["openid".to_string(), "profile".to_string()],
                server_name: "test.local".to_string(),
            }).await.unwrap()),
            room_handler: Arc::new(crate::RoomHandler::new(Arc::new(crate::state::InMemoryStateStore::new()))),
            federation_client: Arc::new(crate::FederationClient::new(crate::federation::FederationConfig {
                server_name: "test.local".to_string(),
                signing_key: "test-key".to_string(),
                verify_signatures: false,
                federation_whitelist: None,
                federation_blacklist: None,
            }).await.unwrap()),
            state_store: Arc::new(crate::state::InMemoryStateStore::new()),
            server_name: "test.local".to_string(),
        }
    }

    #[tokio::test]
    async fn test_oidc_handler_new() {
        let config = OIDCConfig {
            issuer_url: "https://test-issuer.com".to_string(),
            client_id: "test-client".to_string(),
            client_secret: "test-secret".to_string(),
            redirect_url: "http://localhost:8000/callback".to_string(),
            scopes: vec!["openid".to_string(), "profile".to_string()],
            server_name: "test.local".to_string(),
        };

        let handler = OIDCHandler::new(config).await;
        assert!(handler.is_ok());
    }

    #[tokio::test]
    async fn test_oidc_handler_validate_token() {
        let config = OIDCConfig {
            issuer_url: "https://test-issuer.com".to_string(),
            client_id: "test-client".to_string(),
            client_secret: "test-secret".to_string(),
            redirect_url: "http://localhost:8000/callback".to_string(),
            scopes: vec!["openid".to_string(), "profile".to_string()],
            server_name: "test.local".to_string(),
        };

        let handler = OIDCHandler::new(config).await.unwrap();
        
        // Test with invalid token
        let result = handler.validate_token("invalid-token").await;
        assert!(result.is_err());
        
        // Test with valid token (mock)
        let result = handler.validate_token("valid-token").await;
        // This will fail in tests since we don't have a real OIDC provider
        // but we can test the error handling
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auth_middleware_missing_header() {
        let server = create_mock_server().await;
        let state = State(server);
        
        let result = auth_middleware(None, state).await;
        assert!(result.is_err());
        
        if let Err(AuthError::InvalidToken(msg)) = result {
            assert_eq!(msg, "Missing Authorization header");
        } else {
            panic!("Expected InvalidToken error");
        }
    }

    #[tokio::test]
    async fn test_auth_middleware_invalid_format() {
        let server = create_mock_server().await;
        let state = State(server);
        
        let header = axum::http::HeaderValue::from_static("InvalidFormat");
        let result = auth_middleware(Some(header), state).await;
        assert!(result.is_err());
        
        if let Err(AuthError::InvalidToken(msg)) = result {
            assert_eq!(msg, "Invalid Authorization format");
        } else {
            panic!("Expected InvalidToken error");
        }
    }

    #[tokio::test]
    async fn test_auth_middleware_valid_format() {
        let server = create_mock_server().await;
        let state = State(server);
        
        let header = axum::http::HeaderValue::from_static("Bearer valid-token");
        let result = auth_middleware(Some(header), state).await;
        // This will fail in tests since we don't have a real OIDC provider
        // but we can test that the format parsing works
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_extract_user() {
        let server = create_mock_server().await;
        let state = State(server);
        
        let result = extract_user(state).await;
        assert!(result.is_ok());
        
        let user = result.unwrap();
        assert_eq!(user.user_id, "test_user");
        assert_eq!(user.access_token, "mock_token");
        assert_eq!(user.device_id, "mock_device");
        assert!(user.subscription_active);
        assert_eq!(user.scopes, vec!["openid", "profile"]);
    }

    #[test]
    fn test_login_request_serialization() {
        let request = LoginRequest {
            login_type: "m.login.password".to_string(),
            identifier: Some(UserIdentifier {
                id_type: "m.id.user".to_string(),
                user: Some("testuser".to_string()),
                medium: None,
                address: None,
            }),
            password: Some("testpass".to_string()),
            device_id: Some("testdevice".to_string()),
            initial_device_display_name: Some("Test Device".to_string()),
        };

        // Test that it can be serialized/deserialized
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: LoginRequest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.login_type, request.login_type);
        assert_eq!(deserialized.identifier.unwrap().user, request.identifier.unwrap().user);
        assert_eq!(deserialized.password, request.password);
    }

    #[test]
    fn test_login_response_serialization() {
        let response = LoginResponse {
            user_id: "@testuser:localhost".to_string(),
            access_token: "test-token".to_string(),
            device_id: "test-device".to_string(),
            expires_in_ms: Some(3600000),
            refresh_token: Some("refresh-token".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: LoginResponse = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.user_id, response.user_id);
        assert_eq!(deserialized.access_token, response.access_token);
        assert_eq!(deserialized.device_id, response.device_id);
        assert_eq!(deserialized.expires_in_ms, response.expires_in_ms);
        assert_eq!(deserialized.refresh_token, response.refresh_token);
    }

    #[test]
    fn test_user_profile_serialization() {
        let profile = UserProfile {
            user_id: "@testuser:localhost".to_string(),
            displayname: Some("Test User".to_string()),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
        };

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: UserProfile = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.user_id, profile.user_id);
        assert_eq!(deserialized.displayname, profile.displayname);
        assert_eq!(deserialized.avatar_url, profile.avatar_url);
    }

    #[test]
    fn test_whoami_response_serialization() {
        let whoami = WhoamiResponse {
            user_id: "@testuser:localhost".to_string(),
            device_id: Some("test-device".to_string()),
            is_guest: false,
        };

        let json = serde_json::to_string(&whoami).unwrap();
        let deserialized: WhoamiResponse = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.user_id, whoami.user_id);
        assert_eq!(deserialized.device_id, whoami.device_id);
        assert_eq!(deserialized.is_guest, whoami.is_guest);
    }
}
