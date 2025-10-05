use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct TurnCredentials {
    pub username: String,
    pub credential: String,
    pub urls: Vec<String>,
}

#[derive(Deserialize)]
pub struct UserIdQuery {
    pub user_id: i32,
}
