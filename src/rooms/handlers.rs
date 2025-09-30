use std::sync::Arc;

use actix_web::{HttpResponse, Result as ActixResult, delete, get, post, web};

use crate::{
    rooms::{
        RoomService,
        contract::{NewRoom, PaginationParams, UserIdParam},
    },
    shared::response::{AppError, respond_ok},
};

#[get("/{room_id}")]
pub async fn get_room(
    room_id: web::Path<String>,
    room_service: web::Data<Arc<dyn RoomService>>,
) -> ActixResult<HttpResponse> {
    let room_id = room_id.into_inner();

    let room = room_service
        .get_room(&room_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Room with id {} not found", room_id)))?;

    respond_ok(room)
}

#[post("")]
pub async fn create_room(
    room_json: web::Json<NewRoom>,
    room_service: web::Data<Arc<dyn RoomService>>,
) -> ActixResult<HttpResponse> {
    let room = room_service
        .create_room(
            room_json.name.clone(),
            room_json.room_type.clone(),
            room_json.created_by,
            room_json.description.clone(),
        )
        .await?;

    respond_ok(room)
}

#[get("")]
pub async fn list_rooms(
    query: web::Query<PaginationParams>,
    room_service: web::Data<Arc<dyn RoomService>>,
) -> ActixResult<HttpResponse> {
    let limit = query.limit.unwrap_or(10);
    let offset = query.offset.unwrap_or(0);
    let rooms = room_service.list_rooms(limit, offset).await?;
    respond_ok(rooms)
}

#[delete("/{room_id}")]
pub async fn delete_room(
    room_id: web::Path<String>,
    payload: web::Json<UserIdParam>,
    room_service: web::Data<Arc<dyn RoomService>>,
) -> ActixResult<HttpResponse> {
    let room_id = room_id.into_inner();
    room_service.delete_room(&room_id, payload.user_id).await?;
    respond_ok("Room deleted successfully")
}

#[post("/{room_id}/join")]
pub async fn join_room(
    room_id: web::Path<String>,
    payload: web::Json<UserIdParam>,
    room_service: web::Data<Arc<dyn RoomService>>,
) -> ActixResult<HttpResponse> {
    let room_id = room_id.into_inner();
    room_service.join_room(&room_id, payload.user_id).await?;
    respond_ok("Joined room successfully")
}

#[post("/{room_id}/leave")]
pub async fn leave_room(
    room_id: web::Path<String>,
    payload: web::Json<UserIdParam>,
    room_service: web::Data<Arc<dyn RoomService>>,
) -> ActixResult<HttpResponse> {
    let room_id = room_id.into_inner();
    room_service.leave_room(&room_id, payload.user_id).await?;
    respond_ok("Left room successfully")
}

#[get("/{room_id}/users")]
pub async fn list_room_users(
    room_id: web::Path<String>,
    room_service: web::Data<Arc<dyn RoomService>>,
) -> ActixResult<HttpResponse> {
    let room_id = room_id.into_inner();
    let users = room_service.list_room_users(&room_id).await?;
    respond_ok(users)
}

#[get("/{room_id}/users/{user_id}/is-in-room")]
pub async fn is_user_in_room(
    room_id: web::Path<String>,
    user_id: web::Path<i32>,
    room_service: web::Data<Arc<dyn RoomService>>,
) -> ActixResult<HttpResponse> {
    let room_id = room_id.into_inner();
    let user_id = user_id.into_inner();
    let is_in_room = room_service.is_user_in_room(&room_id, user_id).await?;
    respond_ok(is_in_room)
}

#[get("/{room_id}/users/{user_id}/is-owner")]
pub async fn is_user_owner(
    room_id: web::Path<String>,
    user_id: web::Path<i32>,
    room_service: web::Data<Arc<dyn RoomService>>,
) -> ActixResult<HttpResponse> {
    let room_id = room_id.into_inner();
    let user_id = user_id.into_inner();
    let is_owner = room_service.is_user_owner(&room_id, user_id).await?;
    respond_ok(is_owner)
}
