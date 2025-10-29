use actix_web::web;

use super::handlers;

pub fn auth_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(handlers::login)
            .service(handlers::login_post),
    );
}
