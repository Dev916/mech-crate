//! Actix Web Server Module
//!
//! Contains:
//! - Server startup and configuration
//! - Actors for stateful entities
//! - HTTP handlers and middleware
//! - Server function implementations

pub mod actors;
pub mod handlers;
mod config;

use actix_files::Files;
use actix_web::{web, App, HttpServer};
use leptos::config::get_configuration;
use leptos_actix::{generate_route_list, LeptosRoutes};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::app::App as LeptosApp;
use crate::infra::{db, cache};

pub use config::ServerConfig;

/// Application state shared across handlers
pub struct AppState {
    pub db: db::DbPool,
    pub redis: cache::RedisPool,
    pub leptos_options: leptos::config::LeptosOptions,
}

/// Run the Actix-web server
pub async fn run() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,{{SERVICE_SLUG}}=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment
    dotenvy::dotenv().ok();

    // Load configuration
    let config = ServerConfig::from_env();
    
    // Leptos configuration
    let leptos_conf = get_configuration(None).unwrap();
    let leptos_options = leptos_conf.leptos_options.clone();
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(LeptosApp);

    // Initialize database pool
    let db_pool = db::create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");
    
    info!("Database pool created");

    // Initialize Redis pool
    let redis_pool = cache::create_pool(&config.redis_url)
        .expect("Failed to create Redis pool");
    
    info!("Redis pool created");

    // Start actor system
    let _session_manager = actors::SessionManager::start();
    info!("Actor system started");

    info!("Starting server at http://{}", addr);

    HttpServer::new(move || {
        let state = AppState {
            db: db_pool.clone(),
            redis: redis_pool.clone(),
            leptos_options: leptos_options.clone(),
        };

        App::new()
            .app_data(web::Data::new(state))
            // API routes
            .configure(handlers::api_routes)
            // Health check
            .route("/health", web::get().to(handlers::health_check))
            // Static files
            .service(Files::new("/pkg", &leptos_options.site_pkg_dir))
            .service(Files::new("/assets", &leptos_options.site_root).show_files_listing())
            // Leptos routes (must be last)
            .leptos_routes(routes.clone(), {
                let leptos_options = leptos_options.clone();
                move || {
                    use leptos::prelude::*;
                    view! {
                        <!DOCTYPE html>
                        <html lang="en">
                            <head>
                                <meta charset="utf-8"/>
                                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                                <AutoReload options=leptos_options.clone()/>
                                <HydrationScripts options=leptos_options.clone()/>
                            </head>
                            <body>
                                <LeptosApp/>
                            </body>
                        </html>
                    }
                }
            })
    })
    .bind(&addr)?
    .run()
    .await
}
