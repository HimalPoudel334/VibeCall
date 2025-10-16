use std::sync::Arc;

use actix_identity::Identity;
use actix_web::{
    HttpResponse, get,
    http::header::{self, ContentType},
    web,
};
use base64::{Engine as _, engine::general_purpose};
use hmac::{Hmac, Mac};
use serde_json::json;
use sha1::Sha1;
use tera::Context;

use crate::{
    infrastructure::{contract::TurnCredentials, templates::TEMPLATES},
    shared::response::{AppError, respond_ok},
    users::UserService,
};

#[get("/health")]
pub async fn health_check() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "VibeCall backend",
    })))
}

#[get("/")]
pub async fn index(
    identity: Option<Identity>,
    user_service: web::Data<Arc<dyn UserService>>,
) -> actix_web::Result<HttpResponse> {
    let user_id: i32 = match identity
        .and_then(|id| id.id().ok())
        .and_then(|id_str| id_str.parse::<i32>().ok())
    {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Found()
                .append_header((header::LOCATION, "/auth/login"))
                .finish());
        }
    };

    let user = user_service
        .get_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    let mut context = Context::new();
    context.insert("title", "Home Page");
    context.insert("user", &user);

    let rendered = TEMPLATES
        .render("index.html", &context)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered))
}

#[get("/turn-credentials")]
pub async fn get_turn_credentials(identity: Option<Identity>) -> actix_web::Result<HttpResponse> {
    let user_id: i32 = match identity
        .and_then(|id| id.id().ok())
        .and_then(|id_str| id_str.parse::<i32>().ok())
    {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Found()
                .append_header((header::LOCATION, "/auth/login"))
                .finish());
        }
    };

    let shared_secret = "MyVerySecretKey12345";
    let ttl = 86400;

    let timestamp = chrono::Utc::now().timestamp() + ttl;
    let username = format!("{}:user{}", timestamp, user_id);

    let mut mac = Hmac::<Sha1>::new_from_slice(shared_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(username.as_bytes());
    let result = mac.finalize();
    let credential = general_purpose::STANDARD.encode(result.into_bytes());

    TurnCredentials {
        username,
        credential,
        urls: vec![
            "stun:159.13.60.202:3478".to_string(),
            "turn:159.13.60.202:3478".to_string(),
        ],
    };

    let mut context = Context::new();
    context.insert("title", "Home Page");

    let rendered = TEMPLATES
        .render("videos.html", &context)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered))
}
