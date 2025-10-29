use crate::{
    shared::{
        base_types::{email::Email, phone_number::PhoneNumber},
        file_service::FileService,
        response::AppError,
        utils,
    },
    users::{entities::User, repository::UserRepository},
};
use async_trait::async_trait;
use std::{path::Path, sync::Arc};

#[async_trait]
pub trait UserService: Send + Sync {
    async fn get_by_id(&self, id: i32) -> Result<Option<User>, AppError>;

    async fn create(
        &self,
        first_name: String,
        last_name: String,
        email: String,
        phone: String,
        password: String,
        confirm_password: String,
    ) -> Result<User, AppError>;

    async fn update_avatar(&self, user_id: i32, avatar_url: &str) -> Result<User, AppError>;

    async fn upload_avatar(
        &self,
        user_id: i32,
        uploaded_path: &Path,
        original_file_name: Option<String>,
        file_service: Arc<dyn FileService>,
    ) -> Result<User, AppError>;

    async fn authenticate(&self, email: &str, password: &str) -> Result<User, AppError>;
}

pub struct UserServiceImpl {
    repository: Arc<dyn UserRepository + Send + Sync>,
}

impl UserServiceImpl {
    pub fn new(repository: Arc<dyn UserRepository + Send + Sync>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn get_by_id(&self, id: i32) -> Result<Option<User>, AppError> {
        if id <= 0 {
            return Err(AppError::Validation("User ID must be positive".into()));
        }
        self.repository.get_by_id(id).await
    }

    async fn create(
        &self,
        first_name: String,
        last_name: String,
        email: String,
        phone: String,
        password: String,
        confirm_password: String,
    ) -> Result<User, AppError> {
        if first_name.is_empty()
            || last_name.is_empty()
            || email.is_empty()
            || password.is_empty()
            || phone.is_empty()
            || confirm_password.is_empty()
        {
            return Err(AppError::Validation("All fields are mandatory".into()));
        }

        if password != confirm_password {
            return Err(AppError::Validation("Passwords do not match".into()));
        }

        if self.repository.get_by_email(&email).await?.is_some() {
            return Err(AppError::Validation("Email already in use".into()));
        }

        if self.repository.get_by_phone(&phone).await?.is_some() {
            return Err(AppError::Validation("Phone number already in use".into()));
        }

        let hash_password_result = utils::hash_password(&password);
        let hashed_password = match hash_password_result {
            Ok(hash) => hash,
            Err(_) => {
                return Err(AppError::InternalServerError(
                    "Failed to hash password".into(),
                ));
            }
        };

        let new_user =
            super::entities::NewUser::new(first_name, last_name, email, phone, hashed_password);

        self.repository.create(new_user).await
    }

    async fn update_avatar(&self, user_id: i32, avatar_url: &str) -> Result<User, AppError> {
        if user_id <= 0 {
            return Err(AppError::Validation("User ID must be positive".into()));
        }

        if avatar_url.is_empty() {
            return Err(AppError::Validation("Avatar URL cannot be empty".into()));
        }

        let user = self.repository.update_avatar(user_id, avatar_url).await?;

        Ok(user)
    }

    async fn upload_avatar(
        &self,
        user_id: i32,
        uploaded_path: &Path,
        original_file_name: Option<String>,
        file_service: Arc<dyn FileService>,
    ) -> Result<User, AppError> {
        let extension = original_file_name
            .as_ref()
            .and_then(|name| Path::new(name).extension())
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .filter(|ext| ["jpg", "jpeg", "png", "gif"].contains(&ext.as_str()))
            .ok_or_else(|| AppError::BadRequest("Invalid or missing file extension".into()))?;

        let file_bytes = tokio::fs::read(uploaded_path)
            .await
            .map_err(|e| AppError::InternalServerError(format!("File read error: {}", e)))?;

        let filename = file_service
            .save_avatar(file_bytes, &extension)
            .await
            .map_err(|e| AppError::InternalServerError(format!("File save error: {}", e)))?;

        match self.update_avatar(user_id, &filename).await {
            Ok(user) => Ok(user),
            Err(e) => {
                let _ = file_service.delete_avatar(&filename).await;
                Err(e)
            }
        }
    }

    async fn authenticate(&self, username: &str, password: &str) -> Result<User, AppError> {
        if username.is_empty() || password.is_empty() {
            return Err(AppError::Validation(
                "Username and password is required".into(),
            ));
        }

        let user = if let Ok(email) = Email::try_from(username) {
            self.repository.get_by_email(email.get_email()).await?
        } else if let Ok(phone) = PhoneNumber::try_from(username) {
            self.repository.get_by_phone(phone.get_number()).await?
        } else {
            return Err(AppError::Unauthorized(
                "Invalid Username or Password2".to_string(),
            ));
        };

        if user.is_none() {
            return Err(AppError::Unauthorized(
                "Invalid Username or Password3".to_string(),
            ));
        }

        let user = user.unwrap();

        if !utils::verify_password_hash(&user.password, password) {
            return Err(AppError::Unauthorized(
                "Invalid Username or Password4".to_string(),
            ));
        }

        let user = User {
            id: user.id,
            first_name: user.first_name,
            last_name: user.last_name,
            email: user.email,
            phone: user.phone,
            avatar_url: user.avatar_url,
            created_at: user.created_at,
            last_seen: user.last_seen,
        };

        Ok(user)
    }
}
