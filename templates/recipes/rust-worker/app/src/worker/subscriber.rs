//! Redis Pub/Sub Subscriber
//!
//! Subscribes to Redis channels and forwards jobs to the processor.
//! Implements backpressure via bounded channels.
//!
//! See: appendix-streams.md, appendix-frp-rust.md

use std::sync::Arc;
use redis::AsyncCommands;
use tokio_graceful_shutdown::SubsystemHandle;
use tracing::{debug, error, info, warn};

use crate::domain::models::Job;
use super::WorkerState;
use super::processor::JOB_SENDER;

/// Run the Redis subscriber
pub async fn run(subsys: SubsystemHandle, state: Arc<WorkerState>) -> anyhow::Result<()> {
    info!(channel = state.config.channel, "Starting Redis subscriber");

    let client = redis::Client::open(state.config.redis_url.as_str())?;
    let mut pubsub = client.get_async_pubsub().await?;
    
    pubsub.subscribe(&state.config.channel).await?;
    info!(channel = state.config.channel, "Subscribed to channel");

    // Update worker count in Redis
    {
        let mut conn = state.redis.get().await?;
        let _: () = conn.incr("{{SERVICE_SLUG}}:workers:count", 1).await?;
    }

    // Message loop
    let mut stream = pubsub.into_on_message();
    
    loop {
        tokio::select! {
            _ = subsys.on_shutdown_requested() => {
                info!("Subscriber shutting down");
                break;
            }
            msg = stream.next() => {
                match msg {
                    Some(msg) => {
                        let payload: String = msg.get_payload()?;
                        
                        match serde_json::from_str::<Job>(&payload) {
                            Ok(job) => {
                                debug!(job_id = %job.id, job_type = job.job_type, "Received job");
                                
                                // Increment pending counter
                                if let Ok(mut conn) = state.redis.get().await {
                                    let _: Result<(), _> = conn.incr("{{SERVICE_SLUG}}:jobs:pending", 1).await;
                                }
                                
                                // Send to processor with backpressure
                                if let Some(sender) = JOB_SENDER.get() {
                                    if sender.is_full() {
                                        warn!(job_id = %job.id, "Job queue full, applying backpressure");
                                        metrics::counter!("jobs_backpressure").increment(1);
                                    }
                                    
                                    if let Err(e) = sender.send(job).await {
                                        error!("Failed to send job to processor: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to parse job: {}", e);
                                metrics::counter!("jobs_parse_errors").increment(1);
                            }
                        }
                    }
                    None => {
                        warn!("Redis subscription ended");
                        break;
                    }
                }
            }
        }
    }

    // Decrement worker count
    {
        if let Ok(mut conn) = state.redis.get().await {
            let _: Result<(), _> = conn.decr("{{SERVICE_SLUG}}:workers:count", 1).await;
        }
    }

    Ok(())
}

// Helper for tokio stream
use futures::StreamExt;

trait AsyncPubSubExt {
    fn next(&mut self) -> impl std::future::Future<Output = Option<redis::Msg>> + Send;
}

impl AsyncPubSubExt for redis::aio::PubSub {
    async fn next(&mut self) -> Option<redis::Msg> {
        self.on_message().next().await
    }
}
