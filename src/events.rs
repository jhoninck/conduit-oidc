// Matrix Event Types and Handling
// Simplified version using owned types for better compatibility
// Focus: Core event structures and validation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Matrix event wrapper - simplified version for trading platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixEvent {
    pub event_id: String,
    pub event_type: EventType,
    pub content: EventContent,
    pub sender: String,
    pub room_id: String,
    pub origin_server_ts: u64,
    pub unsigned: Option<serde_json::Value>,
    pub state_key: Option<String>, // Present for state events
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    // Message events
    #[serde(rename = "m.room.message")]
    RoomMessage,
    #[serde(rename = "m.room.encrypted")]
    RoomEncrypted,
    #[serde(rename = "m.reaction")]
    Reaction,
    
    // State events
    #[serde(rename = "m.room.create")]
    RoomCreate,
    #[serde(rename = "m.room.member")]
    RoomMember,
    #[serde(rename = "m.room.power_levels")]
    RoomPowerLevels,
    #[serde(rename = "m.room.join_rules")]
    RoomJoinRules,
    #[serde(rename = "m.room.history_visibility")]
    RoomHistoryVisibility,
    #[serde(rename = "m.room.name")]
    RoomName,
    #[serde(rename = "m.room.topic")]
    RoomTopic,
    #[serde(rename = "m.room.avatar")]
    RoomAvatar,
    
    // Custom events for general use
    #[serde(rename = "custom.support.request")]
    CustomSupportRequest,
    #[serde(rename = "custom.alert")]
    CustomAlert,
    
    #[serde(untagged)]
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventContent {
    RoomMessage(RoomMessageContent),
    RoomMember(RoomMemberContent),
    RoomCreate(RoomCreateContent),
    RoomPowerLevels(RoomPowerLevelsContent),
    RoomJoinRules(RoomJoinRulesContent),
    RoomName(RoomNameContent),
    RoomTopic(RoomTopicContent),
    CustomSupport(CustomSupportContent),
    Raw(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomMessageContent {
    pub msgtype: MessageType,
    pub body: String,
    pub formatted_body: Option<String>,
    pub format: Option<String>,
    #[serde(rename = "m.relates_to")]
    pub relates_to: Option<RelatesTo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    #[serde(rename = "m.text")]
    Text,
    #[serde(rename = "m.notice")]
    Notice,
    #[serde(rename = "m.emote")]
    Emote,
    #[serde(rename = "m.image")]
    Image,
    #[serde(rename = "m.file")]
    File,
    #[serde(rename = "m.video")]
    Video,
    #[serde(rename = "m.audio")]
    Audio,
    #[serde(rename = "m.location")]
    Location,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatesTo {
    #[serde(rename = "m.in_reply_to")]
    pub in_reply_to: Option<InReplyTo>,
    #[serde(rename = "rel_type")]
    pub rel_type: String,
    #[serde(rename = "event_id")]
    pub event_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InReplyTo {
    pub event_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomMemberContent {
    pub membership: MembershipState,
    pub displayname: Option<String>,
    pub avatar_url: Option<String>,
    pub reason: Option<String>,
    pub is_direct: Option<bool>,
    pub third_party_invite: Option<ThirdPartyInvite>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MembershipState {
    #[serde(rename = "join")]
    Join,
    #[serde(rename = "leave")]
    Leave,
    #[serde(rename = "invite")]
    Invite,
    #[serde(rename = "ban")]
    Ban,
    #[serde(rename = "knock")]
    Knock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirdPartyInvite {
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomCreateContent {
    pub creator: String,
    #[serde(rename = "m.federate")]
    pub m_federate: Option<bool>,
    #[serde(rename = "room_version")]
    pub room_version: Option<String>,
    #[serde(rename = "predecessor")]
    pub predecessor: Option<RoomPredecessor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomPredecessor {
    pub room_id: String,
    pub event_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomPowerLevelsContent {
    pub users: Option<HashMap<String, i32>>,
    pub users_default: Option<i32>,
    pub events: Option<HashMap<String, i32>>,
    pub events_default: Option<i32>,
    pub state_default: Option<i32>,
    pub ban: Option<i32>,
    pub kick: Option<i32>,
    pub redact: Option<i32>,
    pub invite: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomJoinRulesContent {
    #[serde(rename = "join_rule")]
    pub join_rule: JoinRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JoinRule {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "invite")]
    Invite,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "knock")]
    Knock,
    #[serde(rename = "knock_restricted")]
    KnockRestricted,
    #[serde(rename = "restricted")]
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomNameContent {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomTopicContent {
    pub topic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSupportContent {
    pub request_type: String,
    pub description: String,
    pub priority: String,
    pub user_id: String,
    pub timestamp: u64,
}

// Helper functions for event creation
impl MatrixEvent {
    pub fn new(
        event_type: EventType,
        content: EventContent,
        sender: String,
        room_id: String,
    ) -> Self {
        Self {
            event_id: format!("${}", uuid::Uuid::new_v4().simple()),
            event_type,
            content,
            sender,
            room_id,
            origin_server_ts: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            unsigned: None,
            state_key: None,
        }
    }

    pub fn with_state_key(mut self, state_key: String) -> Self {
        self.state_key = Some(state_key);
        self
    }

    pub fn is_state_event(&self) -> bool {
        self.state_key.is_some()
    }

    pub fn is_message_event(&self) -> bool {
        matches!(self.event_type, EventType::RoomMessage)
    }
}

// Helper functions for content creation
impl EventContent {
    pub fn room_message(msgtype: MessageType, body: String) -> Self {
        EventContent::RoomMessage(RoomMessageContent {
            msgtype,
            body,
            formatted_body: None,
            format: None,
            relates_to: None,
        })
    }

    pub fn room_member(membership: MembershipState, displayname: Option<String>) -> Self {
        EventContent::RoomMember(RoomMemberContent {
            membership,
            displayname,
            avatar_url: None,
            reason: None,
            is_direct: None,
            third_party_invite: None,
        })
    }

    pub fn room_create(creator: String) -> Self {
        EventContent::RoomCreate(RoomCreateContent {
            creator,
            m_federate: Some(true),
            room_version: Some("9".to_string()),
            predecessor: None,
        })
    }

    pub fn room_power_levels() -> Self {
        EventContent::RoomPowerLevels(RoomPowerLevelsContent {
            users: Some(HashMap::new()),
            users_default: Some(0),
            events: Some(HashMap::new()),
            events_default: Some(0),
            state_default: Some(50),
            ban: Some(50),
            kick: Some(50),
            redact: Some(50),
            invite: Some(50),
        })
    }
}

// Validation functions
impl MatrixEvent {
    pub fn validate(&self) -> Result<(), EventValidationError> {
        if self.sender.is_empty() {
            return Err(EventValidationError::EmptySender);
        }
        if self.room_id.is_empty() {
            return Err(EventValidationError::EmptyRoomId);
        }
        if self.event_id.is_empty() {
            return Err(EventValidationError::EmptyEventId);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventValidationError {
    #[error("Sender cannot be empty")]
    EmptySender,
    #[error("Room ID cannot be empty")]
    EmptyRoomId,
    #[error("Event ID cannot be empty")]
    EmptyEventId,
}

// Type aliases for better readability
pub type EventId = String;
pub type RoomId = String;
pub type UserId = String;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_event_new() {
        let event = MatrixEvent::new(
            EventType::RoomMessage,
            EventContent::room_message(MessageType::Text, "Hello, World!".to_string()),
            "@user:localhost".to_string(),
            "!test:localhost".to_string(),
        );

        assert_eq!(event.event_type, EventType::RoomMessage);
        assert_eq!(event.room_id, "!test:localhost");
        assert_eq!(event.sender, "@user:localhost");
        assert!(matches!(event.content, EventContent::RoomMessage(_)));
        assert!(event.event_id.starts_with("$"));
    }

    #[test]
    fn test_matrix_event_with_state_key() {
        let event = MatrixEvent::new(
            EventType::RoomMember,
            EventContent::room_member(MembershipState::Join, Some("Test User".to_string())),
            "@user:localhost".to_string(),
            "!test:localhost".to_string(),
        ).with_state_key("@user:localhost".to_string());

        assert!(event.is_state_event());
        assert_eq!(event.state_key, Some("@user:localhost".to_string()));
    }

    #[test]
    fn test_matrix_event_validation() {
        let valid_event = MatrixEvent::new(
            EventType::RoomMessage,
            EventContent::room_message(MessageType::Text, "Test".to_string()),
            "@user:localhost".to_string(),
            "!test:localhost".to_string(),
        );
        assert!(valid_event.validate().is_ok());

        let invalid_event = MatrixEvent::new(
            EventType::RoomMessage,
            EventContent::room_message(MessageType::Text, "Test".to_string()),
            "".to_string(), // Empty sender
            "!test:localhost".to_string(),
        );
        assert!(invalid_event.validate().is_err());
    }

    #[test]
    fn test_event_type_serialization() {
        let event_types = vec![
            EventType::RoomMessage,
            EventType::RoomMember,
            EventType::RoomCreate,
            EventType::RoomJoinRules,
            EventType::RoomPowerLevels,
            EventType::RoomName,
            EventType::RoomTopic,
        ];

        for event_type in event_types {
            let json = serde_json::to_string(&event_type).unwrap();
            let deserialized: EventType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, event_type);
        }
    }

    #[test]
    fn test_message_type_serialization() {
        let message_types = vec![
            MessageType::Text,
            MessageType::Emote,
            MessageType::Notice,
            MessageType::Image,
            MessageType::File,
            MessageType::Audio,
            MessageType::Video,
            MessageType::Location,
        ];

        for msg_type in message_types {
            let json = serde_json::to_string(&msg_type).unwrap();
            let deserialized: MessageType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, msg_type);
        }
    }

    #[test]
    fn test_membership_state_serialization() {
        let membership_states = vec![
            MembershipState::Join,
            MembershipState::Leave,
            MembershipState::Invite,
            MembershipState::Ban,
        ];

        for state in membership_states {
            let json = serde_json::to_string(&state).unwrap();
            let deserialized: MembershipState = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, state);
        }
    }

    #[test]
    fn test_room_message_content_creation() {
        let content = EventContent::room_message(MessageType::Text, "Test message".to_string());
        
        if let EventContent::RoomMessage(room_content) = content {
            assert_eq!(room_content.msgtype, MessageType::Text);
            assert_eq!(room_content.body, "Test message");
            assert!(room_content.formatted_body.is_none());
            assert!(room_content.format.is_none());
            assert!(room_content.relates_to.is_none());
        } else {
            panic!("Expected RoomMessage content");
        }
    }

    #[test]
    fn test_room_member_content_creation() {
        let content = EventContent::room_member(MembershipState::Join, Some("Test User".to_string()));
        
        if let EventContent::RoomMember(member_content) = content {
            assert_eq!(member_content.membership, MembershipState::Join);
            assert_eq!(member_content.displayname, Some("Test User".to_string()));
            assert!(member_content.avatar_url.is_none());
            assert!(member_content.reason.is_none());
        } else {
            panic!("Expected RoomMember content");
        }
    }

    #[test]
    fn test_room_create_content_creation() {
        let content = EventContent::room_create("@creator:localhost".to_string());
        
        if let EventContent::RoomCreate(create_content) = content {
            assert_eq!(create_content.creator, "@creator:localhost");
            assert_eq!(create_content.m_federate, Some(true));
            assert_eq!(create_content.room_version, Some("9".to_string()));
            assert!(create_content.predecessor.is_none());
        } else {
            panic!("Expected RoomCreate content");
        }
    }

    #[test]
    fn test_room_power_levels_content_creation() {
        let content = EventContent::room_power_levels();
        
        if let EventContent::RoomPowerLevels(power_content) = content {
            assert_eq!(power_content.users_default, Some(0));
            assert_eq!(power_content.events_default, Some(0));
            assert_eq!(power_content.state_default, Some(50));
            assert_eq!(power_content.ban, Some(50));
            assert_eq!(power_content.kick, Some(50));
        } else {
            panic!("Expected RoomPowerLevels content");
        }
    }

    #[test]
    fn test_matrix_event_full_serialization() {
        let event = MatrixEvent::new(
            EventType::RoomMessage,
            EventContent::room_message(MessageType::Text, "Hello, World!".to_string()),
            "@user:localhost".to_string(),
            "!test:localhost".to_string(),
        );

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: MatrixEvent = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.event_type, event.event_type);
        assert_eq!(deserialized.room_id, event.room_id);
        assert_eq!(deserialized.sender, event.sender);
        assert!(matches!(deserialized.content, EventContent::RoomMessage(_)));
    }

    #[test]
    fn test_event_content_enum_serialization() {
        let message_content = EventContent::room_message(MessageType::Text, "Test".to_string());
        let member_content = EventContent::room_member(MembershipState::Join, None);

        // Test message content
        let json = serde_json::to_string(&message_content).unwrap();
        let deserialized: EventContent = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, EventContent::RoomMessage(_)));

        // Test member content
        let json = serde_json::to_string(&member_content).unwrap();
        let deserialized: EventContent = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, EventContent::RoomMember(_)));
    }

    #[test]
    fn test_relates_to_serialization() {
        let relates_to = RelatesTo {
            in_reply_to: Some(InReplyTo {
                event_id: "$reply_event".to_string(),
            }),
            rel_type: "m.reference".to_string(),
            event_id: "$event_id".to_string(),
        };

        let json = serde_json::to_string(&relates_to).unwrap();
        let deserialized: RelatesTo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.rel_type, relates_to.rel_type);
        assert_eq!(deserialized.event_id, relates_to.event_id);
        assert!(deserialized.in_reply_to.is_some());
        if let Some(in_reply_to) = deserialized.in_reply_to {
            assert_eq!(in_reply_to.event_id, "$reply_event");
        }
    }

    #[test]
    fn test_in_reply_to_serialization() {
        let in_reply_to = InReplyTo {
            event_id: "$event_id".to_string(),
        };

        let json = serde_json::to_string(&in_reply_to).unwrap();
        let deserialized: InReplyTo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.event_id, in_reply_to.event_id);
    }
}
