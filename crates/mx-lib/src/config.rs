//! Global MechCrate configuration management
//!
//! Handles the `~/.mech-crate/` directory structure:
//! - config/          - Configuration files
//! - recipes/         - Cached recipes from Unyform
//! - router/          - Global Traefik router installation
//! - mcp/             - MCP server state

use std::path::PathBuf;

use crate::error::Result;

/// Global MechCrate configuration
#[derive(Debug, Clone)]
pub struct MechCrateConfig {
    /// Root directory (~/.mech-crate/)
    pub root: PathBuf,
}

impl MechCrateConfig {
    /// Create a new config with the default root directory
    pub fn new() -> Result<Self> {
        let root = Self::default_root()?;
        Ok(Self { root })
    }

    /// Get the default root directory (~/.mech-crate/)
    pub fn default_root() -> Result<PathBuf> {
        dirs::home_dir()
            .map(|h| h.join(".mech-crate"))
            .ok_or_else(|| crate::error::Error::Config("Could not determine home directory".into()))
    }

    /// Get the config directory (~/.mech-crate/config/)
    pub fn config_dir(&self) -> PathBuf {
        self.root.join("config")
    }

    /// Get the recipes cache directory (~/.mech-crate/recipes/)
    pub fn recipes_dir(&self) -> PathBuf {
        self.root.join("recipes")
    }

    /// Get the router directory (~/.mech-crate/router/)
    pub fn router_dir(&self) -> PathBuf {
        self.root.join("router")
    }

    /// Get the MCP state directory (~/.mech-crate/mcp/)
    pub fn mcp_dir(&self) -> PathBuf {
        self.root.join("mcp")
    }

    /// Get the infrastructure config directory (~/.mech-crate/config/infra/)
    pub fn infra_dir(&self) -> PathBuf {
        self.config_dir().join("infra")
    }

    /// Get the Unyform config directory (~/.mech-crate/config/unyform/)
    pub fn unyform_dir(&self) -> PathBuf {
        self.config_dir().join("unyform")
    }

    /// Ensure all required directories exist
    pub fn ensure_dirs(&self) -> Result<()> {
        let dirs = [
            &self.root,
            &self.config_dir(),
            &self.recipes_dir(),
            &self.router_dir(),
            &self.mcp_dir(),
            &self.infra_dir(),
            &self.unyform_dir(),
        ];

        for dir in dirs {
            if !dir.exists() {
                std::fs::create_dir_all(dir)?;
            }
        }

        Ok(())
    }
}

impl Default for MechCrateConfig {
    fn default() -> Self {
        Self::new().expect("Failed to create default config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dirs() {
        let config = MechCrateConfig::new().unwrap();
        assert!(config.root.ends_with(".mech-crate"));
        assert!(config.config_dir().ends_with("config"));
        assert!(config.recipes_dir().ends_with("recipes"));
    }
}
