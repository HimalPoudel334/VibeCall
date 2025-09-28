use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::users::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub users: HashMap<Uuid, User>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct RoomInfo {
    pub id: String,
    pub user_count: usize,
    pub created_at: chrono::NaiveDateTime,
}
