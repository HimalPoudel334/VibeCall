use super::handlers;
use actix_web::web;

pub fn user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .service(handlers::get_user)
            .service(handlers::create_user)
            .service(handlers::upload_avatar),
    );
}
