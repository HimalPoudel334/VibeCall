use crate::users::{self, entities::User};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository {
    async fn get_by_id(
        &self,
        id: i32,
    ) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>>;
    async fn create(
        &self,
        user: users::entities::NewUser,
    ) -> Result<User, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_by_email(
        &self,
        email: &str,
    ) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_by_phone(
        &self,
        phone: &str,
    ) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>>;
    async fn update_avatar(
        &self,
        user_id: i32,
        avatar_url: &str,
    ) -> Result<User, Box<dyn std::error::Error + Send + Sync>>;
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
    async fn get_by_id(
        &self,
        id: i32,
    ) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
        let user = sqlx::query_as::<_, User>("SELECT id, name, created_at FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    async fn create(
        &self,
        user: users::entities::NewUser,
    ) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query_as::<_, User>(
            "INSERT INTO users (first_name, last_name, email, phone, password) VALUES (?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(user.first_name)
        .bind(user.last_name)
        .bind(user.email)
        .bind(user.phone)
        .bind(user.password)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn get_by_email(
        &self,
        email: &str,
    ) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
        let user = sqlx::query_as::<_, User>("SELECT id, first_name, last_name, email, phone, password, created_at, last_seen FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    async fn get_by_phone(
        &self,
        phone: &str,
    ) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
        let user = sqlx::query_as::<_, User>("SELECT id, first_name, last_name, email, phone, password, created_at, last_seen FROM users WHERE phone = ?")
            .bind(phone)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    async fn update_avatar(
        &self,
        user_id: i32,
        avatar_url: &str,
    ) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
        let user =
            sqlx::query_as::<_, User>("UPDATE users SET avatar_url = ? WHERE id = ? RETURNING id")
                .bind(avatar_url)
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?;

        Ok(user)
    }
}
