//! MCP server management
//!
//! Handles Weaviate backend and MCP server orchestration.

use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use crate::config::MechCrateConfig;
use crate::docker::Compose;
use crate::error::{Error, Result};
use crate::paths;

/// Port file names
const HTTP_PORT_FILE: &str = ".weaviate-http-port";
const GRPC_PORT_FILE: &str = ".weaviate-grpc-port";

/// Default port ranges
const HTTP_PORT_RANGE: (u16, u16) = (8080, 8179);
const GRPC_PORT_RANGE: (u16, u16) = (50051, 50150);

/// MCP project name for docker-compose
const MCP_PROJECT_NAME: &str = "mx-mcp-rag";

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

    /// Get ingest binary path
    pub fn ingest_binary(&self) -> Result<PathBuf> {
        let source_dir = self.source_dir()?;
        Ok(source_dir.join("target").join("release").join("mx-ingest"))
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

        let binary_modified = binary
            .metadata()
            .and_then(|m| m.modified())
            .ok();

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

    /// Find a free port in the given range
    fn find_free_port(&self, start: u16, end: u16) -> Result<u16> {
        for port in start..=end {
            if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)) {
                drop(listener);
                return Ok(port);
            }
        }
        Err(Error::Other(format!(
            "No free port found in range {}-{}",
            start, end
        )))
    }

    /// Allocate or retrieve ports for Weaviate
    pub fn allocate_ports(&self) -> Result<(u16, u16)> {
        let state_dir = self.state_dir();
        std::fs::create_dir_all(&state_dir)?;

        let http_port_file = state_dir.join(HTTP_PORT_FILE);
        let grpc_port_file = state_dir.join(GRPC_PORT_FILE);

        // Check if we have stored ports
        if http_port_file.exists() && grpc_port_file.exists() {
            let http_port: u16 = std::fs::read_to_string(&http_port_file)?
                .trim()
                .parse()
                .map_err(|_| Error::Config("Invalid HTTP port in state file".into()))?;
            let grpc_port: u16 = std::fs::read_to_string(&grpc_port_file)?
                .trim()
                .parse()
                .map_err(|_| Error::Config("Invalid gRPC port in state file".into()))?;

            // Check if Weaviate is already running on these ports
            if self.is_weaviate_ready_at(http_port) {
                return Ok((http_port, grpc_port));
            }

            // Check if ports are still free
            let http_free = TcpListener::bind(("127.0.0.1", http_port)).is_ok();
            let grpc_free = TcpListener::bind(("127.0.0.1", grpc_port)).is_ok();

            if http_free && grpc_free {
                return Ok((http_port, grpc_port));
            }
        }

        // Allocate new ports
        let http_port = self.find_free_port(HTTP_PORT_RANGE.0, HTTP_PORT_RANGE.1)?;
        let grpc_port = self.find_free_port(GRPC_PORT_RANGE.0, GRPC_PORT_RANGE.1)?;

        // Save ports
        std::fs::write(&http_port_file, http_port.to_string())?;
        std::fs::write(&grpc_port_file, grpc_port.to_string())?;

        Ok((http_port, grpc_port))
    }

    /// Get the current HTTP port (if allocated)
    pub fn http_port(&self) -> Option<u16> {
        let port_file = self.state_dir().join(HTTP_PORT_FILE);
        std::fs::read_to_string(port_file)
            .ok()
            .and_then(|s| s.trim().parse().ok())
    }

    /// Get Weaviate URL
    pub fn weaviate_url(&self) -> String {
        let port = self.http_port().unwrap_or(8080);
        format!("http://localhost:{}", port)
    }

    /// Check if Weaviate is ready at a given port
    fn is_weaviate_ready_at(&self, port: u16) -> bool {
        let url = format!("http://localhost:{}/v1/.well-known/ready", port);
        
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .ok();

        client.and_then(|c| c.get(&url).send().ok())
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Check if Weaviate is running
    pub fn is_weaviate_running(&self) -> bool {
        if let Some(port) = self.http_port() {
            self.is_weaviate_ready_at(port)
        } else {
            false
        }
    }

    /// Start Weaviate
    pub fn start_weaviate(&self) -> Result<()> {
        let (http_port, grpc_port) = self.allocate_ports()?;

        // Check if already running
        if self.is_weaviate_ready_at(http_port) {
            return Ok(());
        }

        let source_dir = self.source_dir()?;
        let compose_file = source_dir.join("docker-compose.yml");

        if !compose_file.exists() {
            return Err(Error::Config(format!(
                "Weaviate compose file not found at {}",
                compose_file.display()
            )));
        }

        // Start with docker compose
        let output = Command::new("docker")
            .args(["compose", "-f", compose_file.to_str().unwrap(), "-p", MCP_PROJECT_NAME, "up", "-d"])
            .env("MX_MCP_WEAVIATE_PORT", http_port.to_string())
            .env("MX_MCP_GRPC_PORT", grpc_port.to_string())
            .current_dir(&source_dir)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to start Weaviate: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!("Failed to start Weaviate: {}", stderr)));
        }

        // Wait for ready
        for _ in 0..60 {
            std::thread::sleep(Duration::from_secs(2));
            if self.is_weaviate_ready_at(http_port) {
                return Ok(());
            }
        }

        Err(Error::CommandFailed("Weaviate startup timed out".into()))
    }

    /// Stop Weaviate
    pub fn stop_weaviate(&self) -> Result<()> {
        let source_dir = self.source_dir()?;
        let compose_file = source_dir.join("docker-compose.yml");

        if !compose_file.exists() {
            return Ok(());
        }

        let output = Command::new("docker")
            .args(["compose", "-f", compose_file.to_str().unwrap(), "-p", MCP_PROJECT_NAME, "down"])
            .current_dir(&source_dir)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to stop Weaviate: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!("Failed to stop Weaviate: {}", stderr)));
        }

        Ok(())
    }

    /// Get Weaviate logs
    pub fn weaviate_logs(&self, follow: bool) -> Result<std::process::Output> {
        let source_dir = self.source_dir()?;
        let compose_file = source_dir.join("docker-compose.yml");

        let mut args = vec!["compose", "-f", compose_file.to_str().unwrap(), "-p", MCP_PROJECT_NAME, "logs"];
        if follow {
            args.push("-f");
        }

        Command::new("docker")
            .args(&args)
            .current_dir(&source_dir)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to get logs: {}", e)))
    }

    /// Get Weaviate status
    pub fn weaviate_status(&self) -> Result<std::process::Output> {
        let source_dir = self.source_dir()?;
        let compose_file = source_dir.join("docker-compose.yml");

        Command::new("docker")
            .args(["compose", "-f", compose_file.to_str().unwrap(), "-p", MCP_PROJECT_NAME, "ps"])
            .current_dir(&source_dir)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to get status: {}", e)))
    }

    /// Run document ingestion
    pub fn ingest(&self, clear: bool) -> Result<()> {
        self.ensure_binary()?;

        // Ensure Weaviate is running
        if !self.is_weaviate_running() {
            self.start_weaviate()?;
        }

        let ingest_bin = self.ingest_binary()?;
        if !ingest_bin.exists() {
            return Err(Error::CommandFailed("Ingest binary not found. Run 'mx mcp build' first.".into()));
        }

        let mech_root = paths::source_templates_dir()?
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| Error::Config("Could not find MechCrate root".into()))?;

        let mut cmd = Command::new(&ingest_bin);
        cmd.arg("--weaviate-url").arg(self.weaviate_url());
        cmd.arg("--mech-crate-root").arg(&mech_root);

        if clear {
            cmd.arg("--clear");
        }

        let output = cmd.output()
            .map_err(|e| Error::CommandFailed(format!("Failed to run ingest: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!("Ingestion failed: {}", stderr)));
        }

        Ok(())
    }

    /// Get MCP server info
    pub fn info(&self) -> McpInfo {
        let http_port = self.http_port();
        let mcp_binary = self.mcp_binary().ok();
        let ingest_binary = self.ingest_binary().ok();
        let source_dir = self.source_dir().ok();

        McpInfo {
            state_dir: self.state_dir(),
            source_dir,
            mcp_binary,
            ingest_binary,
            weaviate_url: self.weaviate_url(),
            http_port,
            http_port_range: HTTP_PORT_RANGE,
            grpc_port_range: GRPC_PORT_RANGE,
            binary_built: self.mcp_binary().map(|b| b.exists()).unwrap_or(false),
            weaviate_running: self.is_weaviate_running(),
        }
    }

    /// Generate MCP client configuration
    pub fn generate_config(&self) -> Result<String> {
        let mcp_binary = self.mcp_binary()?;
        let weaviate_url = self.weaviate_url();
        let mech_root = paths::source_templates_dir()?
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| Error::Config("Could not find MechCrate root".into()))?;

        // Create wrapper script
        let wrapper_path = self.state_dir().join("mx-mcp-wrapper.sh");
        let wrapper_content = format!(r#"#!/bin/bash
# MechCrate MCP Server Wrapper
# Auto-starts Weaviate and runs the MCP server

set -e

MECH_CRATE_ROOT="{mech_root}"
source "${{MECH_CRATE_ROOT}}/bin/lib/common.sh"
source "${{MECH_CRATE_ROOT}}/bin/lib/mcp.sh"

# Start Weaviate if not running
if ! _mcp_is_weaviate_running; then
    _mcp_start_weaviate >/dev/null 2>&1 || true
fi

weaviate_url=$(_mcp_get_weaviate_url)

exec "{mcp_binary}" --weaviate-url "$weaviate_url" "$@"
"#, mech_root = mech_root.display(), mcp_binary = mcp_binary.display());

        std::fs::create_dir_all(self.state_dir())?;
        std::fs::write(&wrapper_path, &wrapper_content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&wrapper_path, std::fs::Permissions::from_mode(0o755))?;
        }

        // Generate JSON config
        let config = format!(r#"{{
  "mcpServers": {{
    "mechcrate": {{
      "command": "{}",
      "env": {{
        "MECH_CRATE_ROOT": "{}"
      }}
    }}
  }}
}}"#, wrapper_path.display(), mech_root.display());

        Ok(config)
    }
}

/// MCP server information
#[derive(Debug)]
pub struct McpInfo {
    pub state_dir: PathBuf,
    pub source_dir: Option<PathBuf>,
    pub mcp_binary: Option<PathBuf>,
    pub ingest_binary: Option<PathBuf>,
    pub weaviate_url: String,
    pub http_port: Option<u16>,
    pub http_port_range: (u16, u16),
    pub grpc_port_range: (u16, u16),
    pub binary_built: bool,
    pub weaviate_running: bool,
}

impl Default for McpManager {
    fn default() -> Self {
        Self::new(MechCrateConfig::default())
    }
}
