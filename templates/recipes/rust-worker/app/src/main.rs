//! {{SERVICE_NAME}} - High-Performance Job Worker
//!
//! A Redis pub/sub job worker with PostgreSQL state management,
//! actor-based processing, and optional LLM evaluation capabilities.
//!
//! Architecture:
//! - `cli/` - Command-line interface with subcommands
//! - `worker/` - Job processing with actors
//! - `domain/` - Business logic (pure, no IO)
//! - `infra/` - Database, queue, LLM adapters

mod cli;
mod worker;
mod domain;
mod infra;

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cli::{Cli, Commands};
use worker::WorkerConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,{{SERVICE_SLUG}}=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse CLI
    let cli = Cli::parse();

    match cli.command {
        Commands::Worker(args) => {
            let config = WorkerConfig::from_args(&args)?;
            worker::run(config).await?;
        }
        Commands::Enqueue(args) => {
            cli::enqueue::run(args).await?;
        }
        Commands::Status(args) => {
            cli::status::run(args).await?;
        }
        Commands::Migrate => {
            cli::migrate::run().await?;
        }
    }

    Ok(())
}
