//! Domain Ports
//!
//! Interfaces for external dependencies.
//! Infrastructure layer implements these.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::models::{Job, JobStatus};

/// Job repository port
#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Job>>;
    async fn list_pending(&self, limit: i64) -> anyhow::Result<Vec<Job>>;
    async fn save(&self, job: &Job) -> anyhow::Result<()>;
    async fn update_status(&self, id: Uuid, status: JobStatus) -> anyhow::Result<()>;
    async fn count_by_status(&self, status: JobStatus) -> anyhow::Result<i64>;
}

/// Queue port
#[async_trait]
pub trait Queue: Send + Sync {
    async fn publish(&self, channel: &str, message: &str) -> anyhow::Result<()>;
    async fn subscribe(&self, channel: &str) -> anyhow::Result<Box<dyn tokio_stream::Stream<Item = String> + Send + Unpin>>;
}

/// LLM port
#[async_trait]
pub trait LlmEngine: Send + Sync {
    async fn generate(&self, prompt: &str, max_tokens: usize) -> anyhow::Result<String>;
    async fn evaluate(&self, content: &str) -> anyhow::Result<f64>;
}

/// Clock port (for testing)
pub trait Clock: Send + Sync {
    fn now(&self) -> chrono::DateTime<chrono::Utc>;
}

/// System clock implementation
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}
