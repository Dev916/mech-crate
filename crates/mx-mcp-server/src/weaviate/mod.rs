//! Weaviate Auto-Start Management
//!
//! Handles automatic startup of Weaviate with dynamic port allocation.

use std::fs;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;
use tracing::{debug, info};

use crate::error::{McpError, McpResult};

/// Port ranges for Weaviate
const HTTP_PORT_START: u16 = 8080;
const HTTP_PORT_END: u16 = 8179;
const GRPC_PORT_START: u16 = 50051;
const GRPC_PORT_END: u16 = 50150;

/// State directory for MCP
fn state_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".mech-crate/mcp")
}

/// Path to stored HTTP port
fn http_port_file() -> PathBuf {
    state_dir().join(".weaviate-http-port")
}

/// Path to stored gRPC port
fn grpc_port_file() -> PathBuf {
    state_dir().join(".weaviate-grpc-port")
}

/// Check if a port is free
async fn is_port_free(port: u16) -> bool {
    tokio::net::TcpListener::bind(("127.0.0.1", port)).await.is_ok()
}

/// Find a free port in the given range
async fn find_free_port(start: u16, end: u16) -> Option<u16> {
    for port in start..=end {
        if is_port_free(port).await {
            return Some(port);
        }
    }
    None
}

/// Check if Weaviate is ready at the given URL
pub async fn check_weaviate_ready(url: &str) -> bool {
    let client = reqwest::Client::new();
    match client.get(format!("{}/v1/.well-known/ready", url)).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

/// Get the stored Weaviate HTTP port
pub fn get_stored_port() -> Option<u16> {
    let port_file = http_port_file();
    if port_file.exists() {
        fs::read_to_string(port_file)
            .ok()
            .and_then(|s| s.trim().parse().ok())
    } else {
        None
    }
}

/// Get the Weaviate URL from stored port or default
pub fn get_weaviate_url() -> String {
    let port = get_stored_port().unwrap_or(8080);
    format!("http://localhost:{}", port)
}

/// Weaviate manager for auto-start
pub struct WeaviateManager {
    mcp_dir: PathBuf,
    http_port: u16,
    grpc_port: u16,
}

impl WeaviateManager {
    /// Create a new manager, resolving ports
    pub async fn new(mcp_dir: PathBuf) -> McpResult<Self> {
        // Ensure state directory exists
        let state = state_dir();
        fs::create_dir_all(&state).map_err(|e| McpError::Io(e))?;

        // Try to use stored ports if Weaviate is already running
        if let Some(http_port) = get_stored_port() {
            let url = format!("http://localhost:{}", http_port);
            if check_weaviate_ready(&url).await {
                info!("Weaviate already running on port {}", http_port);
                let grpc_port = fs::read_to_string(grpc_port_file())
                    .ok()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(50051);
                return Ok(Self {
                    mcp_dir,
                    http_port,
                    grpc_port,
                });
            }
        }

        // Allocate new ports
        let http_port = find_free_port(HTTP_PORT_START, HTTP_PORT_END)
            .await
            .ok_or_else(|| McpError::Weaviate(format!(
                "No free ports in HTTP range {}-{}", HTTP_PORT_START, HTTP_PORT_END
            )))?;

        let grpc_port = find_free_port(GRPC_PORT_START, GRPC_PORT_END)
            .await
            .ok_or_else(|| McpError::Weaviate(format!(
                "No free ports in gRPC range {}-{}", GRPC_PORT_START, GRPC_PORT_END
            )))?;

        // Store the ports
        fs::write(http_port_file(), http_port.to_string())
            .map_err(|e| McpError::Io(e))?;
        fs::write(grpc_port_file(), grpc_port.to_string())
            .map_err(|e| McpError::Io(e))?;

        info!("Allocated ports: HTTP={}, gRPC={}", http_port, grpc_port);

        Ok(Self {
            mcp_dir,
            http_port,
            grpc_port,
        })
    }

    /// Get the Weaviate URL
    pub fn url(&self) -> String {
        format!("http://localhost:{}", self.http_port)
    }

    /// Check if Weaviate is ready
    pub async fn is_ready(&self) -> bool {
        check_weaviate_ready(&self.url()).await
    }

    /// Start Weaviate using docker compose
    pub async fn start(&self) -> McpResult<()> {
        // Check if already running
        if self.is_ready().await {
            info!("Weaviate already running at {}", self.url());
            return Ok(());
        }

        info!("Starting Weaviate on port {}...", self.http_port);

        // Run docker compose up
        let output = Command::new("docker")
            .args(["compose", "-p", "mx-mcp-rag", "up", "-d"])
            .current_dir(&self.mcp_dir)
            .env("MX_MCP_WEAVIATE_PORT", self.http_port.to_string())
            .env("MX_MCP_GRPC_PORT", self.grpc_port.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| McpError::CommandFailed(format!("Failed to run docker compose: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(McpError::Weaviate(format!(
                "docker compose failed: {}",
                stderr
            )));
        }

        // Wait for Weaviate to be ready
        info!("Waiting for Weaviate to be ready...");
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 60; // 2 minutes max

        while attempts < MAX_ATTEMPTS {
            if self.is_ready().await {
                info!("Weaviate is ready at {}", self.url());
                return Ok(());
            }
            sleep(Duration::from_secs(2)).await;
            attempts += 1;
            debug!("Waiting for Weaviate... attempt {}/{}", attempts, MAX_ATTEMPTS);
        }

        Err(McpError::Weaviate(
            "Weaviate startup timed out after 2 minutes".to_string(),
        ))
    }
}
