//! Server Configuration

use std::env;

/// Server configuration loaded from environment
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub database_url: String,
    pub redis_url: String,
    pub server_addr: String,
    pub server_port: u16,
    pub environment: Environment,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Production,
    Test,
}

impl ServerConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://{{SERVICE_SLUG}}:secret@db:5432/{{SERVICE_SLUG}}".to_string()),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://redis:6379".to_string()),
            server_addr: env::var("SERVER_ADDR")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            environment: match env::var("RUST_ENV").as_deref() {
                Ok("production") => Environment::Production,
                Ok("test") => Environment::Test,
                _ => Environment::Development,
            },
        }
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }
}
