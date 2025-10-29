use std::sync::Arc;

use actix_multipart::form::MultipartForm;
use actix_web::{
    HttpResponse, Result as ActixResult, get,
    http::header::{self, ContentType},
    post, web,
};
use tera::Context;

use crate::{
    infrastructure::templates::TEMPLATES,
    shared::{
        file_service::FileService,
        response::{AppError, respond_ok},
    },
    users::{
        contract::{AvatarUpload, NewUser},
        service::UserService,
    },
};

#[get("/{id}")]
pub async fn get_user(
    path: web::Path<i32>,
    user_service: web::Data<Arc<dyn UserService>>,
) -> ActixResult<HttpResponse> {
    let user_id = path.into_inner();

    let user = user_service
        .get_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", user_id)))?;

    respond_ok(user)
}

#[get("")]
pub async fn get_current_user(
    identity: actix_identity::Identity,
    user_service: web::Data<Arc<dyn UserService>>,
) -> ActixResult<HttpResponse> {
    let user_id: i32 = identity
        .id()?
        .parse::<i32>()
        .map_err(|_| AppError::BadRequest("Invalid User Id".to_string()))?;

    let user = user_service
        .get_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", user_id)))?;

    respond_ok(user)
}

#[get("/register")]
pub async fn create_user_get() -> ActixResult<HttpResponse> {
    let context = Context::new();
    let rendered = TEMPLATES
        .render("register.html", &context)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered))
}

#[post("")]
pub async fn create_user(
    user_json: web::Json<NewUser>,
    user_service: web::Data<Arc<dyn UserService>>,
) -> ActixResult<HttpResponse> {
    let _ = user_service
        .create(
            user_json.first_name.clone(),
            user_json.last_name.clone(),
            user_json.email.get_email().to_string(),
            user_json.phone.get_number().to_string(),
            user_json.password.clone(),
            user_json.confirm_password.clone(),
        )
        .await?;

    Ok(HttpResponse::Found()
        .append_header((header::LOCATION, "/vibecall/auth/login"))
        .finish())
}

#[post("/{id}/avatar")]
pub async fn upload_avatar(
    MultipartForm(avatar_payload): MultipartForm<AvatarUpload>,
    user_service: web::Data<Arc<dyn UserService>>,
    file_service: web::Data<Arc<dyn FileService>>,
) -> ActixResult<HttpResponse> {
    let user = user_service
        .upload_avatar(
            avatar_payload.user_id.0,
            avatar_payload.avatar.file.path(),
            avatar_payload.avatar.file_name.clone(),
            file_service.get_ref().clone(),
        )
        .await?;

    respond_ok(user)
}
