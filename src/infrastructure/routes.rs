use super::handlers;

pub fn infrastructure_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(handlers::index)
        .service(handlers::get_turn_credentials)
        .service(handlers::health_check);
}
