//! MCP server management
//!
//! Handles building and configuring the MechCrate MCP server binary.

use std::path::{Path, PathBuf};
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
        let target_override = std::env::var_os("CARGO_TARGET_DIR").map(PathBuf::from);
        Ok(resolve_binary_path(&source_dir, target_override.as_deref()))
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

/// Name of the MCP server binary artifact.
const MCP_BIN: &str = "mx-mcp";

/// Resolve where cargo puts the `mx-mcp` release artifact for `source_dir`.
///
/// `crates/mx-mcp-server` is a workspace member, so cargo writes the artifact to
/// the *workspace* `target/release/`, not to a per-crate one. Reading the
/// per-crate path made `needs_build()` permanently true, so `mx mcp config`
/// reported "not built" even straight after a successful `mx mcp build`.
///
/// Resolution order:
/// 1. `CARGO_TARGET_DIR` when the caller sets it,
/// 2. the workspace target dir (nearest ancestor `Cargo.toml` with a
///    `[workspace]` table), if the artifact is there,
/// 3. the per-crate target dir, if the artifact is there — standalone checkouts,
/// 4. otherwise the canonical candidate: workspace if one was found, else
///    per-crate, so "missing" is reported against the path cargo would use.
fn resolve_binary_path(source_dir: &Path, cargo_target_dir: Option<&Path>) -> PathBuf {
    let release = |target: &Path| target.join("release").join(MCP_BIN);

    if let Some(target) = cargo_target_dir {
        return release(target);
    }

    let per_crate = release(&source_dir.join("target"));
    let workspace = workspace_root(source_dir).map(|ws| release(&ws.join("target")));

    if let Some(ws_bin) = &workspace {
        if ws_bin.exists() {
            return ws_bin.clone();
        }
    }
    if per_crate.exists() {
        return per_crate;
    }
    workspace.unwrap_or(per_crate)
}

/// Nearest ancestor of `start` (inclusive) whose `Cargo.toml` declares a workspace.
fn workspace_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        let manifest = dir.join("Cargo.toml");
        if let Ok(text) = std::fs::read_to_string(&manifest) {
            if text
                .lines()
                .any(|l| l.trim_start().starts_with("[workspace]"))
            {
                return Some(dir.to_path_buf());
            }
        }
        current = dir.parent();
    }
    None
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

#[cfg(test)]
mod tests {
    use super::*;

    /// `<root>/Cargo.toml` declares a workspace, `<root>/crates/mx-mcp-server`
    /// is the member — the layout this repo actually ships.
    fn workspace_layout() -> tempfile::TempDir {
        let root = tempfile::tempdir().expect("setup: tempdir");
        std::fs::write(
            root.path().join("Cargo.toml"),
            "[workspace]\nresolver = \"2\"\nmembers = [\"crates/mx-mcp-server\"]\n",
        )
        .expect("setup: write workspace manifest");
        let member = root.path().join("crates/mx-mcp-server");
        std::fs::create_dir_all(&member).expect("setup: create member dir");
        std::fs::write(
            member.join("Cargo.toml"),
            "[package]\nname = \"mx-mcp-server\"\n",
        )
        .expect("setup: write member manifest");
        root
    }

    fn touch(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).expect("setup: create parent");
        std::fs::write(path, b"#!/bin/sh\n").expect("setup: write artifact");
    }

    /// bd:mech-crate-ads — cargo writes a workspace member's artifact to the
    /// *workspace* target dir. Reading the per-crate path made `needs_build()`
    /// permanently true, so `mx mcp config` always reported "not built".
    #[test]
    fn workspace_artifact_is_found_at_the_workspace_target_dir() {
        let root = workspace_layout();
        let member = root.path().join("crates/mx-mcp-server");
        let expected = root.path().join("target/release/mx-mcp");
        touch(&expected);

        assert_eq!(resolve_binary_path(&member, None), expected);
    }

    /// Standalone (non-workspace) checkouts keep working.
    #[test]
    fn per_crate_artifact_is_the_fallback() {
        let crate_dir = tempfile::tempdir().expect("setup: tempdir");
        std::fs::write(
            crate_dir.path().join("Cargo.toml"),
            "[package]\nname = \"mx-mcp-server\"\n",
        )
        .expect("setup: write manifest");
        let expected = crate_dir.path().join("target/release/mx-mcp");
        touch(&expected);

        assert_eq!(resolve_binary_path(crate_dir.path(), None), expected);
    }

    /// A workspace manifest above a crate that nevertheless has its own built
    /// artifact: prefer the one that exists.
    #[test]
    fn per_crate_artifact_wins_when_the_workspace_one_is_absent() {
        let root = workspace_layout();
        let member = root.path().join("crates/mx-mcp-server");
        let expected = member.join("target/release/mx-mcp");
        touch(&expected);

        assert_eq!(resolve_binary_path(&member, None), expected);
    }

    /// Nothing built yet: report against the path cargo would write, so the
    /// "run mx mcp build" hint names the right file.
    #[test]
    fn unbuilt_workspace_resolves_to_the_workspace_candidate() {
        let root = workspace_layout();
        let member = root.path().join("crates/mx-mcp-server");

        assert_eq!(
            resolve_binary_path(&member, None),
            root.path().join("target/release/mx-mcp")
        );
    }

    /// `CARGO_TARGET_DIR` wins outright — cargo honours it, so must we.
    #[test]
    fn cargo_target_dir_override_wins() {
        let root = workspace_layout();
        let member = root.path().join("crates/mx-mcp-server");
        touch(&root.path().join("target/release/mx-mcp"));
        let custom = root.path().join("elsewhere");

        assert_eq!(
            resolve_binary_path(&member, Some(&custom)),
            custom.join("release/mx-mcp")
        );
    }

    #[test]
    fn workspace_root_ignores_plain_package_manifests() {
        let root = workspace_layout();
        let member = root.path().join("crates/mx-mcp-server");
        assert_eq!(workspace_root(&member).as_deref(), Some(root.path()));

        let lone = tempfile::tempdir().expect("setup: tempdir");
        std::fs::write(lone.path().join("Cargo.toml"), "[package]\nname = \"x\"\n")
            .expect("setup: write manifest");
        // A stray ancestor workspace outside the tempdir would break this, so
        // only assert the manifest itself is not mistaken for a workspace.
        assert_ne!(workspace_root(lone.path()).as_deref(), Some(lone.path()));
    }
}
