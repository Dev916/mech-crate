//! Database Infrastructure

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub type DbPool = PgPool;

/// Create database connection pool
pub async fn create_pool(database_url: &str, max_connections: u32) -> anyhow::Result<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .connect(database_url)
        .await?;
    
    Ok(pool)
}

/// Health check
pub async fn health_check(pool: &DbPool) -> anyhow::Result<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Job Repository Implementation
// ─────────────────────────────────────────────────────────────────────────────

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::models::{Job, JobStatus};
use crate::domain::ports::JobRepository;

pub struct PgJobRepository {
    pool: DbPool,
}

impl PgJobRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JobRepository for PgJobRepository {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Job>> {
        let row = sqlx::query!(
            r#"
            SELECT id, job_type, payload, priority, status, attempts, max_attempts,
                   created_at, started_at, completed_at, error, result
            FROM jobs WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Job {
            id: r.id,
            job_type: r.job_type,
            payload: r.payload,
            priority: r.priority as u8,
            status: parse_status(&r.status),
            attempts: r.attempts as u32,
            max_attempts: r.max_attempts as u32,
            created_at: r.created_at,
            started_at: r.started_at,
            completed_at: r.completed_at,
            error: r.error,
            result: r.result,
        }))
    }

    async fn list_pending(&self, limit: i64) -> anyhow::Result<Vec<Job>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, job_type, payload, priority, status, attempts, max_attempts,
                   created_at, started_at, completed_at, error, result
            FROM jobs 
            WHERE status = 'pending' OR status = 'retrying'
            ORDER BY priority DESC, created_at ASC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| Job {
            id: r.id,
            job_type: r.job_type,
            payload: r.payload,
            priority: r.priority as u8,
            status: parse_status(&r.status),
            attempts: r.attempts as u32,
            max_attempts: r.max_attempts as u32,
            created_at: r.created_at,
            started_at: r.started_at,
            completed_at: r.completed_at,
            error: r.error,
            result: r.result,
        }).collect())
    }

    async fn save(&self, job: &Job) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO jobs (id, job_type, payload, priority, status, attempts, max_attempts,
                             created_at, started_at, completed_at, error, result)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (id) DO UPDATE SET
                status = $5,
                attempts = $6,
                started_at = $9,
                completed_at = $10,
                error = $11,
                result = $12
            "#,
            job.id,
            job.job_type,
            job.payload,
            job.priority as i32,
            job.status.as_str(),
            job.attempts as i32,
            job.max_attempts as i32,
            job.created_at,
            job.started_at,
            job.completed_at,
            job.error,
            job.result
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_status(&self, id: Uuid, status: JobStatus) -> anyhow::Result<()> {
        sqlx::query!(
            r#"UPDATE jobs SET status = $2 WHERE id = $1"#,
            id,
            status.as_str()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn count_by_status(&self, status: JobStatus) -> anyhow::Result<i64> {
        let row = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM jobs WHERE status = $1"#,
            status.as_str()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count.unwrap_or(0))
    }
}

fn parse_status(s: &str) -> JobStatus {
    match s {
        "pending" => JobStatus::Pending,
        "running" => JobStatus::Running,
        "completed" => JobStatus::Completed,
        "failed" => JobStatus::Failed,
        "retrying" => JobStatus::Retrying,
        "cancelled" => JobStatus::Cancelled,
        _ => JobStatus::Pending,
    }
}
