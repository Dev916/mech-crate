//! Job Handlers
//!
//! Each job type has its own handler module.
//! Handlers are pure functions that take a job and return a result.

pub mod batch;
pub mod compute;
pub mod llm;
pub mod webhook;

use crate::domain::models::{Job, JobResult};
use crate::worker::WorkerState;

/// Common job handler trait
#[async_trait::async_trait]
pub trait JobHandler: Send + Sync {
    async fn process(&self, job: &Job, state: &WorkerState) -> anyhow::Result<JobResult>;
}
