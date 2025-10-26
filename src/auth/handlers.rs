use std::sync::Arc;

use actix_identity::Identity;
use actix_session::Session;
use actix_web::{
    HttpMessage, HttpResponse, get,
    http::header::{self, ContentType},
    post, web,
};
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
    req: actix_web::HttpRequest,
    form: web::Form<LoginRequest>,
    session: Session,
    user_service: web::Data<Arc<dyn UserService>>,
) -> actix_web::Result<HttpResponse> {
    let user = match user_service
        .authenticate(&form.username, &form.password)
        .await
    {
        Ok(user) => user,
        Err(err) => {
            let mut context = Context::new();
            context.insert("title", "Login");
            context.insert("error", &err.to_string());
            context.insert("username", &form.username);

            let rendered = TEMPLATES.render("login.html", &context).unwrap();

            return Ok(HttpResponse::Ok()
                .content_type(ContentType::html())
                .body(rendered));
        }
    };

    Identity::login(&req.extensions(), user.id.to_string())?;

    let redirect_path = session
        .get::<String>("redirect_after_login")?
        .unwrap_or_else(|| "/".to_string());

    session.remove("redirect_after_login");

    let safe_redirect = if redirect_path.starts_with('/') && !redirect_path.starts_with("//") {
        redirect_path
    } else {
        "/".to_string()
    };

    Ok(HttpResponse::Found()
        .append_header((header::LOCATION, safe_redirect))
        .finish())
}
