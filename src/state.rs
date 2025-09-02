// Matrix State Management
// Simplified state store for Matrix rooms and events
// Focus: Room state tracking and event processing

use crate::events::{MatrixEvent, EventType, EventContent, MembershipState};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Error types for state operations
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Room not found: {0}")]
    RoomNotFound(String),
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Invalid event: {0}")]
    InvalidEvent(String),
    #[error("State conflict: {0}")]
    StateConflict(String),
}

/// Power levels for room permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerLevels {
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

/// Room state representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomState {
    pub room_id: String,
    pub room_version: String,
    pub state_events: HashMap<(EventType, String), MatrixEvent>, // (type, state_key) -> event
    pub members: HashMap<String, MembershipState>,
    pub power_levels: PowerLevels,
    pub creator: String,
    pub join_rules: Option<String>,
    pub name: Option<String>,
    pub topic: Option<String>,
    pub avatar_url: Option<String>,
    pub history_visibility: Option<String>,
}

impl RoomState {
    pub fn new(room_id: String, creator: String, room_version: String) -> Self {
        let mut power_levels = PowerLevels {
            users: Some(HashMap::new()),
            users_default: Some(0),
            events: Some(HashMap::new()),
            events_default: Some(0),
            state_default: Some(50),
            ban: Some(50),
            kick: Some(50),
            redact: Some(50),
            invite: Some(50),
        };

        // Set creator as admin (power level 100)
        if let Some(ref mut users) = power_levels.users {
            users.insert(creator.clone(), 100);
        }

        let mut members = HashMap::new();
        members.insert(creator.clone(), MembershipState::Join);

        Self {
            room_id,
            room_version,
            state_events: HashMap::new(),
            members,
            power_levels,
            creator,
            join_rules: Some("invite".to_string()),
            name: None,
            topic: None,
            avatar_url: None,
            history_visibility: Some("shared".to_string()),
        }
    }

    /// Get user's power level in this room
    pub fn get_user_power_level(&self, user_id: &str) -> i32 {
        self.power_levels
            .users
            .as_ref()
            .and_then(|users| users.get(user_id))
            .copied()
            .unwrap_or(0)
    }

    /// Check if user has required power level
    pub fn user_has_power_level(&self, user_id: &str, required_level: i32) -> bool {
        self.get_user_power_level(user_id) >= required_level
    }

    /// Check if user is member of the room
    pub fn is_member(&self, user_id: &str) -> bool {
        matches!(
            self.members.get(user_id),
            Some(MembershipState::Join)
        )
    }

    /// Check if user is admin (power level 100)
    pub fn is_admin(&self, user_id: &str) -> bool {
        self.user_has_power_level(user_id, 100)
    }

    /// Check if user is moderator (power level 50+)
    pub fn is_moderator(&self, user_id: &str) -> bool {
        self.user_has_power_level(user_id, 50)
    }

    /// Add or update state event
    pub fn add_state_event(&mut self, event: MatrixEvent) -> Result<(), StateError> {
        let state_key = event.state_key.clone()
            .ok_or_else(|| StateError::InvalidEvent("State event missing state key".to_string()))?;

        // Validate event
        event.validate()
            .map_err(|e| StateError::InvalidEvent(e.to_string()))?;

        // Check permissions for state events
        if !self.user_has_power_level(&event.sender, 50) {
            return Err(StateError::InsufficientPermissions);
        }

        // Insert the state event
        self.state_events.insert(
            (event.event_type.clone(), state_key),
            event,
        );

        Ok(())
    }

    /// Process member event
    pub fn process_member_event(&mut self, event: &MatrixEvent) -> Result<(), StateError> {
        if let EventContent::RoomMember(ref content) = event.content {
            let user_id = event.sender.clone();
            
            match content.membership {
                MembershipState::Join => {
                    self.members.insert(user_id, content.membership.clone());
                }
                MembershipState::Leave => {
                    self.members.remove(&user_id);
                }
                MembershipState::Invite => {
                    // Handle invite logic
                }
                MembershipState::Ban => {
                    self.members.remove(&user_id);
                }
                MembershipState::Knock => {
                    // Handle knock logic
                }
            }
        }
        Ok(())
    }

