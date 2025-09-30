use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewRoom {
    pub name: String,
    pub created_by: i32,
    pub description: Option<String>,
    pub room_type: String,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct UserIdParam {
    pub user_id: i32,
}
