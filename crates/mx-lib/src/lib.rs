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
pub mod corpus;
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
// Other crates get the fixtures through the `test-support` feature; this
// crate's own `#[cfg(test)]` modules get them unconditionally, since the
// fixtures' dependencies (`wiremock`, `tempfile`) are already dev-deps here.
// Neither path exists in a release build.
#[cfg(any(feature = "test-support", test))]
pub mod test_support;
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

// scratch: deliberate gate-proof break — DO NOT MERGE.
// `format!("{}", &str)` is `clippy::useless_format`, so
// `cargo clippy --workspace --all-targets -- -D warnings` fails. Proves ci.yml's
// `lint` job gates on clippy. Branch is deleted after the run URL is captured.
pub fn scratch_gate_proof_clippy_warning() -> String {
    format!("{}", "scratch: deliberate gate-proof break")
}