    /// Get state event by type and state key
    pub fn get_state_event(&self, event_type: &EventType, state_key: &str) -> Option<&MatrixEvent> {
        self.state_events.get(&(event_type.clone(), state_key.to_string()))
    }

    /// Get all state events of a specific type
    pub fn get_state_events_by_type(&self, event_type: &EventType) -> Vec<&MatrixEvent> {
        self.state_events
            .iter()
            .filter(|((et, _), _)| et == event_type)
            .map(|(_, event)| event)
            .collect()
    }

    /// Get room summary for client
    pub fn get_summary(&self) -> RoomSummary {
        RoomSummary {
            room_id: self.room_id.clone(),
            name: self.name.clone(),
            topic: self.topic.clone(),
            member_count: self.members.len(),
            join_rules: self.join_rules.clone(),
            history_visibility: self.history_visibility.clone(),
        }
    }
}

/// Room summary for client consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSummary {
    pub room_id: String,
    pub name: Option<String>,
    pub topic: Option<String>,
    pub member_count: usize,
    pub join_rules: Option<String>,
    pub history_visibility: Option<String>,
}

/// State store trait for different storage backends
#[async_trait::async_trait]
pub trait StateStore: Send + Sync {
    async fn get_room(&self, room_id: &str) -> Result<Option<RoomState>, StateError>;
    async fn create_room(&self, room_state: RoomState) -> Result<(), StateError>;
    async fn update_room(&self, room_state: RoomState) -> Result<(), StateError>;
    async fn delete_room(&self, room_id: &str) -> Result<(), StateError>;
    async fn list_rooms(&self) -> Result<Vec<String>, StateError>;
    async fn room_exists(&self, room_id: &str) -> Result<bool, StateError>;
}

/// In-memory state store implementation
pub struct InMemoryStateStore {
    rooms: Arc<RwLock<HashMap<String, RoomState>>>,
}

impl InMemoryStateStore {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl StateStore for InMemoryStateStore {
    async fn get_room(&self, room_id: &str) -> Result<Option<RoomState>, StateError> {
        let rooms = self.rooms.read().await;
        Ok(rooms.get(room_id).cloned())
    }

    async fn create_room(&self, room_state: RoomState) -> Result<(), StateError> {
        let mut rooms = self.rooms.write().await;
        rooms.insert(room_state.room_id.clone(), room_state);
        Ok(())
    }

    async fn update_room(&self, room_state: RoomState) -> Result<(), StateError> {
        let mut rooms = self.rooms.write().await;
        
        // Ensure room exists
        let room = rooms.get_mut(&room_state.room_id)
            .ok_or_else(|| StateError::RoomNotFound(room_state.room_id.clone()))?;
        
        // Update room state
        *room = room_state;
        Ok(())
    }

    async fn delete_room(&self, room_id: &str) -> Result<(), StateError> {
        let mut rooms = self.rooms.write().await;
        
        if rooms.remove(room_id).is_some() {
            Ok(())
        } else {
            Err(StateError::RoomNotFound(room_id.to_string()))
        }
    }

    async fn list_rooms(&self) -> Result<Vec<String>, StateError> {
        let rooms = self.rooms.read().await;
        Ok(rooms.keys().cloned().collect())
    }

    async fn room_exists(&self, room_id: &str) -> Result<bool, StateError> {
        let rooms = self.rooms.read().await;
        Ok(rooms.contains_key(room_id))
    }
}

/// State conflict resolution
pub struct StateResolver;

impl StateResolver {
    /// Resolve state conflicts using Matrix state resolution algorithm
    pub fn resolve_state_conflicts(
        &self,
        auth_events: &[MatrixEvent],
        state_events: &[MatrixEvent],
    ) -> Result<Vec<MatrixEvent>, StateError> {
        // Simplified state resolution - in production this would implement
        // the full Matrix state resolution algorithm
        
        // For now, just return events in timestamp order
        let mut sorted_events = state_events.to_vec();
        sorted_events.sort_by_key(|e| e.origin_server_ts);
        
        Ok(sorted_events)
    }

