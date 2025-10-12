use actix_web::{middleware, web};

use crate::infrastructure::middlewares::auth_middleware;

use super::handlers;

pub fn infrastructure_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .wrap(middleware::from_fn(auth_middleware::auth))
            .service(handlers::index)
            .service(handlers::get_turn_credentials)
            .service(handlers::health_check),
    );
}
