//! {{SERVICE_NAME}} - Actix Server Entry Point
//!
//! Starts the Leptos SSR server with:
//! - Actix-web HTTP server
//! - Actor system for stateful entities
//! - Database connection pool
//! - Redis connection pool

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use {{SERVICE_SLUG}}::server;
    server::run().await
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // Client-side only - no server main
}
