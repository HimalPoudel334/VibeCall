use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    calls::{
        entities::{Call, CallParticipant, CallStatus},
        repository::CallRepository,
    },
    rooms::RoomService,
    shared::response::AppError,
    users::UserService,
};

#[async_trait]
pub trait CallService: Send + Sync {
    async fn create_call(
        &self,
        room_id: String,
        caller_id: i32,
        status: String,
    ) -> Result<Call, AppError>;

    async fn get_call_by_id(&self, call_id: i32) -> Result<Option<Call>, AppError>;

    async fn update_call_status(&self, call_id: i32, status: String) -> Result<(), AppError>;

    async fn end_call(&self, call_id: i32, user_id: i32) -> Result<(), AppError>;

    async fn get_calls_by_room_id(&self, room_id: &str) -> Result<Vec<Call>, AppError>;

    async fn get_calls_by_user_id(&self, user_id: i32) -> Result<Vec<Call>, AppError>;

    async fn get_user_participated_calls(&self, user_id: i32) -> Result<Vec<Call>, AppError>;

    async fn get_active_calls(&self) -> Result<Vec<Call>, AppError>;

    async fn add_call_participant(&self, call_id: i32, user_id: i32) -> Result<(), AppError>;

    async fn remove_call_participant(&self, call_id: i32, user_id: i32) -> Result<(), AppError>;

    async fn list_call_participants(&self, call_id: i32) -> Result<Vec<CallParticipant>, AppError>;

    async fn list_active_participants(
        &self,
        call_id: i32,
    ) -> Result<Vec<CallParticipant>, AppError>;

    async fn count_active_participants(&self, call_id: i32) -> Result<i64, AppError>;
}

pub struct CallServiceImpl {
    call_repo: Arc<dyn CallRepository + Send + Sync>,
    room_service: Arc<dyn RoomService + Send + Sync>,
    user_service: Arc<dyn UserService + Send + Sync>,
}

impl CallServiceImpl {
    pub fn new(
        call_repo: Arc<dyn CallRepository + Send + Sync>,
        room_service: Arc<dyn RoomService + Send + Sync>,
        user_service: Arc<dyn UserService + Send + Sync>,
    ) -> Self {
        Self {
            call_repo,
            room_service,
            user_service,
        }
    }
}

