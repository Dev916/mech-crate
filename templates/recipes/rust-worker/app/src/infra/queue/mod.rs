//! Redis Queue Infrastructure

use deadpool_redis::{Config, Pool, Runtime};

pub type RedisPool = Pool;

/// Create Redis connection pool
pub fn create_pool(redis_url: &str) -> anyhow::Result<RedisPool> {
    let config = Config::from_url(redis_url);
    let pool = config.create_pool(Some(Runtime::Tokio1))?;
    Ok(pool)
}

/// Health check
pub async fn health_check(pool: &RedisPool) -> anyhow::Result<()> {
    let _conn = pool.get().await?;
    Ok(())
}
