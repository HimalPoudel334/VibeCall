use crate::calls::entities::CallStatus;
use crate::calls::service::CallService;
use crate::rooms::service::RoomService;
use actix_ws::Message;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{self, UnboundedSender};

pub type Sender = UnboundedSender<OutgoingMessage>;

#[derive(Debug, Clone)]
pub struct Connection {
    pub user_id: i32,
    pub room_id: String,
    pub call_id: Option<i32>, // Track which call this connection belongs to
    pub sender: Sender,
}

pub struct SignalingServer {
    // In-memory tracking
    connections: DashMap<i32, Connection>,
    rooms: DashMap<String, Vec<i32>>,

    // Injected services for database operations
    call_service: Arc<dyn CallService>,
    room_service: Arc<dyn RoomService>,
}

impl SignalingServer {
    pub fn new(call_service: Arc<dyn CallService>, room_service: Arc<dyn RoomService>) -> Self {
        Self {
            connections: DashMap::new(),
            rooms: DashMap::new(),
            call_service,
            room_service,
        }
    }

    // User joins a room via WebSocket
    pub async fn add_connection(
        &self,
        user_id: i32,
        room_id: String,
        sender: Sender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Add to in-memory tracking
        let connection = Connection {
            user_id,
            room_id: room_id.clone(),
            call_id: None,
            sender,
        };

        self.connections.insert(user_id, connection);

        self.rooms
            .entry(room_id.clone())
            .or_insert_with(Vec::new)
            .push(user_id);

        // 2. Database: Record room membership
        self.room_service.join_room(&room_id, user_id).await?;

        Ok(())
    }

    // User starts or joins an active call
    pub async fn join_call(
        &self,
        user_id: i32,
        room_id: String,
    ) -> Result<i32, Box<dyn std::error::Error>> {
        // 1. Check if there's an active call in this room
        let active_calls = self.call_service.get_calls_by_room_id(&room_id).await?;

        let call_id = if let Some(active_call) =
            active_calls.iter().find(|c| c.status == CallStatus::Active)
        {
            // Join existing call
            active_call.id
        } else {
            // Create new call
            let call = self
                .call_service
                .create_call(room_id.clone(), user_id, CallStatus::Active.to_string())
                .await?;
            call.id
        };

        // 2. Add user as participant in database
        self.call_service
            .add_call_participant(call_id, user_id)
            .await?;

        // 3. Update in-memory connection with call_id
        if let Some(mut connection) = self.connections.get_mut(&user_id) {
            connection.call_id = Some(call_id);
        }

        Ok(call_id)
    }

    // User disconnects
    pub async fn remove_connection(&self, user_id: i32) {
        if let Some((_, connection)) = self.connections.remove(&user_id) {
            // 1. Remove from in-memory room tracking
            if let Some(mut room_users) = self.rooms.get_mut(&connection.room_id) {
                room_users.retain(|&id| id != user_id);
            }

            // 2. Database: Mark participant as left (if in a call)
            if let Some(call_id) = connection.call_id {
                let _ = self
                    .call_service
                    .remove_call_participant(call_id, user_id)
                    .await;
            }

            // 4. Database: Mark as left room
            let _ = self
                .room_service
                .leave_room(&connection.room_id, user_id)
                .await;
        }
    }

    // Update send_to_user to accept string
    pub fn send_to_user(&self, user_id: i32, message: &str) -> Result<(), String> {
        if let Some(connection) = self.connections.get(&user_id) {
            let msg = Message::Text(message.into());
            connection
                .sender
                .send(msg)
                .map_err(|e| format!("Failed to send message: {}", e))?;
            Ok(())
        } else {
            Err(format!("User {} not connected", user_id))
        }
    }

    pub fn broadcast_to_room(&self, room_id: &str, sender_id: i32, message: &str) {
        if let Some(room_users) = self.rooms.get(room_id) {
            for &user_id in room_users.iter() {
                if user_id != sender_id {
                    let _ = self.send_to_user(user_id, message);
                }
            }
        }
    }

    // Get currently connected users in room (in-memory)
    pub fn get_room_users(&self, room_id: &str) -> Vec<i32> {
        self.rooms
            .get(room_id)
            .map(|users| users.clone())
            .unwrap_or_default()
    }
}
