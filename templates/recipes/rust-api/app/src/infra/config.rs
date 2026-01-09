//! Application configuration

use std::env;

/// Application configuration loaded from environment
#[derive(Debug, Clone)]
pub struct Config {
    /// Server port
    pub port: u16,
    
    /// Database URL
    pub database_url: Option<String>,
    
    /// Environment (development, production)
    pub environment: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            database_url: env::var("DATABASE_URL").ok(),
            environment: env::var("RUST_ENV")
                .unwrap_or_else(|_| "development".to_string()),
        })
    }
    
    /// Check if running in production
    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }
}
