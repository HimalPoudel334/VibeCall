use serde::Deserialize;

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
