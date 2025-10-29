use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use serde::Deserialize;

use crate::shared::base_types::{email::Email, phone_number::PhoneNumber};

#[derive(Deserialize)]
pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub email: Email,
    pub phone: PhoneNumber,
    pub password: String,
    pub confirm_password: String,
}

#[derive(MultipartForm)]
pub struct AvatarUpload {
    pub user_id: Text<i32>,
    #[multipart(limit = "1MB")]
    pub avatar: TempFile,
}
