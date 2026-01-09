//! API layer - HTTP handlers and routing

pub mod handlers;
pub mod middleware;

use actix_web::web;

/// Configure API routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(handlers::health::health_check))
            .route("/status", web::get().to(handlers::health::status))
    );
}
