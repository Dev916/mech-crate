//! Worker Configuration

use crate::cli::WorkerArgs;

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub workers: usize,
    pub batch_size: usize,
    pub redis_url: String,
    pub database_url: String,
    pub channel: String,
    pub metrics: bool,
    pub metrics_port: u16,
    pub llm_enabled: bool,
    pub llm_model_path: String,
    pub shutdown_timeout: u64,
}

impl WorkerConfig {
    pub fn from_args(args: &WorkerArgs) -> anyhow::Result<Self> {
        Ok(Self {
            workers: args.workers,
            batch_size: args.batch_size,
            redis_url: args.redis_url.clone(),
            database_url: args.database_url.clone(),
            channel: args.channel.clone(),
            metrics: args.metrics,
            metrics_port: args.metrics_port,
            llm_enabled: args.llm_enabled,
            llm_model_path: args.llm_model_path.clone(),
            shutdown_timeout: args.shutdown_timeout,
        })
    }
}
