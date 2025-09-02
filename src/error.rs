// Error types for the Matrix server
// Translated from Synapse's error handling patterns

use thiserror::Error;

pub type Result<T> = std::result::Result<T, MatrixServerError>;

#[derive(Error, Debug)]
pub enum MatrixServerError {
    #[error("Authentication error: {0}")]
    Auth(#[from] crate::auth::AuthError),
    
    #[error("Room error: {0}")]
    Room(#[from] crate::room::RoomError),
    
    #[error("Federation error: {0}")]
    Federation(#[from] crate::federation::FederationError),
    
    #[error("State error: {0}")]
    State(#[from] crate::state::StateError),
    
    #[error("Client error: {0}")]
    Client(#[from] crate::client_server::ClientError),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Internal server error: {0}")]
    Internal(String),
}

// HTTP status code mapping for Matrix errors
impl MatrixServerError {
    pub fn status_code(&self) -> u16 {
        match self {
            MatrixServerError::Auth(_) => 401, // Unauthorized
            MatrixServerError::Room(room_err) => room_err.status_code(),
            MatrixServerError::Federation(_) => 500, // Internal server error for federation issues
            MatrixServerError::Client(client_err) => client_err.status_code(),
            MatrixServerError::NetworkError(_) => 503, // Service unavailable
            MatrixServerError::ConfigError(_) => 500, // Internal server error
            MatrixServerError::DatabaseError(_) => 500, // Internal server error
            MatrixServerError::SerializationError(_) => 400, // Bad request
            MatrixServerError::HttpError(_) => 502, // Bad gateway
            MatrixServerError::Internal(_) => 500, // Internal server error
            MatrixServerError::State(_) => 500, // Internal server error
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            MatrixServerError::Auth(_) => "M_UNAUTHORIZED",
            MatrixServerError::Room(room_err) => room_err.error_code(),
            MatrixServerError::Federation(_) => "M_FEDERATION_ERROR",
            MatrixServerError::Client(client_err) => client_err.error_code(),
            MatrixServerError::NetworkError(_) => "M_UNKNOWN",
            MatrixServerError::ConfigError(_) => "M_UNKNOWN",
            MatrixServerError::DatabaseError(_) => "M_UNKNOWN",
            MatrixServerError::SerializationError(_) => "M_BAD_JSON",
            MatrixServerError::HttpError(_) => "M_UNKNOWN",
            MatrixServerError::Internal(_) => "M_UNKNOWN",
            MatrixServerError::State(_) => "M_UNKNOWN",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthError;
    use crate::room::RoomError;
    use crate::federation::FederationError;
    use crate::state::StateError;
    use crate::client_server::ClientError;

    #[test]
    fn test_matrix_server_error_status_codes() {
        assert_eq!(MatrixServerError::Auth(AuthError::InvalidToken("token".to_string())).status_code(), 401);
        assert_eq!(MatrixServerError::Room(RoomError::RoomNotFound("room".to_string())).status_code(), 404);
        assert_eq!(MatrixServerError::Federation(FederationError::RoomNotFound("room".to_string())).status_code(), 500);
        assert_eq!(MatrixServerError::Client(ClientError::InvalidCredentials).status_code(), 401);
        assert_eq!(MatrixServerError::NetworkError("error".to_string()).status_code(), 503);
        assert_eq!(MatrixServerError::ConfigError("error".to_string()).status_code(), 500);
        assert_eq!(MatrixServerError::DatabaseError("error".to_string()).status_code(), 500);
        assert_eq!(MatrixServerError::Internal("error".to_string()).status_code(), 500);
    }

    #[test]
    fn test_matrix_server_error_codes() {
        assert_eq!(MatrixServerError::Auth(AuthError::InvalidToken("token".to_string())).error_code(), "M_UNAUTHORIZED");
        assert_eq!(MatrixServerError::Room(RoomError::RoomNotFound("room".to_string())).error_code(), "M_NOT_FOUND");
        assert_eq!(MatrixServerError::Federation(FederationError::RoomNotFound("room".to_string())).error_code(), "M_FEDERATION_ERROR");
        assert_eq!(MatrixServerError::Client(ClientError::InvalidCredentials).error_code(), "M_FORBIDDEN");
        assert_eq!(MatrixServerError::NetworkError("error".to_string()).error_code(), "M_UNKNOWN");
        assert_eq!(MatrixServerError::ConfigError("error".to_string()).error_code(), "M_UNKNOWN");
        assert_eq!(MatrixServerError::DatabaseError("error".to_string()).error_code(), "M_UNKNOWN");
        assert_eq!(MatrixServerError::Internal("error".to_string()).error_code(), "M_UNKNOWN");
    }

    #[test]
    fn test_error_conversion_from_auth_error() {
        let auth_error = AuthError::InvalidToken("token".to_string());
        let matrix_error: MatrixServerError = auth_error.into();
        
        match matrix_error {
            MatrixServerError::Auth(_) => assert!(true),
            _ => panic!("Expected Auth variant"),
        }
    }

    #[test]
    fn test_error_conversion_from_room_error() {
        let room_error = RoomError::RoomNotFound("test_room".to_string());
        let matrix_error: MatrixServerError = room_error.into();
        
        match matrix_error {
            MatrixServerError::Room(_) => assert!(true),
            _ => panic!("Expected Room variant"),
        }
    }

    #[test]
    fn test_error_conversion_from_federation_error() {
        let federation_error = FederationError::RoomNotFound("test_room".to_string());
        let matrix_error: MatrixServerError = federation_error.into();
        
        match matrix_error {
            MatrixServerError::Federation(_) => assert!(true),
            _ => panic!("Expected Federation variant"),
        }
    }

    #[test]
    fn test_error_conversion_from_client_error() {
        let client_error = ClientError::InvalidCredentials;
        let matrix_error: MatrixServerError = client_error.into();
        
        match matrix_error {
            MatrixServerError::Client(_) => assert!(true),
            _ => panic!("Expected Client variant"),
        }
    }

    #[test]
    fn test_error_conversion_from_state_error() {
        let state_error = StateError::RoomNotFound("test_room".to_string());
        let matrix_error: MatrixServerError = state_error.into();
        
        match matrix_error {
            MatrixServerError::State(_) => assert!(true),
            _ => panic!("Expected State variant"),
        }
    }

    #[test]
    fn test_error_display() {
        let error = MatrixServerError::NetworkError("connection failed".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Network error"));
        assert!(display.contains("connection failed"));
    }

    #[test]
    fn test_error_debug() {
        let error = MatrixServerError::ConfigError("invalid config".to_string());
        let debug = format!("{:?}", error);
        assert!(debug.contains("ConfigError"));
        assert!(debug.contains("invalid config"));
    }

    #[test]
    fn test_serialization_error_conversion() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let matrix_error: MatrixServerError = json_error.into();
        
        match matrix_error {
            MatrixServerError::SerializationError(_) => assert!(true),
            _ => panic!("Expected SerializationError variant"),
        }
    }

    #[test]
    fn test_result_type_alias() {
        let success_result: Result<String> = Ok("success".to_string());
        let error_result: Result<String> = Err(MatrixServerError::Internal("error".to_string()));
        
        assert!(success_result.is_ok());
        assert!(error_result.is_err());
        
        match success_result {
            Ok(value) => assert_eq!(value, "success"),
            Err(_) => panic!("Expected Ok variant"),
        }
        
        match error_result {
            Ok(_) => panic!("Expected Err variant"),
            Err(MatrixServerError::Internal(msg)) => assert_eq!(msg, "error"),
            Err(_) => panic!("Expected Internal error variant"),
        }
    }

    #[test]
    fn test_error_with_different_auth_errors() {
        let auth_errors = vec![
            AuthError::InvalidToken("token".to_string()),
            AuthError::TokenExpired,
            AuthError::InsufficientPermissions("user".to_string()),
            AuthError::UserNotFound("user".to_string()),
        ];
        
        for auth_error in auth_errors {
            let matrix_error: MatrixServerError = auth_error.into();
            assert_eq!(matrix_error.status_code(), 401);
            assert_eq!(matrix_error.error_code(), "M_UNAUTHORIZED");
        }
    }

    #[test]
    fn test_error_with_different_room_errors() {
        let room_errors = vec![
            RoomError::RoomNotFound("room".to_string()),
            RoomError::UserNotInRoom("user".to_string()),
            RoomError::InsufficientPermissions("user".to_string()),
        ];
        
        for room_error in room_errors {
            let matrix_error: MatrixServerError = room_error.into();
            // Room errors have different status codes, so we just check it's a valid error
            assert!(matrix_error.status_code() >= 400 && matrix_error.status_code() < 600);
        }
    }

    #[test]
    fn test_error_message_formatting() {
        let error = MatrixServerError::Internal("test internal error".to_string());
        let message = format!("{}", error);
        assert!(message.contains("Internal server error"));
        assert!(message.contains("test internal error"));
    }

    #[test]
    fn test_error_with_empty_strings() {
        let error = MatrixServerError::NetworkError("".to_string());
        let message = format!("{}", error);
        assert!(message.contains("Network error"));
        assert!(message.contains(""));
    }
}
