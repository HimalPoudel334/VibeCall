use std::sync::Arc;

use actix_web::{HttpResponse, Result as ActixResult, get, post, web};

use crate::{
    calls::{
        CallService,
        contract::{NewCall, UpdateCallStatus, UserIdParam},
    },
    shared::response::{AppError, respond_ok},
};

#[get("/{call_id}")]
pub async fn get_call_by_id(
    call_id: web::Path<i32>,
    call_service: web::Data<Arc<dyn CallService>>,
) -> ActixResult<HttpResponse> {
    let call_id = call_id.into_inner();
    let call = call_service
        .get_call_by_id(call_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Call {} not found", call_id)))?;

    respond_ok(call)
}

#[post("")]
pub async fn create_call(
    call_json: web::Json<NewCall>,
    call_service: web::Data<Arc<dyn CallService>>,
) -> ActixResult<HttpResponse> {
    let call = call_service
        .create_call(
            call_json.room_id.clone(),
            call_json.caller_id,
            call_json.status.clone(),
        )
        .await?;

    respond_ok(call)
}

#[post("/{call_id}/update-status")]
pub async fn update_call_status(
    call_id: web::Path<i32>,
    status_json: web::Json<UpdateCallStatus>,
    call_service: web::Data<Arc<dyn CallService>>,
) -> ActixResult<HttpResponse> {
    let call_id = call_id.into_inner();
    call_service
        .update_call_status(call_id, status_json.status.clone())
        .await?;

    respond_ok("Call status updated successfully")
}

#[post("/{call_id}/end")]
pub async fn end_call(
    call_id: web::Path<i32>,
    user_id: web::Json<UserIdParam>,
    call_service: web::Data<Arc<dyn CallService>>,
) -> ActixResult<HttpResponse> {
    let call_id = call_id.into_inner();
    call_service.end_call(call_id, user_id.user_id).await?;
    respond_ok("Call ended successfully")
}

#[get("/room/{room_id}")]
pub async fn get_calls_by_room_id(
    room_id: web::Path<String>,
    call_service: web::Data<Arc<dyn CallService>>,
) -> ActixResult<HttpResponse> {
    let room_id = room_id.into_inner();
    let calls = call_service.get_calls_by_room_id(&room_id).await?;
    respond_ok(calls)
}

#[get("/user/{user_id}")]
pub async fn get_calls_by_user_id(
    user_id: web::Path<i32>,
    call_service: web::Data<Arc<dyn CallService>>,
) -> ActixResult<HttpResponse> {
    let user_id = user_id.into_inner();
    let calls = call_service.get_calls_by_user_id(user_id).await?;
    respond_ok(calls)
}

#[get("/user/{user_id}/active")]
pub async fn get_user_participated_calls(
    user_id: web::Path<i32>,
    call_service: web::Data<Arc<dyn CallService>>,
) -> ActixResult<HttpResponse> {
    let user_id = user_id.into_inner();
    let calls = call_service.get_user_participated_calls(user_id).await?;
    respond_ok(calls)
}

#[get("/active")]
pub async fn get_active_calls(
    call_service: web::Data<Arc<dyn CallService>>,
) -> ActixResult<HttpResponse> {
    let calls = call_service.get_active_calls().await?;
    respond_ok(calls)
}

#[get("/{call_id}/participants")]
pub async fn list_call_participants(
    call_id: web::Path<i32>,
    call_service: web::Data<Arc<dyn CallService>>,
) -> ActixResult<HttpResponse> {
    let call_id = call_id.into_inner();
    let participants = call_service.list_call_participants(call_id).await?;
    respond_ok(participants)
}

#[post("/{call_id}/participants/add")]
pub async fn add_call_participant(
    call_id: web::Path<i32>,
    user_id: web::Json<UserIdParam>,
    call_service: web::Data<Arc<dyn CallService>>,
) -> ActixResult<HttpResponse> {
    let call_id = call_id.into_inner();
    call_service
        .add_call_participant(call_id, user_id.user_id)
        .await?;
    respond_ok("Participant added successfully")
}

#[post("/{call_id}/participants/remove")]
pub async fn remove_call_participant(
    call_id: web::Path<i32>,
    user_id: web::Json<UserIdParam>,
    call_service: web::Data<Arc<dyn CallService>>,
) -> ActixResult<HttpResponse> {
    let call_id = call_id.into_inner();
    call_service
        .remove_call_participant(call_id, user_id.user_id)
        .await?;
    respond_ok("Participant removed successfully")
}

#[get("/{call_id}/participants/active")]
pub async fn list_active_participants(
    call_id: web::Path<i32>,
    call_service: web::Data<Arc<dyn CallService>>,
) -> ActixResult<HttpResponse> {
    let call_id = call_id.into_inner();
    let participants = call_service.list_active_participants(call_id).await?;
    respond_ok(participants)
}

#[get("/{call_id}/participants/active/count")]
pub async fn count_active_participants(
    call_id: web::Path<i32>,
    call_service: web::Data<Arc<dyn CallService>>,
) -> ActixResult<HttpResponse> {
    let call_id = call_id.into_inner();
    let count = call_service.count_active_participants(call_id).await?;
    respond_ok(count)
}
