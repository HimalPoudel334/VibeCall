use serde::{Deserialize, Serialize};

use crate::users::User;

#[derive(Deserialize)]
pub struct NewCall {
    pub room_id: String,
    pub caller_id: i32,
    pub status: String,
}

#[derive(Deserialize)]
pub struct UpdateCallStatus {
    pub status: String,
}

#[derive(Deserialize)]
pub struct UserIdParam {
    pub user_id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub avatar_url: String,
}

impl From<User> for PublicUser {
    fn from(u: User) -> Self {
        PublicUser {
            id: u.id,
            first_name: u.first_name,
            last_name: u.last_name,
            avatar_url: u.avatar_url,
        }
    }
}
