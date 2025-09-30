use actix_web::web;

use crate::rooms::handlers;

pub fn room_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/room")
            .service(handlers::get_room)
            .service(handlers::create_room)
            .service(handlers::list_rooms)
            .service(handlers::delete_room)
            .service(handlers::join_room)
            .service(handlers::leave_room)
            .service(handlers::list_room_users)
            .service(handlers::is_user_in_room)
            .service(handlers::is_user_owner),
    );
}
