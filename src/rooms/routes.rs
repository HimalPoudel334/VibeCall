use actix_web::web;

use crate::rooms::handlers;

pub fn room_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/room").service(handlers::get_room));
}
