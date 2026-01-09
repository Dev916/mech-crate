//! Status CLI Command
//!
//! Check status of workers and jobs.

use redis::AsyncCommands;
use serde::Serialize;

use super::StatusArgs;

#[derive(Serialize)]
struct WorkerStatus {
    redis_connected: bool,
    db_connected: bool,
    pending_jobs: Option<u64>,
    active_workers: Option<u64>,
}

pub async fn run(args: StatusArgs) -> anyhow::Result<()> {
    let mut status = WorkerStatus {
        redis_connected: false,
        db_connected: false,
        pending_jobs: None,
        active_workers: None,
    };

    // Check Redis
    match redis::Client::open(args.redis_url.as_str()) {
        Ok(client) => {
            match client.get_multiplexed_async_connection().await {
                Ok(mut conn) => {
                    status.redis_connected = true;
                    
                    // Get worker count from Redis
                    let workers: Option<u64> = conn.get("{{SERVICE_SLUG}}:workers:count").await.ok();
                    status.active_workers = workers;
                    
                    // Get pending job count
                    let pending: Option<u64> = conn.get("{{SERVICE_SLUG}}:jobs:pending").await.ok();
                    status.pending_jobs = pending;
                }
                Err(e) => {
                    tracing::error!("Redis connection failed: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::error!("Invalid Redis URL: {}", e);
        }
    }

    // Check Database
    if let Some(db_url) = args.database_url {
        match sqlx::PgPool::connect(&db_url).await {
            Ok(pool) => {
                match sqlx::query("SELECT 1").execute(&pool).await {
                    Ok(_) => status.db_connected = true,
                    Err(e) => tracing::error!("Database query failed: {}", e),
                }
            }
            Err(e) => {
                tracing::error!("Database connection failed: {}", e);
            }
        }
    }

    // Output
    match args.format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        _ => {
            println!("Worker Status");
            println!("─────────────────────────────────────");
            println!("Redis:          {}", if status.redis_connected { "✓ Connected" } else { "✗ Disconnected" });
            println!("Database:       {}", if status.db_connected { "✓ Connected" } else { "✗ Disconnected" });
            if let Some(workers) = status.active_workers {
                println!("Active Workers: {}", workers);
            }
            if let Some(pending) = status.pending_jobs {
                println!("Pending Jobs:   {}", pending);
            }
        }
    }

    Ok(())
}
