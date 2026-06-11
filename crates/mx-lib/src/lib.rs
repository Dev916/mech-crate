//! MechCrate shared library
//!
//! This library provides the core functionality for MechCrate:
//! - Project detection and analysis
//! - Recipe management and installation
//! - Template processing
//! - Infrastructure configuration
//! - Docker/Compose integration
//! - Unyform API client

pub mod config;
pub mod docker;
pub mod env;
pub mod error;
pub mod infra;
pub mod mcp;
pub mod paths;
pub mod project;
pub mod recipe;
pub mod router;
pub mod template;
pub mod unyform;
pub mod upgrade;

pub use config::MechCrateConfig;
pub use env::{ensure_full_path, ensure_path};
pub use error::{Error, Result};
pub use mcp::McpManager;
pub use paths::{
    home_dir, is_initialized, mech_crate_root, recipes_dir, recorded_source_root, save_source_root,
    source_templates_dir, templates_dir,
};
pub use project::{Project, ProjectDetector};
