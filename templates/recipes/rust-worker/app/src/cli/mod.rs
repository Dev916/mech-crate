//! CLI Module
//!
//! Command-line interface with subcommands:
//! - `worker` - Start the job worker
//! - `enqueue` - Enqueue a job manually
//! - `status` - Check worker/job status
//! - `migrate` - Run database migrations

pub mod enqueue;
pub mod status;
pub mod migrate;

use clap::{Parser, Subcommand};

/// {{SERVICE_NAME}} - High-Performance Job Worker
#[derive(Parser, Debug)]
#[command(name = "{{SERVICE_SLUG}}")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the job worker
    Worker(WorkerArgs),
    
    /// Enqueue a job manually
    Enqueue(EnqueueArgs),
    
    /// Check status of workers and jobs
    Status(StatusArgs),
    
    /// Run database migrations
    Migrate,
}

/// Worker command arguments
#[derive(Parser, Debug, Clone)]
pub struct WorkerArgs {
    /// Number of worker threads
    #[arg(short = 'w', long, env = "WORKER_THREADS", default_value = "4")]
    pub workers: usize,

    /// Batch size for processing
    #[arg(short = 'b', long, env = "BATCH_SIZE", default_value = "100")]
    pub batch_size: usize,

    /// Redis URL
    #[arg(long, env = "REDIS_URL", default_value = "redis://localhost:6379")]
    pub redis_url: String,

    /// Database URL
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,

    /// Channel to subscribe to
    #[arg(short = 'c', long, env = "CHANNEL", default_value = "jobs")]
    pub channel: String,

    /// Enable metrics server
    #[arg(long, env = "METRICS_ENABLED", default_value = "true")]
    pub metrics: bool,

    /// Metrics port
    #[arg(long, env = "METRICS_PORT", default_value = "9090")]
    pub metrics_port: u16,

    /// Enable LLM processing
    #[arg(long, env = "LLM_ENABLED", default_value = "false")]
    pub llm_enabled: bool,

    /// LLM model path
    #[arg(long, env = "LLM_MODEL_PATH", default_value = "/models/model.gguf")]
    pub llm_model_path: String,

    /// Graceful shutdown timeout (seconds)
    #[arg(long, env = "SHUTDOWN_TIMEOUT", default_value = "30")]
    pub shutdown_timeout: u64,
}

/// Enqueue command arguments
#[derive(Parser, Debug)]
pub struct EnqueueArgs {
    /// Job type
    #[arg(short = 't', long)]
    pub job_type: String,

    /// Job payload (JSON)
    #[arg(short = 'p', long)]
    pub payload: String,

    /// Priority (0-10, higher = more urgent)
    #[arg(long, default_value = "5")]
    pub priority: u8,

    /// Redis URL
    #[arg(long, env = "REDIS_URL", default_value = "redis://localhost:6379")]
    pub redis_url: String,

    /// Channel to publish to
    #[arg(short = 'c', long, env = "CHANNEL", default_value = "jobs")]
    pub channel: String,
}

/// Status command arguments
#[derive(Parser, Debug)]
pub struct StatusArgs {
    /// Redis URL
    #[arg(long, env = "REDIS_URL", default_value = "redis://localhost:6379")]
    pub redis_url: String,

    /// Database URL
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: Option<String>,

    /// Output format (text, json)
    #[arg(short = 'f', long, default_value = "text")]
    pub format: String,
}
