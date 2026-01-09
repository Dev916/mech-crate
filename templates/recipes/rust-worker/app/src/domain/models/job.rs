//! Job Domain Model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Job entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique job ID (UUID v7 for time-ordering)
    pub id: Uuid,
    /// Job type (determines handler)
    pub job_type: String,
    /// Job payload (JSON)
    pub payload: serde_json::Value,
    /// Priority (0-10, higher = more urgent)
    pub priority: u8,
    /// Current status
    pub status: JobStatus,
    /// Number of processing attempts
    pub attempts: u32,
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// When job was created
    pub created_at: DateTime<Utc>,
    /// When job started processing
    pub started_at: Option<DateTime<Utc>>,
    /// When job completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Job result (if completed)
    pub result: Option<serde_json::Value>,
}

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Retrying,
    Cancelled,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Retrying => "retrying",
            JobStatus::Cancelled => "cancelled",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled)
    }
}

/// Job result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub output: serde_json::Value,
}

/// Job creation command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJob {
    pub job_type: String,
    pub payload: serde_json::Value,
    pub priority: Option<u8>,
    pub max_attempts: Option<u32>,
}

impl CreateJob {
    pub fn into_job(self) -> Job {
        Job {
            id: Uuid::now_v7(),
            job_type: self.job_type,
            payload: self.payload,
            priority: self.priority.unwrap_or(5),
            status: JobStatus::Pending,
            attempts: 0,
            max_attempts: self.max_attempts.unwrap_or(3),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
            result: None,
        }
    }
}

/// Job errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum JobError {
    #[error("Job not found: {0}")]
    NotFound(Uuid),
    
    #[error("Invalid job type: {0}")]
    InvalidType(String),
    
    #[error("Job already completed")]
    AlreadyCompleted,
    
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
    
    #[error("Job cancelled")]
    Cancelled,
    
    #[error("Processing error: {0}")]
    ProcessingError(String),
}
