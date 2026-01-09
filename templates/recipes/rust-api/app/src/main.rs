//! {{SERVICE_NAME}} API Service
//!
//! A production-ready Rust API built with Actix-web.

mod api;
mod domain;
mod infra;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::infra::config::Config;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Load environment
    dotenvy::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,actix_web=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Load configuration
    let config = Config::from_env()?;
    let bind_addr = format!("0.0.0.0:{}", config.port);
    
    tracing::info!("Starting server on {}", bind_addr);
    
    HttpServer::new(move || {
        let cors = Cors::permissive();
        
        App::new()
            .wrap(TracingLogger::default())
            .wrap(middleware::Compress::default())
            .wrap(cors)
            .configure(api::configure)
    })
    .bind(&bind_addr)?
    .run()
    .await?;
    
    Ok(())
}
