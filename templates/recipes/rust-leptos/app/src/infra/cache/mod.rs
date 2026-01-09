//! Redis Cache Infrastructure
//!
//! Redis connection pool and cache implementation.

use deadpool_redis::{Config, Pool, Runtime};
use redis::AsyncCommands;

pub type RedisPool = Pool;

/// Create Redis connection pool
pub fn create_pool(redis_url: &str) -> Result<RedisPool, deadpool_redis::CreatePoolError> {
    let config = Config::from_url(redis_url);
    config.create_pool(Some(Runtime::Tokio1))
}

/// Check Redis connection
pub async fn health_check(pool: &RedisPool) -> Result<(), deadpool_redis::PoolError> {
    let _conn = pool.get().await?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Cache Implementation
// ─────────────────────────────────────────────────────────────────────────────

use async_trait::async_trait;
use crate::domain::ports::Cache;

/// Redis cache adapter
pub struct RedisCache {
    pool: RedisPool,
    prefix: String,
}

impl RedisCache {
    pub fn new(pool: RedisPool, prefix: &str) -> Self {
        Self {
            pool,
            prefix: prefix.to_string(),
        }
    }

    fn key(&self, key: &str) -> String {
        format!("{}:{}", self.prefix, key)
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn get(&self, key: &str) -> Result<Option<String>, anyhow::Error> {
        let mut conn = self.pool.get().await?;
        let value: Option<String> = conn.get(self.key(key)).await?;
        Ok(value)
    }

    async fn set(&self, key: &str, value: &str, ttl_seconds: Option<u64>) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get().await?;
        let key = self.key(key);

        if let Some(ttl) = ttl_seconds {
            conn.set_ex(&key, value, ttl).await?;
        } else {
            conn.set(&key, value).await?;
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool, anyhow::Error> {
        let mut conn = self.pool.get().await?;
        let deleted: i64 = conn.del(self.key(key)).await?;
        Ok(deleted > 0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Session Store
// ─────────────────────────────────────────────────────────────────────────────

use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

/// Session store backed by Redis
pub struct SessionStore {
    pool: RedisPool,
    prefix: String,
    default_ttl: u64,
}

impl SessionStore {
    pub fn new(pool: RedisPool, prefix: &str, default_ttl: u64) -> Self {
        Self {
            pool,
            prefix: prefix.to_string(),
            default_ttl,
        }
    }

    fn key(&self, session_id: Uuid) -> String {
        format!("{}:session:{}", self.prefix, session_id)
    }

    /// Store session data
    pub async fn set<T: Serialize>(&self, session_id: Uuid, data: &T) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get().await?;
        let value = serde_json::to_string(data)?;
        conn.set_ex(self.key(session_id), value, self.default_ttl).await?;
        Ok(())
    }

    /// Get session data
    pub async fn get<T: DeserializeOwned>(&self, session_id: Uuid) -> Result<Option<T>, anyhow::Error> {
        let mut conn = self.pool.get().await?;
        let value: Option<String> = conn.get(self.key(session_id)).await?;

        match value {
            Some(v) => Ok(Some(serde_json::from_str(&v)?)),
            None => Ok(None),
        }
    }

    /// Refresh session TTL
    pub async fn refresh(&self, session_id: Uuid) -> Result<bool, anyhow::Error> {
        let mut conn = self.pool.get().await?;
        let result: bool = conn.expire(self.key(session_id), self.default_ttl as i64).await?;
        Ok(result)
    }

    /// Delete session
    pub async fn delete(&self, session_id: Uuid) -> Result<bool, anyhow::Error> {
        let mut conn = self.pool.get().await?;
        let deleted: i64 = conn.del(self.key(session_id)).await?;
        Ok(deleted > 0)
    }
}
