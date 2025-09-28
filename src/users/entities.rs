use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub avatar_url: String,
    pub password: String,
    pub created_at: chrono::NaiveDateTime,
    pub last_seen: chrono::NaiveDateTime,
}

pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub avatar_url: String,
    pub password: String,
}

impl NewUser {
    pub fn new(
        first_name: String,
        last_name: String,
        email: String,
        phone: String,
        password: String,
    ) -> Self {
        Self {
            first_name,
            last_name,
            email,
            phone,
            avatar_url: String::from("default_user.png"),
            password,
        }
    }
}
