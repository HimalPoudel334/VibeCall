use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, sqlite::SqliteRow};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub avatar_url: String,
    pub created_at: chrono::NaiveDateTime,
    pub last_seen: chrono::NaiveDateTime,
}

impl<'r> FromRow<'r, SqliteRow> for User {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let base_url =
            std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8085".to_string());

        let avatar_filename: String = row.try_get("avatar_url")?;

        let avatar_url = if avatar_filename.starts_with("http") {
            avatar_filename
        } else {
            format!(
                "{}/media/images/avatars/{}",
                base_url.trim_end_matches('/'),
                avatar_filename
            )
        };

        Ok(User {
            id: row.try_get("id")?,
            first_name: row.try_get("first_name")?,
            last_name: row.try_get("last_name")?,
            email: row.try_get("email")?,
            phone: row.try_get("phone")?,
            avatar_url,
            created_at: row.try_get("created_at")?,
            last_seen: row.try_get("last_seen")?,
        })
    }
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
            avatar_url: String::from("user_default.png"),
            password,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct UserWithPassword {
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
