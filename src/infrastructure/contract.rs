use serde::Serialize;

#[derive(Serialize)]
pub struct TurnCredentials {
    pub username: String,
    pub credential: String,
    pub urls: Vec<String>,
}
