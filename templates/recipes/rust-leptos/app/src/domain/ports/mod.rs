//! Domain Ports
//!
//! Interfaces (traits) for external dependencies.
//! Infrastructure adapters implement these ports.
//! See: appendix-algebraic-effects-optics.md

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::models::{CreateUser, UpdateUser, User, UserError};

/// Clock port - abstracts time for testability
pub trait Clock: Send + Sync {
    fn now(&self) -> chrono::DateTime<chrono::Utc>;
}

/// User repository port
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, anyhow::Error>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, anyhow::Error>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<User>, anyhow::Error>;
    async fn create(&self, user: &User) -> Result<User, anyhow::Error>;
    async fn update(&self, user: &User) -> Result<User, anyhow::Error>;
    async fn delete(&self, id: Uuid) -> Result<bool, anyhow::Error>;
}

/// Cache port
#[async_trait]
pub trait Cache: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>, anyhow::Error>;
    async fn set(&self, key: &str, value: &str, ttl_seconds: Option<u64>) -> Result<(), anyhow::Error>;
    async fn delete(&self, key: &str) -> Result<bool, anyhow::Error>;
}

/// Event publisher port
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: DomainEvent) -> Result<(), anyhow::Error>;
}

/// Domain events
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DomainEvent {
    UserCreated { user_id: Uuid },
    UserUpdated { user_id: Uuid },
    UserDeleted { user_id: Uuid },
}

/// Real clock implementation
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

/// Fake clock for testing
#[cfg(test)]
pub struct FakeClock {
    time: std::sync::Mutex<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
impl FakeClock {
    pub fn new(time: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            time: std::sync::Mutex::new(time),
        }
    }

    pub fn advance(&self, duration: chrono::Duration) {
        let mut time = self.time.lock().unwrap();
        *time = *time + duration;
    }
}

#[cfg(test)]
impl Clock for FakeClock {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        *self.time.lock().unwrap()
    }
}
