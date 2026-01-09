//! Enqueue CLI Command
//!
//! Manually enqueue jobs for processing.

use redis::AsyncCommands;
use uuid::Uuid;
use chrono::Utc;

use crate::domain::models::{Job, JobStatus};
use super::EnqueueArgs;

pub async fn run(args: EnqueueArgs) -> anyhow::Result<()> {
    tracing::info!("Connecting to Redis at {}", args.redis_url);
    
    let client = redis::Client::open(args.redis_url.as_str())?;
    let mut conn = client.get_multiplexed_async_connection().await?;

    // Parse payload
    let payload: serde_json::Value = serde_json::from_str(&args.payload)
        .map_err(|e| anyhow::anyhow!("Invalid JSON payload: {}", e))?;

    // Create job
    let job = Job {
        id: Uuid::now_v7(),
        job_type: args.job_type.clone(),
        payload,
        priority: args.priority,
        status: JobStatus::Pending,
        attempts: 0,
        max_attempts: 3,
        created_at: Utc::now(),
        started_at: None,
        completed_at: None,
        error: None,
        result: None,
    };

    // Serialize and publish
    let message = serde_json::to_string(&job)?;
    let subscribers: i64 = conn.publish(&args.channel, &message).await?;

    if subscribers > 0 {
        tracing::info!(
            "✓ Enqueued job {} ({}) to {} ({} subscribers)",
            job.id, args.job_type, args.channel, subscribers
        );
    } else {
        tracing::warn!(
            "⚠ Enqueued job {} ({}) to {} but no subscribers!",
            job.id, args.job_type, args.channel
        );
    }

    println!("{}", serde_json::to_string_pretty(&job)?);

    Ok(())
}
