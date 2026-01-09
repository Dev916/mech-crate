//! Health check handlers

use actix_web::{web, HttpResponse};
use serde::Serialize;
use chrono::{DateTime, Utc};

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct StatusResponse {
    status: &'static str,
    version: &'static str,
    uptime_seconds: u64,
}

/// Health check endpoint
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok",
        timestamp: Utc::now(),
    })
}

/// Status endpoint with more details
pub async fn status() -> HttpResponse {
    HttpResponse::Ok().json(StatusResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        uptime_seconds: 0, // TODO: Track actual uptime
    })
}
