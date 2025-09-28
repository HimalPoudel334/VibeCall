use std::sync::Arc;

use actix_multipart::form::MultipartForm;
use actix_web::{HttpResponse, Result as ActixResult, get, post, web};

use crate::{
    shared::{file_service::FileService, response::ApiResponse},
    users::{
        contract::{AvatarUpload, NewUser},
        entities::User,
        service::UserService,
    },
};

#[get("/{id}")]
pub async fn get_user(
    path: web::Path<i32>,
    user_service: web::Data<Arc<dyn UserService>>,
) -> ActixResult<HttpResponse> {
    let user_id = path.into_inner();

    match user_service.get_by_id(user_id).await {
        Ok(Some(user)) => Ok(HttpResponse::Ok().json(ApiResponse::success(user))),
        Ok(None) => Ok(HttpResponse::NotFound()
            .json(ApiResponse::<User>::not_found("User not found".to_string()))),
        Err(e) => Ok(HttpResponse::InternalServerError()
            .json(ApiResponse::<User>::error(format!("Database error: {}", e)))),
    }
}

#[post("")]
pub async fn create_user(
    user_json: web::Json<NewUser>,
    user_service: web::Data<Arc<dyn UserService>>,
) -> ActixResult<HttpResponse> {
    match user_service
        .create(
            user_json.first_name.clone(),
            user_json.last_name.clone(),
            user_json.email.get_email().to_string(),
            user_json.phone.get_number().to_string(),
            user_json.password.clone(),
        )
        .await
    {
        Ok(user) => Ok(HttpResponse::Ok().json(ApiResponse::success(user))),
        Err(e) => Ok(
            HttpResponse::BadRequest().json(ApiResponse::<User>::error(format!(
                "Error creating user: {}",
                e
            ))),
        ),
    }
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

    Ok(HttpResponse::Ok().json(ApiResponse::success(user)))
}
