//! Health Check Handler

use actix_web::{web, HttpResponse};
use serde::Serialize;

use crate::server::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    database: &'static str,
    redis: &'static str,
    version: &'static str,
}

/// Health check endpoint
pub async fn health_check(state: web::Data<AppState>) -> HttpResponse {
    // Check database
    let db_status = match state.db.acquire().await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    // Check Redis
    let redis_status = match state.redis.get().await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    let response = HealthResponse {
        status: if db_status == "healthy" && redis_status == "healthy" {
            "healthy"
        } else {
            "degraded"
        },
        database: db_status,
        redis: redis_status,
        version: env!("CARGO_PKG_VERSION"),
    };

    if response.status == "healthy" {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::ServiceUnavailable().json(response)
    }
}
