use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    rooms::entities::{Room, RoomMemberRole, RoomType},
    shared::response::AppError,
    users::User,
};

#[async_trait]
pub trait RoomRepository {
    async fn create(
        &self,
        name: String,
        room_type: RoomType,
        created_by: i32,
        description: Option<String>,
    ) -> Result<Room, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_by_id(
        &self,
        room_id: &str,
    ) -> Result<Option<Room>, Box<dyn std::error::Error + Send + Sync>>;

    async fn list_rooms(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Room>, Box<dyn std::error::Error + Send + Sync>>;

    async fn delete(&self, room_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn join_room(
        &self,
        room_id: &str,
        user_id: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn leave_room(
        &self,
        room_id: &str,
        user_id: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn list_room_users(
        &self,
        room_id: &str,
    ) -> Result<Vec<User>, Box<dyn std::error::Error + Send + Sync>>;

    async fn is_user_in_room(
        &self,
        room_id: &str,
        user_id: i32,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;

    async fn is_user_owner(
        &self,
        room_id: &str,
        user_id: i32,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;

    async fn count_active_members(
        &self,
        room_id: &str,
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>>;

    async fn update_member_role(
        &self,
        room_id: &str,
        user_id: i32,
        role: RoomMemberRole,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub struct SqliteRoomRepository {
    pool: SqlitePool,
}

impl SqliteRoomRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoomRepository for SqliteRoomRepository {
    async fn create(
        &self,
        name: String,
        room_type: RoomType,
        created_by: i32,
        description: Option<String>,
    ) -> Result<Room, Box<dyn std::error::Error + Send + Sync>> {
        let room_id = Uuid::new_v4().to_string();
        let room = sqlx::query_as::<_, Room>(
            r#"
                INSERT INTO rooms (id, name, room_type, created_by, description, max_participants, is_active)
                VALUES ($1, $2, $3, $4, $5, 10, TRUE)
                RETURNING *
            "#,
        )
        .bind(&room_id)
        .bind(&name)
        .bind(room_type.to_string())
        .bind(created_by)
        .bind(description)
        .fetch_one(&self.pool)
        .await?;

        Ok(room)
    }

    async fn get_by_id(
        &self,
        room_id: &str,
    ) -> Result<Option<Room>, Box<dyn std::error::Error + Send + Sync>> {
        let room = sqlx::query_as::<_, Room>("SELECT * FROM rooms WHERE id = $1")
            .bind(room_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(room)
    }

    async fn list_rooms(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Room>, Box<dyn std::error::Error + Send + Sync>> {
        let rooms = sqlx::query_as::<_, Room>(
            r#"
                SELECT 
                    *
                FROM rooms 
                ORDER BY created_at DESC 
                LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rooms)
    }

    async fn delete(&self, room_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query("DELETE FROM rooms WHERE id = $1")
            .bind(room_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn join_room(
        &self,
        room_id: &str,
        user_id: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO room_members (room_id, user_id)
            VALUES ($1, $2)
            "#,
        )
        .bind(room_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn leave_room(
        &self,
        room_id: &str,
        user_id: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let updated = sqlx::query(
            r#"
            UPDATE room_members
            SET left_at = CURRENT_TIMESTAMP
            WHERE room_id = $1 AND user_id = $2 AND left_at IS NULL
            "#,
        )
        .bind(room_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        if updated.rows_affected() == 0 {
            return Err(Box::new(AppError::NotFound(format!(
                "User {} not in room {}",
                user_id, room_id
            ))));
        }
        Ok(())
    }

    async fn list_room_users(
        &self,
        room_id: &str,
    ) -> Result<Vec<User>, Box<dyn std::error::Error + Send + Sync>> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT u.*
            FROM users u
            JOIN room_members rm ON u.id = rm.user_id
            WHERE rm.room_id = $1 AND rm.left_at IS NULL
            "#,
        )
        .bind(room_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }

    async fn is_user_in_room(
        &self,
        room_id: &str,
        user_id: i32,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let exists = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM room_members
                WHERE room_id = $1 AND user_id = $2 AND left_at IS NULL
            )
            "#,
        )
        .bind(room_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    async fn is_user_owner(
        &self,
        room_id: &str,
        user_id: i32,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let is_owner = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM room_members
                WHERE room_id = $1 AND user_id = $2 AND role = 'owner' AND left_at IS NULL
            )
            "#,
        )
        .bind(room_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(is_owner)
    }

    async fn count_active_members(
        &self,
        room_id: &str,
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let count = sqlx::query_scalar(
            "SELECT COUNT(*) FROM room_members WHERE room_id = $1 AND left_at IS NULL",
        )
        .bind(room_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    async fn update_member_role(
        &self,
        room_id: &str,
        user_id: i32,
        role: RoomMemberRole,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let updated = sqlx::query(
            r#"
            UPDATE room_members SET role = $1
            WHERE room_id = $2 AND user_id = $3
            "#,
        )
        .bind(role)
        .bind(room_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        if updated.rows_affected() == 0 {
            return Err(Box::new(AppError::NotFound(format!(
                "User {} not in room {}",
                user_id, room_id
            ))));
        }
        Ok(())
    }
}
