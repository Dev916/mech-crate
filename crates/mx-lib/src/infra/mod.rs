//! Infrastructure configuration
//!
//! Manages infrastructure provider configurations (Cloudflare, AWS, DigitalOcean, Hetzner).

mod config;

pub use config::{InfraConfig, InfraProvider};
