//! Job Coordinator Actor
//!
//! Coordinates job distribution across workers.
//! Implements fair scheduling and priority handling.

use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::models::{Job, JobStatus};
use super::{Actor, ActorAddr, spawn_actor};

/// Messages for job coordinator
pub enum CoordinatorMessage {
    /// Register a new job
    RegisterJob(Job),
    /// Job completed
    JobCompleted { job_id: Uuid, success: bool },
    /// Get job status
    GetStatus { job_id: Uuid, reply: tokio::sync::oneshot::Sender<Option<JobStatus>> },
    /// Get active job count
    GetActiveCount { reply: tokio::sync::oneshot::Sender<usize> },
}

/// Job coordinator state
pub struct JobCoordinator {
    active_jobs: DashMap<Uuid, JobStatus>,
    completed_count: usize,
    failed_count: usize,
}

impl JobCoordinator {
    pub fn new() -> Self {
        Self {
            active_jobs: DashMap::new(),
            completed_count: 0,
            failed_count: 0,
        }
    }

    pub fn spawn(self) -> ActorAddr<CoordinatorMessage> {
        spawn_actor(self, 1000)
    }
}

#[async_trait::async_trait]
impl Actor for JobCoordinator {
    type Message = CoordinatorMessage;

    async fn handle(&mut self, msg: Self::Message) {
        match msg {
            CoordinatorMessage::RegisterJob(job) => {
                self.active_jobs.insert(job.id, JobStatus::Pending);
                metrics::gauge!("jobs_active").set(self.active_jobs.len() as f64);
            }
            
            CoordinatorMessage::JobCompleted { job_id, success } => {
                self.active_jobs.remove(&job_id);
                
                if success {
                    self.completed_count += 1;
                    metrics::counter!("jobs_completed_total").increment(1);
                } else {
                    self.failed_count += 1;
                    metrics::counter!("jobs_failed_total").increment(1);
                }
                
                metrics::gauge!("jobs_active").set(self.active_jobs.len() as f64);
            }
            
            CoordinatorMessage::GetStatus { job_id, reply } => {
                let status = self.active_jobs.get(&job_id).map(|s| s.clone());
                let _ = reply.send(status);
            }
            
            CoordinatorMessage::GetActiveCount { reply } => {
                let _ = reply.send(self.active_jobs.len());
            }
        }
    }

    async fn started(&mut self) {
        tracing::info!("JobCoordinator started");
    }

    async fn stopped(&mut self) {
        tracing::info!(
            completed = self.completed_count,
            failed = self.failed_count,
            "JobCoordinator stopped"
        );
    }
}