    /// Validate event against auth events
    pub fn validate_event_auth(
        &self,
        event: &MatrixEvent,
        auth_events: &[MatrixEvent],
        room_state: &RoomState,
    ) -> Result<(), StateError> {
        // Basic auth validation
        if !room_state.is_member(&event.sender) {
            return Err(StateError::InsufficientPermissions);
        }

        // Check power levels for state events
        if event.is_state_event() {
            let required_level = room_state.power_levels.state_default.unwrap_or(50);
            if !room_state.user_has_power_level(&event.sender, required_level) {
                return Err(StateError::InsufficientPermissions);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{MatrixEvent, EventType, EventContent, RoomMessageContent, MessageType};

    fn create_test_event() -> MatrixEvent {
        MatrixEvent::new(
            EventType::RoomMessage,
            EventContent::room_message(MessageType::Text, "Test message".to_string()),
            "@user:localhost".to_string(),
            "!test:localhost".to_string(),
        )
    }

    fn create_test_room_state() -> RoomState {
        RoomState::new(
            "!test:localhost".to_string(),
            "@creator:localhost".to_string(),
            "6".to_string(),
        )
    }

    #[test]
    fn test_room_state_new() {
        let state = create_test_room_state();
        
        assert_eq!(state.room_id, "!test:localhost");
        assert_eq!(state.room_version, "6");
        assert_eq!(state.creator, "@creator:localhost");
        assert_eq!(state.history_visibility, Some("shared".to_string()));
        assert_eq!(state.power_levels.users_default, Some(0));
        assert_eq!(state.power_levels.events_default, Some(0));
    }

    #[test]
    fn test_room_state_add_state_event() {
        let mut state = create_test_room_state();
        let mut event = create_test_event();
        
        // Make it a state event by adding a state key
        event.state_key = Some("@user:localhost".to_string());
        // Use creator as sender to have sufficient power level
        event.sender = "@creator:localhost".to_string();
        
        let result = state.add_state_event(event.clone());
        assert!(result.is_ok());
        
        assert_eq!(state.state_events.len(), 1);
        let key = (event.event_type, "@user:localhost".to_string());
        assert!(state.state_events.contains_key(&key));
    }

    #[test]
    fn test_room_state_get_member() {
        let mut state = create_test_room_state();
        let member = MembershipState::Join;
        
        state.members.insert("@user:localhost".to_string(), member);
        
        let retrieved = state.members.get("@user:localhost");
        assert!(retrieved.is_some());
        assert_eq!(*retrieved.unwrap(), MembershipState::Join);
    }

    #[test]
    fn test_room_state_get_member_nonexistent() {
        let state = create_test_room_state();
        
        let retrieved = state.members.get("@nonexistent:localhost");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_room_state_add_member() {
        let mut state = create_test_room_state();
        let member = MembershipState::Join;
        
        state.members.insert("@newuser:localhost".to_string(), member);
        
        let retrieved = state.members.get("@newuser:localhost");
        assert!(retrieved.is_some());
        assert_eq!(*retrieved.unwrap(), MembershipState::Join);
    }

    #[test]
    fn test_room_state_update_member() {
        let mut state = create_test_room_state();
        let member = MembershipState::Join;
        
        state.members.insert("@user:localhost".to_string(), member);
        
        // Update membership
        if let Some(member) = state.members.get_mut("@user:localhost") {
            *member = MembershipState::Leave;
        }
        
        let retrieved = state.members.get("@user:localhost");
        assert!(retrieved.is_some());
        assert_eq!(*retrieved.unwrap(), MembershipState::Leave);
    }

    #[test]
    fn test_room_state_get_power_level() {
        let mut state = create_test_room_state();
        let member = MembershipState::Join;
        
        state.members.insert("@user:localhost".to_string(), member);
        
        let power_level = state.get_user_power_level("@user:localhost");
        assert_eq!(power_level, 0); // Default power level for non-members
        
        // Creator should have power level 100
        let creator_power_level = state.get_user_power_level("@creator:localhost");
        assert_eq!(creator_power_level, 100);
    }

    #[test]
    fn test_room_state_get_power_level_default() {
        let state = create_test_room_state();
        
        let power_level = state.get_user_power_level("@user:localhost");
        assert_eq!(power_level, 0); // Default power level for non-members
    }

    #[test]
    fn test_room_state_is_member() {
        let mut state = create_test_room_state();
        let member = MembershipState::Join;
        
        state.members.insert("@user:localhost".to_string(), member);
        
        assert!(state.is_member("@user:localhost"));
        assert!(!state.is_member("@nonmember:localhost"));
    }

    #[test]
    fn test_room_state_serialization() {
        let state = create_test_room_state();
        
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: RoomState = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.room_id, state.room_id);
        assert_eq!(deserialized.room_version, state.room_version);
        assert_eq!(deserialized.history_visibility, state.history_visibility);
    }

    #[test]
    fn test_room_member_serialization() {
        let member = MembershipState::Join;
        
        let json = serde_json::to_string(&member).unwrap();
        let deserialized: MembershipState = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized, member);
    }

    #[test]
    fn test_room_power_levels_serialization() {
        let power_levels = PowerLevels {
            users: Some(HashMap::new()),
            users_default: Some(0),
            events: Some(HashMap::new()),
            events_default: Some(50),
            state_default: Some(50),
            ban: Some(50),
            kick: Some(50),
            redact: Some(50),
            invite: Some(50),
        };
        
        let json = serde_json::to_string(&power_levels).unwrap();
        let deserialized: PowerLevels = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.users_default, power_levels.users_default);
        assert_eq!(deserialized.events_default, power_levels.events_default);
        assert_eq!(deserialized.state_default, power_levels.state_default);
        assert_eq!(deserialized.ban, power_levels.ban);
        assert_eq!(deserialized.kick, power_levels.kick);
    }

    #[test]
    fn test_in_memory_state_store() {
        let store = InMemoryStateStore::new();
        
        // Test that we can create a store
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let rooms = store.rooms.read().await;
            assert!(rooms.is_empty());
        });
    }

