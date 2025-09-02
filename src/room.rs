// Room Management Handler
// Simplified room management for Matrix chat system
// Focus: Room creation, membership, and message handling

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::events::{
    MatrixEvent, EventType, EventContent, RoomCreateContent, RoomMemberContent, 
    RoomMessageContent, MessageType, MembershipState, RoomPowerLevelsContent,
    RoomJoinRulesContent, JoinRule, RoomNameContent, RoomTopicContent
};
use crate::state::{StateStore, RoomState, StateError};
use crate::auth::{AuthenticatedUser, AuthError};

#[derive(Error, Debug)]
pub enum RoomError {
    #[error("Room not found: {0}")]
    RoomNotFound(String),
    
    #[error("User not in room: {0}")]
    UserNotInRoom(String),
    
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),
    
    #[error("Room already exists: {0}")]
    RoomAlreadyExists(String),
    
    #[error("Invalid room configuration: {0}")]
    InvalidRoomConfig(String),
    
    #[error("Message too large: {0} bytes")]
    MessageTooLarge(usize),
    
    #[error("State error: {0}")]
    StateError(#[from] StateError),
    
    #[error("Auth error: {0}")]
    AuthError(#[from] AuthError),
}

impl RoomError {
    pub fn status_code(&self) -> u16 {
        match self {
            RoomError::RoomNotFound(_) => 404,
            RoomError::UserNotInRoom(_) => 403,
            RoomError::InsufficientPermissions(_) => 403,
            RoomError::RoomAlreadyExists(_) => 409,
            RoomError::InvalidRoomConfig(_) => 400,
            RoomError::MessageTooLarge(_) => 413,
            RoomError::StateError(_) => 500,
            RoomError::AuthError(auth_err) => auth_err.status_code(),
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            RoomError::RoomNotFound(_) => "M_NOT_FOUND",
            RoomError::UserNotInRoom(_) => "M_FORBIDDEN",
            RoomError::InsufficientPermissions(_) => "M_FORBIDDEN",
            RoomError::RoomAlreadyExists(_) => "M_ROOM_IN_USE",
            RoomError::InvalidRoomConfig(_) => "M_BAD_JSON",
            RoomError::MessageTooLarge(_) => "M_TOO_LARGE",
            RoomError::StateError(_) => "M_UNKNOWN",
            RoomError::AuthError(auth_err) => auth_err.error_code(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConfig {
    pub name: Option<String>,
    pub topic: Option<String>,
    pub room_alias_name: Option<String>,
    pub invite: Vec<String>,
    pub room_version: Option<String>,
    pub creation_content: Option<serde_json::Value>,
    pub initial_state: Vec<StateEventConfig>,
    pub preset: Option<RoomPreset>,
    pub is_direct: Option<bool>,
    pub power_level_content_override: Option<RoomPowerLevelsContent>,
    pub federate: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEventConfig {
    #[serde(rename = "type")]
    pub event_type: String,
    pub state_key: String,
    pub content: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoomPreset {
    #[serde(rename = "private_chat")]
    PrivateChat,
    #[serde(rename = "public_chat")]
    PublicChat,
    #[serde(rename = "trusted_private_chat")]
    TrustedPrivateChat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub room_config: RoomConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoomResponse {
    pub room_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRoomRequest {
    pub room_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRoomResponse {
    pub room_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveRoomRequest {
    pub room_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub room_id: String,
    pub msgtype: MessageType,
    pub body: String,
    pub formatted_body: Option<String>,
    pub format: Option<String>,
    pub relates_to: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub event_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMessagesRequest {
    pub room_id: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMessagesResponse {
    pub chunk: Vec<MatrixEvent>,
    pub start: String,
    pub end: String,
}

/// Room handler - manages room operations
pub struct RoomHandler {
    state_store: Arc<dyn StateStore + Send + Sync>,
}

impl RoomHandler {
    pub fn new(state_store: Arc<dyn StateStore + Send + Sync>) -> Self {
        Self { state_store }
    }

    /// Create a new room
    pub async fn create_room(
        &self,
        creator: &AuthenticatedUser,
        config: RoomConfig,
    ) -> Result<CreateRoomResponse, RoomError> {
        // Generate room ID
        let room_id = self.generate_room_id(&config.room_alias_name)?;
        
        // Check if room already exists
        if self.state_store.room_exists(&room_id).await? {
            return Err(RoomError::RoomAlreadyExists(room_id));
        }

        // Create room state
        let room_version = config.room_version.unwrap_or_else(|| "9".to_string());
        let mut room_state = RoomState::new(
            room_id.clone(),
            creator.user_id.clone(),
            room_version,
        );

        // Apply initial state events
        for state_config in &config.initial_state {
            let event = MatrixEvent::new(
                EventType::Custom(state_config.event_type.clone()),
                EventContent::Raw(state_config.content.clone()),
                creator.user_id.clone(),
                room_id.clone(),
            ).with_state_key(state_config.state_key.clone());

            room_state.add_state_event(event)?;
        }

        // Set room name and topic if provided
        if let Some(name) = config.name {
            let name_event = MatrixEvent::new(
                EventType::RoomName,
                EventContent::RoomName(RoomNameContent { name }),
                creator.user_id.clone(),
                room_id.clone(),
            ).with_state_key("".to_string());
            room_state.add_state_event(name_event)?;
        }

        if let Some(topic) = config.topic {
            let topic_event = MatrixEvent::new(
                EventType::RoomTopic,
                EventContent::RoomTopic(RoomTopicContent { topic }),
                creator.user_id.clone(),
                room_id.clone(),
            ).with_state_key("".to_string());
            room_state.add_state_event(topic_event)?;
        }

        // Set join rules based on preset
        let join_rule = match config.preset {
            Some(RoomPreset::PublicChat) => "public",
            Some(RoomPreset::PrivateChat) | Some(RoomPreset::TrustedPrivateChat) => "invite",
            None => "invite",
        };
        room_state.join_rules = Some(join_rule.to_string());

        // Set history visibility
        room_state.history_visibility = Some("shared".to_string());

        // Store room in state store
        self.state_store.create_room(room_state).await?;

        Ok(CreateRoomResponse { room_id })
    }

    /// Join a room
    pub async fn join_room(
        &self,
        user: &AuthenticatedUser,
        request: JoinRoomRequest,
    ) -> Result<JoinRoomResponse, RoomError> {
        let room_id = request.room_id;

        // Get room state
        let mut room_state = self.state_store
            .get_room(&room_id)
            .await?
            .ok_or_else(|| RoomError::RoomNotFound(room_id.clone()))?;

        // Check join rules
        let join_rule = room_state.join_rules.as_deref().unwrap_or("invite");
        match join_rule {
            "public" => {
                // Anyone can join public rooms
            }
            "invite" => {
                // Check if user was invited or is admin
                if !room_state.is_admin(&user.user_id) {
                        return Err(RoomError::InsufficientPermissions(
                        "Room requires invitation".to_string()
                        ));
                }
            }
            _ => {
                return Err(RoomError::InsufficientPermissions(
                    format!("Unknown join rule: {}", join_rule)
                ));
            }
        }

        // Add user to room
        let member_event = MatrixEvent::new(
            EventType::RoomMember,
            EventContent::RoomMember(RoomMemberContent {
                membership: MembershipState::Join,
                displayname: None,
                avatar_url: None,
                reason: request.reason,
                is_direct: None,
                third_party_invite: None,
            }),
            user.user_id.clone(),
            room_id.clone(),
        ).with_state_key(user.user_id.clone());

        room_state.process_member_event(&member_event)?;

        // Update room state
        self.state_store.update_room(room_state).await?;

        Ok(JoinRoomResponse { room_id })
    }

    /// Leave a room
    pub async fn leave_room(
        &self,
        user: &AuthenticatedUser,
        request: LeaveRoomRequest,
    ) -> Result<(), RoomError> {
        let room_id = request.room_id;

        // Get room state
        let mut room_state = self.state_store
            .get_room(&room_id)
            .await?
            .ok_or_else(|| RoomError::RoomNotFound(room_id.clone()))?;

        // Check if user is in room
        if !room_state.is_member(&user.user_id) {
            return Err(RoomError::UserNotInRoom(user.user_id.clone()));
        }

        // Remove user from room
        let member_event = MatrixEvent::new(
            EventType::RoomMember,
            EventContent::RoomMember(RoomMemberContent {
                membership: MembershipState::Leave,
                displayname: None,
                avatar_url: None,
                reason: request.reason,
                is_direct: None,
                third_party_invite: None,
            }),
            user.user_id.clone(),
            room_id.clone(),
        ).with_state_key(user.user_id.clone());

        room_state.process_member_event(&member_event)?;

        // Update room state
        self.state_store.update_room(room_state).await?;

        Ok(())
    }

    /// Send a message to a room
    pub async fn send_message(
        &self,
        user: &AuthenticatedUser,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, RoomError> {
        let room_id = request.room_id;

        // Get room state
        let room_state = self.state_store
            .get_room(&room_id)
            .await?
            .ok_or_else(|| RoomError::RoomNotFound(room_id.clone()))?;

        // Check if user is in room
        if !room_state.is_member(&user.user_id) {
            return Err(RoomError::UserNotInRoom(user.user_id.clone()));
        }

        // Check message size
        if request.body.len() > 65536 {
            return Err(RoomError::MessageTooLarge(request.body.len()));
        }

        // Create message event
        let message_content = RoomMessageContent {
            msgtype: request.msgtype,
            body: request.body,
            formatted_body: request.formatted_body,
            format: request.format,
            relates_to: None, // TODO: Convert request.relates_to to proper RelatesTo type
        };

        let event = MatrixEvent::new(
            EventType::RoomMessage,
            EventContent::RoomMessage(message_content),
            user.user_id.clone(),
            room_id.clone(),
        );

        // Generate event ID
        let event_id = format!("${}", Uuid::new_v4().simple());

        // TODO: Store event in timeline
        // For now, just return success

        Ok(SendMessageResponse { event_id })
    }

    /// Get messages from a room
    pub async fn get_messages(
        &self,
        user: &AuthenticatedUser,
        request: GetMessagesRequest,
    ) -> Result<GetMessagesResponse, RoomError> {
        let room_id = request.room_id;

        // Get room state
        let room_state = self.state_store
            .get_room(&room_id)
            .await?
            .ok_or_else(|| RoomError::RoomNotFound(room_id.clone()))?;

        // Check if user is in room
        if !room_state.is_member(&user.user_id) {
            return Err(RoomError::UserNotInRoom(user.user_id.clone()));
        }

        // TODO: Implement actual message retrieval from timeline
        // For now, return empty response
        Ok(GetMessagesResponse {
            chunk: vec![],
            start: request.from.unwrap_or_else(|| "0".to_string()),
            end: request.to.unwrap_or_else(|| "0".to_string()),
        })
    }

    /// Generate a room ID
    fn generate_room_id(&self, alias: &Option<String>) -> Result<String, RoomError> {
        if let Some(alias_name) = alias {
            // Use alias if provided
            Ok(format!("#{}:matrix.local", alias_name))
        } else {
            // Generate random room ID
            Ok(format!("!{}:matrix.local", Uuid::new_v4().simple()))
        }
    }

    /// Get room summary
    pub async fn get_room_summary(
        &self,
        user: &AuthenticatedUser,
        room_id: &str,
    ) -> Result<serde_json::Value, RoomError> {
        // Get room state
        let room_state = self.state_store
            .get_room(room_id)
            .await?
            .ok_or_else(|| RoomError::RoomNotFound(room_id.to_string()))?;

        // Check if user is in room
        if !room_state.is_member(&user.user_id) {
            return Err(RoomError::UserNotInRoom(user.user_id.clone()));
        }

        // Return room summary
        let summary = room_state.get_summary();
        Ok(serde_json::to_value(summary)
            .map_err(|e| RoomError::InvalidRoomConfig(e.to_string()))?)
    }

    /// List user's rooms
    pub async fn list_rooms(
        &self,
        user: &AuthenticatedUser,
    ) -> Result<Vec<String>, RoomError> {
        // Get all rooms
        let all_rooms = self.state_store.list_rooms().await?;
        
        // Filter to rooms user is in
        let mut user_rooms = Vec::new();
        for room_id in all_rooms {
            if let Some(room_state) = self.state_store.get_room(&room_id).await? {
                if room_state.is_member(&user.user_id) {
                    user_rooms.push(room_id);
                }
            }
        }

        Ok(user_rooms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{MatrixEvent, EventType, EventContent, RoomMessageContent, MessageType};

    fn create_test_room_config() -> RoomConfig {
        RoomConfig {
            name: Some("Test Room".to_string()),
            topic: Some("A test room".to_string()),
            room_alias_name: Some("testroom".to_string()),
            invite: vec![],
            room_version: Some("9".to_string()),
            creation_content: None,
            initial_state: vec![],
            preset: Some(RoomPreset::PublicChat),
            is_direct: Some(false),
            power_level_content_override: None,
            federate: Some(true),
        }
    }

    fn create_test_event() -> MatrixEvent {
        MatrixEvent::new(
            EventType::RoomMessage,
            EventContent::RoomMessage(RoomMessageContent {
                body: "Hello, world!".to_string(),
                msgtype: MessageType::Text,
                relates_to: None,
                format: None,
                formatted_body: None,
            }),
            "@user:localhost".to_string(),
            "!testroom:localhost".to_string(),
        )
    }

    #[test]
    fn test_room_config_new() {
        let config = create_test_room_config();
        assert_eq!(config.name, Some("Test Room".to_string()));
        assert_eq!(config.topic, Some("A test room".to_string()));
        assert_eq!(config.room_alias_name, Some("testroom".to_string()));
        assert_eq!(config.preset, Some(RoomPreset::PublicChat));
        assert_eq!(config.is_direct, Some(false));
        assert_eq!(config.federate, Some(true));
    }

    #[test]
    fn test_room_config_serialization() {
        let config = create_test_room_config();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: RoomConfig = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.topic, deserialized.topic);
        assert_eq!(config.room_alias_name, deserialized.room_alias_name);
        assert_eq!(config.preset, deserialized.preset);
    }

    #[test]
    fn test_room_preset_serialization() {
        let presets = vec![
            RoomPreset::PrivateChat,
            RoomPreset::PublicChat,
            RoomPreset::TrustedPrivateChat,
        ];
        
        for preset in presets {
            let serialized = serde_json::to_string(&preset).unwrap();
            let deserialized: RoomPreset = serde_json::from_str(&serialized).unwrap();
            assert_eq!(preset, deserialized);
        }
    }

    #[test]
    fn test_room_error_status_codes() {
        assert_eq!(RoomError::RoomNotFound("room".to_string()).status_code(), 404);
        assert_eq!(RoomError::UserNotInRoom("user".to_string()).status_code(), 403);
        assert_eq!(RoomError::InsufficientPermissions("user".to_string()).status_code(), 403);
        assert_eq!(RoomError::RoomAlreadyExists("room".to_string()).status_code(), 409);
        assert_eq!(RoomError::InvalidRoomConfig("config".to_string()).status_code(), 400);
        assert_eq!(RoomError::MessageTooLarge(1000).status_code(), 413);
    }

    #[test]
    fn test_room_error_codes() {
        assert_eq!(RoomError::RoomNotFound("room".to_string()).error_code(), "M_NOT_FOUND");
        assert_eq!(RoomError::UserNotInRoom("user".to_string()).error_code(), "M_FORBIDDEN");
        assert_eq!(RoomError::InsufficientPermissions("user".to_string()).error_code(), "M_FORBIDDEN");
        assert_eq!(RoomError::RoomAlreadyExists("room".to_string()).error_code(), "M_ROOM_IN_USE");
        assert_eq!(RoomError::InvalidRoomConfig("config".to_string()).error_code(), "M_BAD_JSON");
        assert_eq!(RoomError::MessageTooLarge(1000).error_code(), "M_TOO_LARGE");
    }

    #[test]
    fn test_create_room_request_serialization() {
        let request = CreateRoomRequest {
            room_config: create_test_room_config(),
        };
        
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: CreateRoomRequest = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(request.room_config.name, deserialized.room_config.name);
        assert_eq!(request.room_config.topic, deserialized.room_config.topic);
    }

    #[test]
    fn test_create_room_response_serialization() {
        let response = CreateRoomResponse {
            room_id: "!testroom:localhost".to_string(),
        };
        
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: CreateRoomResponse = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(response.room_id, deserialized.room_id);
    }

    #[test]
    fn test_join_room_request_serialization() {
        let request = JoinRoomRequest {
            room_id: "!testroom:localhost".to_string(),
            reason: Some("Testing".to_string()),
        };
        
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: JoinRoomRequest = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(request.room_id, deserialized.room_id);
        assert_eq!(request.reason, deserialized.reason);
    }

    #[test]
    fn test_join_room_response_serialization() {
        let response = JoinRoomResponse {
            room_id: "!testroom:localhost".to_string(),
        };
        
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: JoinRoomResponse = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(response.room_id, deserialized.room_id);
    }

    #[test]
    fn test_leave_room_request_serialization() {
        let request = LeaveRoomRequest {
            room_id: "!testroom:localhost".to_string(),
            reason: Some("Testing".to_string()),
        };
        
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: LeaveRoomRequest = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(request.room_id, deserialized.room_id);
        assert_eq!(request.reason, deserialized.reason);
    }

    #[test]
    fn test_state_event_config_serialization() {
        let config = StateEventConfig {
            event_type: "m.room.name".to_string(),
            state_key: "".to_string(),
            content: serde_json::json!({"name": "Test Room"}),
        };
        
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: StateEventConfig = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(config.event_type, deserialized.event_type);
        assert_eq!(config.state_key, deserialized.state_key);
        assert_eq!(config.content, deserialized.content);
    }
}
