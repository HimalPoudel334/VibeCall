use std::sync::Arc;

use actix_web::{HttpResponse, Result as ActixResult, get, web};

use crate::{rooms::RoomService, shared::response::respond_ok};

#[get("/{room_id}")]
async fn get_room(
    room_id: web::Path<String>,
    room_service: web::Data<Arc<dyn RoomService>>,
) -> ActixResult<HttpResponse> {
    respond_ok(room_service.get_room(&room_id.into_inner()).await?)
}