#[async_trait]
impl CallService for CallServiceImpl {
    async fn create_call(
        &self,
        room_id: String,
        caller_id: i32,
        status: String,
    ) -> Result<Call, AppError> {
        if room_id.trim().is_empty() {
            return Err(AppError::Validation("Room ID cannot be empty".into()));
        }

        let call_status = status.parse::<CallStatus>()?;
        match call_status {
            CallStatus::Active | CallStatus::Initiated => {}
            _ => {
                return Err(AppError::Validation(
                    "Call status must be either 'active' or 'initiated'".into(),
                ));
            }
        }

        let room = self
            .room_service
            .get_room(&room_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Room {} not found", room_id)))?;

        if !room.is_active {
            return Err(AppError::Validation(format!(
                "Room {} is not active",
                room_id
            )));
        }

        let _caller = self
            .user_service
            .get_by_id(caller_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", caller_id)))?;

        if !self
            .room_service
            .is_user_in_room(&room_id, caller_id)
            .await?
        {
            return Err(AppError::Unauthorized(format!(
                "User {} is not a member of room {}",
                caller_id, room_id
            )));
        }

        let call = self
            .call_repo
            .create_call(room_id, caller_id, call_status)
            .await?;

        self.call_repo
            .add_call_participant(call.id, caller_id)
            .await?;

        Ok(call)
    }

    async fn get_call_by_id(&self, call_id: i32) -> Result<Option<Call>, AppError> {
        self.call_repo.get_call_by_id(call_id).await
    }

    async fn update_call_status(&self, call_id: i32, status: String) -> Result<(), AppError> {
        let status = status.parse::<CallStatus>()?;

        let _call = self
            .call_repo
            .get_call_by_id(call_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Call {} not found", call_id)))?;

        self.call_repo.update_call_status(call_id, status).await
    }

    async fn end_call(&self, call_id: i32, user_id: i32) -> Result<(), AppError> {
        // Check if call exists
        let call = self
            .call_repo
            .get_call_by_id(call_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Call {} not found", call_id)))?;

        // Check if user is the caller or a room owner
        let is_caller = call.caller_id == user_id;
        let is_room_owner = self
            .room_service
            .is_user_owner(&call.room_id, user_id)
            .await?;

        if !is_caller && !is_room_owner {
            return Err(AppError::Unauthorized(
                "Only the caller or room owner can end the call".into(),
            ));
        }

        self.call_repo.end_call(call_id).await
    }

    async fn get_calls_by_room_id(&self, room_id: &str) -> Result<Vec<Call>, AppError> {
        // Validate room exists
        self.room_service
            .get_room(room_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Room {} not found", room_id)))?;

        self.call_repo.get_calls_by_room_id(room_id).await
    }

    async fn get_calls_by_user_id(&self, user_id: i32) -> Result<Vec<Call>, AppError> {
        // Validate user exists
        self.user_service
            .get_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

        self.call_repo.get_calls_by_user_id(user_id).await
    }

    async fn get_user_participated_calls(&self, user_id: i32) -> Result<Vec<Call>, AppError> {
        // Validate user exists
        self.user_service
            .get_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

        self.call_repo.get_user_participated_calls(user_id).await
    }

    async fn get_active_calls(&self) -> Result<Vec<Call>, AppError> {
        self.call_repo.get_active_calls().await
    }

    async fn add_call_participant(&self, call_id: i32, user_id: i32) -> Result<(), AppError> {
        // Check if call exists and is active
        let call = self
            .call_repo
            .get_call_by_id(call_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Call {} not found", call_id)))?;
        if call.status != CallStatus::Active {
            return Err(AppError::Validation(format!(
                "Cannot add participant to non-active call {}",
                call_id
            )));
        }

        // Check if user exists
        self.user_service
            .get_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

        // Check if user is in the room
        if !self
            .room_service
            .is_user_in_room(&call.room_id, user_id)
            .await?
        {
            return Err(AppError::Unauthorized(format!(
                "User {} is not a member of room {}",
                user_id, call.room_id
            )));
        }

        // Check room's participant limit
        let room = self
            .room_service
            .get_room(&call.room_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Room {} not found", call.room_id)))?;
        let active_participants = self
            .call_repo
            .list_active_participants(call_id)
            .await?
            .len() as i64;
        if active_participants >= room.max_participants as i64 {
            return Err(AppError::Validation(format!(
                "Call {} has reached the room's participant limit ({})",
                call_id, room.max_participants
            )));
        }

        self.call_repo.add_call_participant(call_id, user_id).await
    }

    async fn remove_call_participant(&self, call_id: i32, user_id: i32) -> Result<(), AppError> {
        // Check if call exists
        let _call = self
            .call_repo
            .get_call_by_id(call_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Call {} not found", call_id)))?;

        // Check if user is a participant
        let participants = self.call_repo.list_active_participants(call_id).await?;
        if !participants.iter().any(|p| p.user_id == user_id) {
            return Err(AppError::NotFound(format!(
                "User {} is not a participant in call {}",
                user_id, call_id
            )));
        }

        self.call_repo
            .remove_call_participant(call_id, user_id)
            .await?;

        // If no participants remain, end the call
        let remaining_participants = self.call_repo.count_active_participants(call_id).await?;
        if remaining_participants == 0 {
            self.call_repo.end_call(call_id).await?;
        }

        Ok(())
    }

    async fn list_call_participants(&self, call_id: i32) -> Result<Vec<CallParticipant>, AppError> {
        self.call_repo
            .get_call_by_id(call_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Call {} not found", call_id)))?;

        self.call_repo.list_call_participants(call_id).await
    }

    async fn list_active_participants(
        &self,
        call_id: i32,
    ) -> Result<Vec<CallParticipant>, AppError> {
        self.call_repo
            .get_call_by_id(call_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Call {} not found", call_id)))?;

        self.call_repo.list_active_participants(call_id).await
    }

    async fn count_active_participants(&self, call_id: i32) -> Result<i64, AppError> {
        self.call_repo
            .get_call_by_id(call_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Call {} not found", call_id)))?;

        self.call_repo.count_active_participants(call_id).await
    }
}
