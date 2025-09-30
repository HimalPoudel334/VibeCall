use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    rooms::{
        entities::{Room, RoomMemberRole, RoomType},
        repository::RoomRepository,
    },
    shared::response::AppError,
    users::User,
};

#[async_trait]
pub trait RoomService: Send + Sync {
    async fn create_room(
        &self,
        name: String,
        room_type: String,
        created_by: i32,
        description: Option<String>,
    ) -> Result<Room, AppError>;

    async fn get_room(&self, room_id: &str) -> Result<Option<Room>, AppError>;

    async fn list_rooms(&self, limit: i64, offset: i64) -> Result<Vec<Room>, AppError>;

    async fn delete_room(&self, room_id: &str, user_id: i32) -> Result<(), AppError>;

    async fn join_room(&self, room_id: &str, user_id: i32) -> Result<(), AppError>;

    async fn leave_room(&self, room_id: &str, user_id: i32) -> Result<(), AppError>;

    async fn list_room_users(&self, room_id: &str) -> Result<Vec<User>, AppError>;

    async fn is_user_in_room(&self, room_id: &str, user_id: i32) -> Result<bool, AppError>;

    async fn is_user_owner(&self, room_id: &str, user_id: i32) -> Result<bool, AppError>;

    async fn join_room_with_role(
        &self,
        room_id: &str,
        user_id: i32,
        role: RoomMemberRole,
    ) -> Result<(), AppError>;
}

pub struct RoomServiceImpl {
    repo: Arc<dyn RoomRepository + Send + Sync>,
}

impl RoomServiceImpl {
    pub fn new(repo: Arc<dyn RoomRepository + Send + Sync>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl RoomService for RoomServiceImpl {
    async fn create_room(
        &self,
        name: String,
        room_type: String,
        created_by: i32,
        description: Option<String>,
    ) -> Result<Room, AppError> {
        if name.trim().is_empty() {
            return Err(AppError::Validation("Room name cannot be empty".into()));
        }

        let room_type = room_type.parse::<RoomType>()?;

        let max_participants = match room_type {
            RoomType::OneOnOne => 2,
            RoomType::Private | RoomType::Instant => 10,
            RoomType::Group | RoomType::Meeting => 50,
            RoomType::Public => 100,
        };

        let mut room = self
            .repo
            .create(name, room_type, created_by, description)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        room.max_participants = max_participants;

        self.join_room_with_role(&room.id, created_by, RoomMemberRole::Owner)
            .await?;

        Ok(room)
    }

    async fn get_room(&self, room_id: &str) -> Result<Option<Room>, AppError> {
        self.repo
            .get_by_id(room_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn list_rooms(&self, limit: i64, offset: i64) -> Result<Vec<Room>, AppError> {
        if limit <= 0 || offset < 0 {
            return Err(AppError::Validation("Invalid limit or offset".into()));
        }
        self.repo
            .list_rooms(limit, offset)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn delete_room(&self, room_id: &str, user_id: i32) -> Result<(), AppError> {
        let _room = self
            .get_room(room_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Room {} not found", room_id)))?;

        if !self.is_user_owner(room_id, user_id).await? {
            return Err(AppError::Unauthorized(
                "Only room owner can delete the room".into(),
            ));
        }

        self.repo
            .delete(room_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    async fn join_room(&self, room_id: &str, user_id: i32) -> Result<(), AppError> {
        self.join_room_with_role(room_id, user_id, RoomMemberRole::Participant)
            .await
    }

    async fn leave_room(&self, room_id: &str, user_id: i32) -> Result<(), AppError> {
        if !self.is_user_in_room(room_id, user_id).await? {
            return Err(AppError::NotFound(format!(
                "User {} not in room {}",
                user_id, room_id
            )));
        }

        self.repo
            .leave_room(room_id, user_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let room = self
            .get_room(room_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Room {} not found", room_id)))?;
        if room.room_type == RoomType::Instant {
            let count = self
                .repo
                .count_active_members(room_id)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
            if count == 0 {
                self.repo
                    .delete(room_id)
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }
        }

        Ok(())
    }

    async fn list_room_users(&self, room_id: &str) -> Result<Vec<User>, AppError> {
        self.repo
            .list_room_users(room_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn is_user_in_room(&self, room_id: &str, user_id: i32) -> Result<bool, AppError> {
        self.repo
            .is_user_in_room(room_id, user_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn is_user_owner(&self, room_id: &str, user_id: i32) -> Result<bool, AppError> {
        self.repo
            .is_user_owner(room_id, user_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn join_room_with_role(
        &self,
        room_id: &str,
        user_id: i32,
        role: RoomMemberRole,
    ) -> Result<(), AppError> {
        let room = self
            .get_room(room_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Room {} not found", room_id)))?;

        if self.is_user_in_room(room_id, user_id).await? {
            return Err(AppError::Validation(format!(
                "User {} already in room {}",
                user_id, room_id
            )));
        }

        let count = self
            .repo
            .count_active_members(room_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        if count >= room.max_participants as i64 {
            return Err(AppError::Validation(format!(
                "Room {} is full (max {})",
                room_id, room.max_participants
            )));
        }

        if room.room_type == RoomType::OneOnOne && count >= 2 {
            return Err(AppError::Validation(
                "OneOnOne room limited to 2 participants".into(),
            ));
        }
        if room.room_type == RoomType::Private && role != RoomMemberRole::Owner {
            return Err(AppError::Unauthorized(
                "Private rooms require an invitation".into(),
            ));
        }

        self.repo
            .join_room(room_id, user_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if role != RoomMemberRole::Participant {
            self.repo
                .update_member_role(room_id, user_id, role)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
        }

        Ok(())
    }
}
