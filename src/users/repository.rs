use crate::{
    shared::response::AppError,
    users::{self, entities::User},
};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository {
    async fn get_by_id(&self, id: i32) -> Result<Option<User>, AppError>;
    async fn create(&self, user: users::entities::NewUser) -> Result<User, AppError>;
    async fn get_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn get_by_phone(&self, phone: &str) -> Result<Option<User>, AppError>;
    async fn update_avatar(&self, user_id: i32, avatar_url: &str) -> Result<User, AppError>;
}

// Concrete implementation
pub struct SqliteUserRepository {
    pool: sqlx::SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn get_by_id(&self, id: i32) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT 
                id, 
                first_name,
                last_name, 
                email, 
                phone, 
                last_seen, 
                avatar_url, 
                created_at 
            FROM users 
            WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn create(&self, user: users::entities::NewUser) -> Result<User, AppError> {
        let created_user = sqlx::query_as::<_, User>(
            r#"
                INSERT INTO users (
                    first_name, last_name, email,
                    phone, avatar_url, password
                )
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING
                    id, first_name, last_name, email, phone, created_at, last_seen, avatar_url
                "#,
        )
        .bind(user.first_name)
        .bind(user.last_name)
        .bind(user.email)
        .bind(user.phone)
        .bind(user.avatar_url)
        .bind(user.password)
        .fetch_one(&self.pool)
        .await?;

        Ok(created_user)
    }

    async fn get_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT 
                id, 
                first_name, 
                last_name, 
                email, 
                phone,
                created_at, 
                last_seen 
            FROM users 
            WHERE email = $1"#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn get_by_phone(&self, phone: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT 
                id, 
                first_name, 
                last_name, 
                email, 
                phone,
                created_at, 
                last_seen 
            FROM users 
            WHERE phone = $1"#,
        )
        .bind(phone)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update_avatar(&self, user_id: i32, avatar_url: &str) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE 
                users 
            SET 
                avatar_url = %1 
            WHERE id = $2
            RETURNING    
                id, first_name, last_name, email, phone, created_at, last_seen, avatar_url
            "#,
        )
        .bind(avatar_url)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }
}
