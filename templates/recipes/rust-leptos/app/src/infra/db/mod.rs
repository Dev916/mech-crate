//! Database Infrastructure
//!
//! PostgreSQL connection pool and repository implementations.

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub type DbPool = PgPool;

/// Create database connection pool
pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .connect(database_url)
        .await
}

/// Check database connection
pub async fn health_check(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map(|_| ())
}

// ─────────────────────────────────────────────────────────────────────────────
// Repository Implementations
// ─────────────────────────────────────────────────────────────────────────────

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::models::User;
use crate::domain::ports::UserRepository;

/// PostgreSQL user repository
pub struct PgUserRepository {
    pool: DbPool,
}

impl PgUserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, anyhow::Error> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT id, email, name, created_at, updated_at FROM users WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, anyhow::Error> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT id, email, name, created_at, updated_at FROM users WHERE email = $1"#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<User>, anyhow::Error> {
        let users = sqlx::query_as!(
            User,
            r#"SELECT id, email, name, created_at, updated_at FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    async fn create(&self, user: &User) -> Result<User, anyhow::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, email, name, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, email, name, created_at, updated_at
            "#,
            user.id,
            user.email,
            user.name,
            user.created_at,
            user.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update(&self, user: &User) -> Result<User, anyhow::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET email = $2, name = $3, updated_at = $4
            WHERE id = $1
            RETURNING id, email, name, created_at, updated_at
            "#,
            user.id,
            user.email,
            user.name,
            user.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, anyhow::Error> {
        let result = sqlx::query!(
            r#"DELETE FROM users WHERE id = $1"#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
