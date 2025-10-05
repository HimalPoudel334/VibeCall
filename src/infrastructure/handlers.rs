use actix_files::NamedFile;
use actix_web::{HttpResponse, Responder, get, web};
use base64::{Engine as _, engine::general_purpose};
use hmac::{Hmac, Mac};
use serde_json::json;
use sha1::Sha1;

use crate::infrastructure::contract::{TurnCredentials, UserIdQuery};

#[get("/health")]
pub async fn health_check() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "VibeCall backend",
    })))
}

#[get("/")]
pub async fn index() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("templates/index.html")?)
}

#[get("/turn-credentials")]
pub async fn get_turn_credentials(user_id: web::Query<UserIdQuery>) -> impl Responder {
    let shared_secret = "MyVerySecretKey12345";
    let ttl = 86400;

    let timestamp = chrono::Utc::now().timestamp() + ttl;
    let username = format!("{}:user{}", timestamp, user_id.user_id);

    let mut mac = Hmac::<Sha1>::new_from_slice(shared_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(username.as_bytes());
    let result = mac.finalize();
    let credential = general_purpose::STANDARD.encode(result.into_bytes());

    HttpResponse::Ok().json(TurnCredentials {
        username,
        credential,
        urls: vec![
            "stun:159.13.60.202:3478".to_string(),
            "turn:159.13.60.202:3478".to_string(),
        ],
    })
}
