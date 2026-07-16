//! MCP server management
//!
//! Handles building and configuring the MechCrate MCP server binary.

use std::path::PathBuf;
use std::process::Command;

use crate::config::MechCrateConfig;
use crate::error::{Error, Result};
use crate::paths;

/// MCP server manager
#[derive(Debug)]
pub struct McpManager {
    config: MechCrateConfig,
}

impl McpManager {
    /// Create a new MCP manager
    pub fn new(config: MechCrateConfig) -> Self {
        Self { config }
    }

    /// Get the MCP state directory (~/.mech-crate/mcp)
    pub fn state_dir(&self) -> PathBuf {
        self.config.mcp_dir()
    }

    /// Get the MCP server source directory (crates/mx-mcp-server)
    pub fn source_dir(&self) -> Result<PathBuf> {
        // Try new workspace structure first
        let mech_root = paths::source_templates_dir()?
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| Error::Config("Could not find MechCrate root".into()))?;

        let new_path = mech_root.join("crates").join("mx-mcp-server");
        if new_path.exists() {
            return Ok(new_path);
        }

        // Try old location
        let old_path = mech_root.join("mcp-server");
        if old_path.exists() {
            return Ok(old_path);
        }

        Err(Error::Config("MCP server directory not found".into()))
    }

    /// Get MCP binary path
    pub fn mcp_binary(&self) -> Result<PathBuf> {
        let source_dir = self.source_dir()?;
        Ok(source_dir.join("target").join("release").join("mx-mcp"))
    }

    /// Check if MCP binary needs building
    pub fn needs_build(&self) -> bool {
        let binary = match self.mcp_binary() {
            Ok(b) => b,
            Err(_) => return true,
        };

        if !binary.exists() {
            return true;
        }

        // Check if source files are newer than binary
        let source_dir = match self.source_dir() {
            Ok(d) => d,
            Err(_) => return true,
        };

        let src_dir = source_dir.join("src");
        if !src_dir.exists() {
            return false;
        }

        let binary_modified = binary.metadata().and_then(|m| m.modified()).ok();

        if let Some(binary_time) = binary_modified {
            for entry in walkdir::WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
            {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified > binary_time {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Build the MCP server
    pub fn build(&self) -> Result<()> {
        let source_dir = self.source_dir()?;

        let output = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(&source_dir)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to run cargo: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!("Build failed: {}", stderr)));
        }

        Ok(())
    }

    /// Ensure the MCP binary is built
    pub fn ensure_binary(&self) -> Result<()> {
        if self.needs_build() {
            self.build()?;
        }
        Ok(())
    }

    /// Get MCP server info
    pub fn info(&self) -> McpInfo {
        let mcp_binary = self.mcp_binary().ok();
        let source_dir = self.source_dir().ok();

        McpInfo {
            state_dir: self.state_dir(),
            source_dir,
            mcp_binary,
            binary_built: self.mcp_binary().map(|b| b.exists()).unwrap_or(false),
        }
    }

    /// Generate MCP client configuration
    pub fn generate_config(&self) -> Result<String> {
        let mcp_binary = self.mcp_binary()?;
        let mech_root = paths::source_templates_dir()?
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| Error::Config("Could not find MechCrate root".into()))?;

        // Create wrapper script
        let wrapper_path = self.state_dir().join("mx-mcp-wrapper.sh");
        let wrapper_content = format!(
            r#"#!/bin/bash
# MechCrate MCP Server Wrapper

set -e

export MECH_CRATE_ROOT="{mech_root}"

exec "{mcp_binary}" "$@"
"#,
            mech_root = mech_root.display(),
            mcp_binary = mcp_binary.display()
        );

        std::fs::create_dir_all(self.state_dir())?;
        std::fs::write(&wrapper_path, &wrapper_content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&wrapper_path, std::fs::Permissions::from_mode(0o755))?;
        }

        // Generate JSON config
        let config = format!(
            r#"{{
  "mcpServers": {{
    "mechcrate": {{
      "command": "{}",
      "env": {{
        "MECH_CRATE_ROOT": "{}"
      }}
    }}
  }}
}}"#,
            wrapper_path.display(),
            mech_root.display()
        );

        Ok(config)
    }
}

/// MCP server information
#[derive(Debug)]
pub struct McpInfo {
    pub state_dir: PathBuf,
    pub source_dir: Option<PathBuf>,
    pub mcp_binary: Option<PathBuf>,
    pub binary_built: bool,
}

impl Default for McpManager {
    fn default() -> Self {
        Self::new(MechCrateConfig::default())
    }
}
