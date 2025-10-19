use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct TurnCredentials {
    pub username: String,
    pub credential: String,
    pub urls: Vec<String>,
}

#[derive(Deserialize)]
pub struct RoomIdQueryParam {
    pub room_id: String,
}
