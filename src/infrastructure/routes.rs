use super::handlers;

pub fn infrastructure_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(handlers::health_check);
}
