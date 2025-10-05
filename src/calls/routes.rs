use crate::calls::{handlers, websocket};
use actix_web::web;

pub fn call_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/call")
            .service(web::scope("/ws").service(websocket::websocket_handler))
            .service(handlers::echo)
            .service(handlers::create_call)
            .service(handlers::get_call_by_id)
            .service(handlers::update_call_status)
            .service(handlers::end_call)
            .service(handlers::get_calls_by_room_id)
            .service(handlers::get_calls_by_user_id)
            .service(handlers::get_user_participated_calls)
            .service(handlers::get_active_calls)
            .service(handlers::list_call_participants)
            .service(handlers::add_call_participant)
            .service(handlers::remove_call_participant)
            .service(handlers::list_active_participants)
            .service(handlers::count_active_participants),
    );
}
