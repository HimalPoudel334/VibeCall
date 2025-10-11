use std::sync::Arc;

use actix_web::{HttpResponse, get, http::header::ContentType, post, web};
use tera::Context;

use crate::{
    auth::contract::LoginRequest, infrastructure::templates::TEMPLATES, shared::response::AppError,
    users::UserService,
};

#[get("/login")]
pub async fn login() -> actix_web::Result<HttpResponse> {
    let mut context = Context::new();
    context.insert("title", "Login");

    let rendered = TEMPLATES
        .render("login.html", &context)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered))
}

#[get("/register")]
pub async fn register() -> actix_web::Result<HttpResponse> {
    let mut context = Context::new();
    context.insert("title", "Register");

    let rendered = TEMPLATES
        .render("register.html", &context)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered))
}

#[post("/login")]
pub async fn login_post(
    data: web::Form<LoginRequest>,
    user_service: web::Data<Arc<dyn UserService>>,
) -> actix_web::Result<HttpResponse> {
    let user = user_service
        .authenticate(&data.username, &data.password)
        .await?;

    let mut context = Context::new();
    context.insert("user", &user);

    let rendered = TEMPLATES
        .render("index.html", &context)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered))
}
