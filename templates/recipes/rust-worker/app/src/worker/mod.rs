//! Worker Module
//!
//! Job processing with:
//! - Redis pub/sub subscription
//! - Actor-based job processing
//! - Backpressure and rate limiting
//! - Graceful shutdown
//!
//! See: appendix-actor-model.md, appendix-frp-rust.md

pub mod actors;
pub mod jobs;
mod config;
mod subscriber;
mod processor;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_graceful_shutdown::{SubsystemBuilder, SubsystemHandle, Toplevel};
use tracing::info;

pub use config::WorkerConfig;

use crate::infra::{db, queue};

/// Shared worker state
pub struct WorkerState {
    pub config: WorkerConfig,
    pub db: db::DbPool,
    pub redis: queue::RedisPool,
    pub shutdown_tx: broadcast::Sender<()>,
}

/// Run the worker
pub async fn run(config: WorkerConfig) -> anyhow::Result<()> {
    info!(
        workers = config.workers,
        batch_size = config.batch_size,
        channel = config.channel,
        "Starting worker"
    );

    // Create database pool
    let db = db::create_pool(&config.database_url, config.workers as u32 * 2).await?;
    info!("Database pool created");

    // Create Redis pool
    let redis = queue::create_pool(&config.redis_url)?;
    info!("Redis pool created");

    // Shutdown channel
    let (shutdown_tx, _) = broadcast::channel(1);

    // Create shared state
    let state = Arc::new(WorkerState {
        config: config.clone(),
        db,
        redis,
        shutdown_tx: shutdown_tx.clone(),
    });

    // Start metrics server if enabled
    if config.metrics {
        start_metrics_server(config.metrics_port)?;
    }

    // Run with graceful shutdown
    Toplevel::new(|s| async move {
        s.start(SubsystemBuilder::new("subscriber", {
            let state = state.clone();
            move |subsys| subscriber::run(subsys, state)
        }));

        s.start(SubsystemBuilder::new("processor", {
            let state = state.clone();
            move |subsys| processor::run(subsys, state)
        }));
    })
    .catch_signals()
    .handle_shutdown_requests(Duration::from_secs(config.shutdown_timeout))
    .await
    .map_err(|e| anyhow::anyhow!("Shutdown error: {:?}", e))
}

fn start_metrics_server(port: u16) -> anyhow::Result<()> {
    use metrics_exporter_prometheus::PrometheusBuilder;

    PrometheusBuilder::new()
        .with_http_listener(([0, 0, 0, 0], port))
        .install()?;

    info!(port, "Metrics server started");
    Ok(())
}