    #[test]
    fn test_in_memory_state_store_add_room() {
        let store = InMemoryStateStore::new();
        let room_state = create_test_room_state();
        
        // Use the async method
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            store.create_room(room_state.clone()).await.unwrap();
            
            let rooms = store.rooms.read().await;
            assert_eq!(rooms.len(), 1);
            assert!(rooms.contains_key(&room_state.room_id));
        });
    }

    #[test]
    fn test_in_memory_state_store_get_room() {
        let store = InMemoryStateStore::new();
        let room_state = create_test_room_state();
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            store.create_room(room_state.clone()).await.unwrap();
            
            let retrieved = store.get_room(&room_state.room_id).await.unwrap();
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap().room_id, room_state.room_id);
        });
    }

    #[test]
    fn test_in_memory_state_store_get_room_nonexistent() {
        let store = InMemoryStateStore::new();
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let retrieved = store.get_room("!nonexistent:localhost").await.unwrap();
            assert!(retrieved.is_none());
        });
    }

    #[test]
    fn test_in_memory_state_store_update_room() {
        let store = InMemoryStateStore::new();
        let mut room_state = create_test_room_state();
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            store.create_room(room_state.clone()).await.unwrap();
            
            // Update the room
            room_state.room_version = "7".to_string();
            store.update_room(room_state.clone()).await.unwrap();
            
            let retrieved = store.get_room(&room_state.room_id).await.unwrap();
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap().room_version, "7");
        });
    }

    #[test]
    fn test_in_memory_state_store_remove_room() {
        let store = InMemoryStateStore::new();
        let room_state = create_test_room_state();
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            store.create_room(room_state.clone()).await.unwrap();
            
            // Check initial state
            {
                let rooms = store.rooms.read().await;
                assert_eq!(rooms.len(), 1);
            } // Drop read lock here
            
            store.delete_room(&room_state.room_id).await.unwrap();
            
            // Check final state
            {
                let rooms = store.rooms.read().await;
                assert_eq!(rooms.len(), 0);
            }
        });
    }

    #[test]
    fn test_in_memory_state_store_list_rooms() {
        let store = InMemoryStateStore::new();
        let room_state1 = create_test_room_state();
        let mut room_state2 = create_test_room_state();
        room_state2.room_id = "!test2:localhost".to_string();
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            store.create_room(room_state1.clone()).await.unwrap();
            store.create_room(room_state2.clone()).await.unwrap();
            
            let rooms = store.list_rooms().await.unwrap();
            assert_eq!(rooms.len(), 2);
            assert!(rooms.contains(&room_state1.room_id));
            assert!(rooms.contains(&room_state2.room_id));
        });
    }
}
