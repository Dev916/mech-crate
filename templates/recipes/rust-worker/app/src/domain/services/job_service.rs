//! Job Service
//!
//! Pure business logic for job operations.
//! All functions are pure - no IO.

use crate::domain::models::{Job, JobError, JobStatus};
use chrono::Utc;

/// Validate job can be processed
pub fn validate_processable(job: &Job) -> Result<(), JobError> {
    if job.status.is_terminal() {
        return Err(JobError::AlreadyCompleted);
    }
    
    if job.attempts >= job.max_attempts {
        return Err(JobError::MaxRetriesExceeded);
    }
    
    Ok(())
}

/// Transition job to running state
pub fn start_processing(job: Job) -> Job {
    Job {
        status: JobStatus::Running,
        started_at: Some(Utc::now()),
        attempts: job.attempts + 1,
        ..job
    }
}

/// Transition job to completed state
pub fn complete_success(job: Job, result: serde_json::Value) -> Job {
    Job {
        status: JobStatus::Completed,
        completed_at: Some(Utc::now()),
        result: Some(result),
        error: None,
        ..job
    }
}

/// Transition job to failed state
pub fn complete_failure(job: Job, error: String) -> Job {
    let status = if job.attempts >= job.max_attempts {
        JobStatus::Failed
    } else {
        JobStatus::Retrying
    };
    
    Job {
        status,
        completed_at: if status == JobStatus::Failed { Some(Utc::now()) } else { None },
        error: Some(error),
        ..job
    }
}

/// Cancel a job
pub fn cancel(job: Job, reason: String) -> Result<Job, JobError> {
    if job.status.is_terminal() {
        return Err(JobError::AlreadyCompleted);
    }
    
    Ok(Job {
        status: JobStatus::Cancelled,
        completed_at: Some(Utc::now()),
        error: Some(reason),
        ..job
    })
}

/// Calculate backoff delay for retry
pub fn calculate_backoff_ms(attempts: u32, base_ms: u64, max_ms: u64) -> u64 {
    let delay = base_ms.saturating_mul(2u64.pow(attempts));
    std::cmp::min(delay, max_ms)
}

/// Check if job should be retried
pub fn should_retry(job: &Job) -> bool {
    job.status == JobStatus::Retrying && job.attempts < job.max_attempts
}

/// Priority score for queue ordering (higher = process first)
pub fn priority_score(job: &Job) -> i64 {
    // Base priority (0-10)
    let base = job.priority as i64 * 1000;
    
    // Bonus for waiting time (older jobs get priority)
    let age_secs = (Utc::now() - job.created_at).num_seconds();
    let age_bonus = std::cmp::min(age_secs, 3600); // Cap at 1 hour
    
    // Penalty for retries
    let retry_penalty = job.attempts as i64 * 100;
    
    base + age_bonus - retry_penalty
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn test_job() -> Job {
        Job {
            id: Uuid::new_v4(),
            job_type: "test".to_string(),
            payload: serde_json::json!({}),
            priority: 5,
            status: JobStatus::Pending,
            attempts: 0,
            max_attempts: 3,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
            result: None,
        }
    }

    #[test]
    fn test_start_processing() {
        let job = test_job();
        let started = start_processing(job);
        
        assert_eq!(started.status, JobStatus::Running);
        assert!(started.started_at.is_some());
        assert_eq!(started.attempts, 1);
    }

    #[test]
    fn test_complete_success() {
        let job = start_processing(test_job());
        let completed = complete_success(job, serde_json::json!({"result": "ok"}));
        
        assert_eq!(completed.status, JobStatus::Completed);
        assert!(completed.completed_at.is_some());
        assert!(completed.result.is_some());
    }

    #[test]
    fn test_complete_failure_with_retries() {
        let mut job = start_processing(test_job());
        job.attempts = 1;
        
        let failed = complete_failure(job, "error".to_string());
        
        assert_eq!(failed.status, JobStatus::Retrying);
        assert!(failed.completed_at.is_none());
    }

    #[test]
    fn test_complete_failure_max_retries() {
        let mut job = start_processing(test_job());
        job.attempts = 3;
        
        let failed = complete_failure(job, "error".to_string());
        
        assert_eq!(failed.status, JobStatus::Failed);
        assert!(failed.completed_at.is_some());
    }

    #[test]
    fn test_backoff_calculation() {
        assert_eq!(calculate_backoff_ms(0, 100, 10000), 100);
        assert_eq!(calculate_backoff_ms(1, 100, 10000), 200);
        assert_eq!(calculate_backoff_ms(2, 100, 10000), 400);
        assert_eq!(calculate_backoff_ms(10, 100, 10000), 10000); // Capped
    }
}
