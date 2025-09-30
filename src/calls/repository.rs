use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::{
    calls::entities::{Call, CallParticipant, CallStatus},
    shared::response::AppError,
};

#[async_trait]
pub trait CallRepository {
    async fn create_call(
        &self,
        room_id: String,
        caller_id: i32,
        status: CallStatus,
    ) -> Result<Call, AppError>;

    async fn get_call_by_id(&self, call_id: i32) -> Result<Option<Call>, AppError>;

    async fn update_call_status(&self, call_id: i32, status: CallStatus) -> Result<(), AppError>;

    async fn end_call(&self, call_id: i32) -> Result<(), AppError>;

    // Room-based queries
    async fn get_calls_by_room_id(&self, room_id: &str) -> Result<Vec<Call>, AppError>;

    // User-based queries
    async fn get_calls_by_user_id(&self, user_id: i32) -> Result<Vec<Call>, AppError>;

    async fn get_user_participated_calls(&self, user_id: i32) -> Result<Vec<Call>, AppError>;

    // Active calls
    async fn get_active_calls(&self) -> Result<Vec<Call>, AppError>;

    // Participant management
    async fn list_call_participants(&self, call_id: i32) -> Result<Vec<CallParticipant>, AppError>;

    async fn add_call_participant(&self, call_id: i32, user_id: i32) -> Result<(), AppError>;

    async fn remove_call_participant(&self, call_id: i32, user_id: i32) -> Result<(), AppError>;

    async fn list_active_participants(
        &self,
        call_id: i32,
    ) -> Result<Vec<CallParticipant>, AppError>;

    async fn count_active_participants(&self, call_id: i32) -> Result<i64, AppError>;
}

pub struct SqliteCallRepository {
    pool: SqlitePool,
}

impl SqliteCallRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CallRepository for SqliteCallRepository {
    async fn create_call(
        &self,
        room_id: String,
        caller_id: i32,
        status: CallStatus,
    ) -> Result<Call, AppError> {
        let call = sqlx::query_as::<_, Call>(
            r#"
            INSERT INTO calls (room_id, caller_id, status)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(room_id)
        .bind(caller_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;

        Ok(call)
    }

    async fn get_call_by_id(&self, call_id: i32) -> Result<Option<Call>, AppError> {
        let call = sqlx::query_as::<_, Call>("SELECT * FROM calls WHERE id = $1")
            .bind(call_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(call)
    }

    async fn update_call_status(&self, call_id: i32, status: CallStatus) -> Result<(), AppError> {
        sqlx::query("UPDATE calls SET status = $1 WHERE id = $2")
            .bind(status)
            .bind(call_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn end_call(&self, call_id: i32) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE calls 
            SET status = 'ended', 
                ended_at = CURRENT_TIMESTAMP,
                duration = CAST((julianday(CURRENT_TIMESTAMP) - julianday(started_at)) * 86400 AS INTEGER)
            WHERE id = $1
            "#
        )
        .bind(call_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_calls_by_room_id(&self, room_id: &str) -> Result<Vec<Call>, AppError> {
        let calls = sqlx::query_as::<_, Call>(
            "SELECT * FROM calls WHERE room_id = $1 ORDER BY started_at DESC",
        )
        .bind(room_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(calls)
    }

    async fn get_calls_by_user_id(&self, user_id: i32) -> Result<Vec<Call>, AppError> {
        let calls = sqlx::query_as::<_, Call>(
            "SELECT * FROM calls WHERE caller_id = $1 ORDER BY started_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(calls)
    }

    async fn get_user_participated_calls(&self, user_id: i32) -> Result<Vec<Call>, AppError> {
        let calls = sqlx::query_as::<_, Call>(
            r#"
            SELECT DISTINCT c.*
            FROM calls c
            JOIN call_participants cp ON c.id = cp.call_id
            WHERE cp.user_id = $1
            ORDER BY c.started_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(calls)
    }

    async fn get_active_calls(&self) -> Result<Vec<Call>, AppError> {
        let calls = sqlx::query_as::<_, Call>(
            "SELECT * FROM calls WHERE status = 'active' ORDER BY started_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(calls)
    }

    async fn list_call_participants(&self, call_id: i32) -> Result<Vec<CallParticipant>, AppError> {
        let participants = sqlx::query_as::<_, CallParticipant>(
            "SELECT * FROM call_participants WHERE call_id = $1 ORDER BY joined_at",
        )
        .bind(call_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(participants)
    }

    async fn add_call_participant(&self, call_id: i32, user_id: i32) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO call_participants (call_id, user_id)
            VALUES ($1, $2)
            ON CONFLICT (call_id, user_id) DO NOTHING
            "#,
        )
        .bind(call_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_call_participant(&self, call_id: i32, user_id: i32) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE call_participants 
            SET left_at = CURRENT_TIMESTAMP,
                duration = CAST((julianday(CURRENT_TIMESTAMP) - julianday(joined_at)) * 86400 AS INTEGER)
            WHERE call_id = $1 AND user_id = $2 AND left_at IS NULL
            "#
        )
        .bind(call_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_active_participants(
        &self,
        call_id: i32,
    ) -> Result<Vec<CallParticipant>, AppError> {
        let participants = sqlx::query_as::<_, CallParticipant>(
            "SELECT * FROM call_participants WHERE call_id = $1 AND left_at IS NULL",
        )
        .bind(call_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(participants)
    }

    async fn count_active_participants(&self, call_id: i32) -> Result<i64, AppError> {
        let count = sqlx::query_scalar(
            "SELECT COUNT(*) as count FROM call_participants WHERE call_id = $1 AND left_at IS NULL",
        )
        .bind(call_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }
}
