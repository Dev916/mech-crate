//! Infrastructure configuration
//!
//! Manages infrastructure provider configurations (Cloudflare, AWS, DigitalOcean, Hetzner).

mod config;

pub use config::{
    body_is_linked, clear_link_flag, parse_env_body, set_link_flag, CredentialScope, InfraConfig,
    InfraProvider, LinkOutcome, Resolution, ResolvedKey, UnlinkOutcome, DEPRECATED_LINK_MARKER,
    LINK_FLAG,
};
