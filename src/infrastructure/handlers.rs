use std::sync::Arc;

use actix_identity::Identity;
use actix_web::{HttpResponse, get, http::header::ContentType, web};
use base64::{Engine as _, engine::general_purpose};
use hmac::{Hmac, Mac};
use serde_json::json;
use sha1::Sha1;
use tera::Context;

use crate::{
    infrastructure::{contract::RoomIdQueryParam, templates::TEMPLATES},
    shared::response::AppError,
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
    identity: Identity,
    user_service: web::Data<Arc<dyn UserService>>,
) -> actix_web::Result<HttpResponse> {
    let user_id: i32 = identity
        .id()?
        .parse::<i32>()
        .map_err(|_| AppError::BadRequest("Invalid User Id".to_string()))?;

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
pub async fn get_turn_credentials(
    identity: Identity,
    room_id: web::Query<RoomIdQueryParam>,
    user_service: web::Data<Arc<dyn UserService>>,
) -> actix_web::Result<HttpResponse> {
    let user_id: i32 = identity
        .id()?
        .parse::<i32>()
        .map_err(|_| AppError::BadRequest("Invalid User Id".to_string()))?;

    let shared_secret = "MyVerySecretKey12345";
    let ttl = 86400;

    let timestamp = chrono::Utc::now().timestamp() + ttl;
    let username = format!("{}:user{}", timestamp, user_id);

    let mut mac = Hmac::<Sha1>::new_from_slice(shared_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(username.as_bytes());
    let result = mac.finalize();
    let credential = general_purpose::STANDARD.encode(result.into_bytes());

    let ice_servers = json!([
        { "urls": "stun:stun.l.google.com:19302" },
        {
            "urls": vec![
                "stun:159.13.60.202:3478".to_string(),
                "turn:159.13.60.202:3478".to_string()
            ],
            "username": username,
            "credential": credential
        }
    ]);

    let user_name = user_service
        .get_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))
        .map(|user| format!("{} {}", user.first_name, user.last_name))?;

    let mut context = Context::new();
    context.insert("title", "Home Page");
    context.insert("user_id", &user_id);
    context.insert("user_name", &user_name);
    context.insert("room_id", &room_id.room_id);
    context.insert("ice_servers", &ice_servers.to_string());

    let rendered = TEMPLATES
        .render("videos.html", &context)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered))
}
