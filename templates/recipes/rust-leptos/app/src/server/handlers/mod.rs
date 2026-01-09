//! HTTP Handlers
//!
//! API routes and handlers for the Actix-web server.
//! Follows hexagonal architecture - handlers are adapters that translate
//! HTTP to domain operations.

mod api;
mod health;

pub use api::api_routes;
pub use health::health_check;
