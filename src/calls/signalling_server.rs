use crate::calls::service::CallService;
use crate::calls::{entities::CallStatus, websocket::OutgoingMessage};
use crate::rooms::service::RoomService;
use crate::shared::response::AppError;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub type Sender = UnboundedSender<OutgoingMessage>;

#[derive(Debug, Clone)]
pub struct Connection {
    pub user_id: i32,
    pub room_id: String,
    pub call_id: Option<i32>,
    pub sender: Sender,
}

pub struct SignalingServer {
    connections: DashMap<i32, Connection>,
    rooms: DashMap<String, Vec<i32>>,

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

        self.rooms.entry(room_id.clone()).or_default().push(user_id);

        self.room_service.join_room(&room_id, user_id).await?;

        Ok(())
    }

    pub async fn join_call(&self, user_id: i32, room_id: String) -> Result<i32, AppError> {
        let active_calls = self
            .call_service
            .get_active_calls_by_room_id(&room_id)
            .await?;

        let call_id = if let Some(active_call) = active_calls.first() {
            self.call_service
                .add_call_participant(active_call.id, user_id)
                .await?;

            active_call.id
        } else {
            let call = self
                .call_service
                .create_call(room_id.clone(), user_id, CallStatus::Active.to_string())
                .await?;
            call.id
        };

        if let Some(mut connection) = self.connections.get_mut(&user_id) {
            connection.call_id = Some(call_id);
        }

        Ok(call_id)
    }

    pub async fn remove_connection(&self, user_id: i32) {
        if let Some((_, connection)) = self.connections.remove(&user_id) {
            if let Some(mut room_users) = self.rooms.get_mut(&connection.room_id) {
                room_users.retain(|&id| id != user_id);
            }

            if let Some(call_id) = connection.call_id {
                let _ = self
                    .call_service
                    .remove_call_participant(call_id, user_id)
                    .await;
            }
            //
            // let _ = self
            //     .room_service
            //     .leave_room(&connection.room_id, user_id)
            //     .await;
        }
    }

    pub fn send_to_user(&self, user_id: i32, message: &str) -> Result<(), String> {
        if let Some(connection) = self.connections.get(&user_id) {
            let msg = OutgoingMessage::Text(message.into());
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

    pub async fn get_room_users(&self, room_id: &str) -> Vec<i32> {
        self.rooms
            .get(room_id)
            .map(|users| users.clone())
            .unwrap_or_default()
    }
}
