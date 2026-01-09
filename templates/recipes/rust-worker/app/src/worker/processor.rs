//! Job Processor
//!
//! Processes jobs from the queue using a worker pool.
//! Implements actor-like message processing with supervision.
//!
//! See: appendix-actor-model.md, appendix-concurrency-time.md

use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use async_channel::{Receiver, Sender, bounded};
use redis::AsyncCommands;
use tokio_graceful_shutdown::SubsystemHandle;
use tracing::{debug, error, info, warn, Span};
use uuid::Uuid;

use crate::domain::models::{Job, JobStatus, JobResult};
use crate::domain::services::job_service;
use super::WorkerState;
use super::jobs;

/// Global job sender (for subscriber to send jobs)
pub static JOB_SENDER: OnceLock<Sender<Job>> = OnceLock::new();

/// Job processing result
pub struct ProcessResult {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub result: Option<JobResult>,
    pub error: Option<String>,
    pub duration: Duration,
}

/// Run the job processor
pub async fn run(subsys: SubsystemHandle, state: Arc<WorkerState>) -> anyhow::Result<()> {
    let workers = state.config.workers;
    info!(workers, "Starting job processor");

    // Create bounded channel for backpressure
    let (tx, rx) = bounded::<Job>(state.config.batch_size * 2);
    
    // Store sender globally for subscriber
    JOB_SENDER.set(tx).expect("JOB_SENDER already set");

    // Spawn worker tasks
    let mut handles = Vec::with_capacity(workers);
    
    for worker_id in 0..workers {
        let rx = rx.clone();
        let state = state.clone();
        let subsys = subsys.clone();
        
        let handle = tokio::spawn(async move {
            worker_loop(worker_id, rx, state, subsys).await
        });
        
        handles.push(handle);
    }

    // Wait for all workers to complete
    for handle in handles {
        if let Err(e) = handle.await {
            error!("Worker task failed: {}", e);
        }
    }

    info!("All workers stopped");
    Ok(())
}

/// Individual worker loop
async fn worker_loop(
    worker_id: usize,
    rx: Receiver<Job>,
    state: Arc<WorkerState>,
    subsys: SubsystemHandle,
) {
    info!(worker_id, "Worker started");
    
    loop {
        tokio::select! {
            _ = subsys.on_shutdown_requested() => {
                info!(worker_id, "Worker shutting down");
                break;
            }
            job = rx.recv() => {
                match job {
                    Ok(job) => {
                        let result = process_job(worker_id, &job, &state).await;
                        handle_result(&job, result, &state).await;
                    }
                    Err(_) => {
                        debug!(worker_id, "Channel closed, worker stopping");
                        break;
                    }
                }
            }
        }
    }
    
    info!(worker_id, "Worker stopped");
}

/// Process a single job
async fn process_job(
    worker_id: usize,
    job: &Job,
    state: &WorkerState,
) -> ProcessResult {
    let start = Instant::now();
    let span = tracing::info_span!(
        "process_job",
        job_id = %job.id,
        job_type = job.job_type,
        worker_id
    );
    let _guard = span.enter();

    debug!("Processing job");
    metrics::counter!("jobs_started").increment(1);

    // Update job status in DB
    if let Err(e) = update_job_status(&state.db, job.id, JobStatus::Running).await {
        warn!("Failed to update job status: {}", e);
    }

    // Dispatch to appropriate handler
    let result = match job.job_type.as_str() {
        "batch_process" => jobs::batch::process(job, state).await,
        "compute" => jobs::compute::process(job, state).await,
        "llm_evaluate" => jobs::llm::process(job, state).await,
        "webhook" => jobs::webhook::process(job, state).await,
        _ => {
            Err(anyhow::anyhow!("Unknown job type: {}", job.job_type))
        }
    };

    let duration = start.elapsed();

    match result {
        Ok(job_result) => {
            debug!(duration_ms = duration.as_millis(), "Job completed");
            metrics::counter!("jobs_completed").increment(1);
            metrics::histogram!("job_duration_ms").record(duration.as_millis() as f64);
            
            ProcessResult {
                job_id: job.id,
                status: JobStatus::Completed,
                result: Some(job_result),
                error: None,
                duration,
            }
        }
        Err(e) => {
            error!(error = %e, "Job failed");
            metrics::counter!("jobs_failed").increment(1);
            
            ProcessResult {
                job_id: job.id,
                status: if job.attempts + 1 >= job.max_attempts {
                    JobStatus::Failed
                } else {
                    JobStatus::Retrying
                },
                result: None,
                error: Some(e.to_string()),
                duration,
            }
        }
    }
}

/// Handle job result (update DB, retry if needed)
async fn handle_result(job: &Job, result: ProcessResult, state: &WorkerState) {
    // Decrement pending counter
    if let Ok(mut conn) = state.redis.get().await {
        let _: Result<(), _> = conn.decr("{{SERVICE_SLUG}}:jobs:pending", 1).await;
    }

    // Update job in database
    if let Err(e) = save_job_result(&state.db, &result).await {
        error!(job_id = %job.id, error = %e, "Failed to save job result");
    }

    // Handle retry
    if result.status == JobStatus::Retrying {
        let mut retry_job = job.clone();
        retry_job.attempts += 1;
        retry_job.status = JobStatus::Pending;
        
        // Exponential backoff
        let delay = Duration::from_secs(2u64.pow(retry_job.attempts as u32));
        
        info!(
            job_id = %job.id,
            attempt = retry_job.attempts,
            delay_secs = delay.as_secs(),
            "Scheduling retry"
        );

        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            if let Some(sender) = JOB_SENDER.get() {
                let _ = sender.send(retry_job).await;
            }
        });
    }
}

async fn update_job_status(db: &sqlx::PgPool, job_id: Uuid, status: JobStatus) -> anyhow::Result<()> {
    sqlx::query!(
        r#"UPDATE jobs SET status = $2, started_at = NOW() WHERE id = $1"#,
        job_id,
        status.as_str()
    )
    .execute(db)
    .await?;
    
    Ok(())
}

async fn save_job_result(db: &sqlx::PgPool, result: &ProcessResult) -> anyhow::Result<()> {
    let result_json = result.result.as_ref().map(|r| serde_json::to_value(r).unwrap());
    
    sqlx::query!(
        r#"
        UPDATE jobs 
        SET status = $2, 
            completed_at = NOW(), 
            result = $3, 
            error = $4
        WHERE id = $1
        "#,
        result.job_id,
        result.status.as_str(),
        result_json,
        result.error
    )
    .execute(db)
    .await?;
    
    Ok(())
}
