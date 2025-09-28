use actix_web::{HttpResponse, get};
use serde_json::json;

#[get("/health")]
pub async fn health_check() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "VibeCall backend",
    })))
}

