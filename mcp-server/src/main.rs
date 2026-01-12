//! MechCrate MCP Server
//!
//! This MCP server enables LLMs to interact with MechCrate projects,
//! providing tools for project management, service orchestration,
//! infrastructure configuration, and documentation retrieval via RAG.

mod error;
mod mcp;
mod mx;
mod project;
mod rag;
mod tools;
mod weaviate;

use clap::Parser;
use tracing::{info, warn, Level};
use tracing_subscriber::{fmt, EnvFilter};

use crate::mcp::server::McpServer;
use crate::weaviate::WeaviateManager;

#[derive(Parser, Debug)]
#[command(name = "mx-mcp")]
#[command(about = "MechCrate MCP Server - LLM-powered project management")]
#[command(version)]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Weaviate endpoint URL (auto-detected if not specified)
    #[arg(long, env = "WEAVIATE_URL")]
    weaviate_url: Option<String>,

    /// MechCrate root directory (auto-detected if not specified)
    #[arg(long, env = "MECH_CRATE_ROOT")]
    mech_crate_root: Option<String>,

    /// Disable auto-start of Weaviate
    #[arg(long)]
    no_auto_start: bool,

    /// Skip Weaviate entirely (RAG will be unavailable)
    #[arg(long)]
    no_rag: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    let level = if args.debug { Level::DEBUG } else { Level::INFO };
    let filter = EnvFilter::from_default_env()
        .add_directive(level.into());
    
    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    info!("Starting MechCrate MCP Server v{}", env!("CARGO_PKG_VERSION"));

    // Determine Weaviate URL
    let weaviate_url = if args.no_rag {
        info!("RAG disabled - Weaviate will not be used");
        "http://localhost:8080".to_string() // Placeholder, won't be used
    } else if let Some(url) = args.weaviate_url {
        // Explicit URL provided
        info!("Using provided Weaviate URL: {}", url);
        url
    } else if !args.no_auto_start {
        // Auto-start Weaviate
        let mcp_dir = detect_mcp_dir(&args.mech_crate_root)?;
        info!("MCP server directory: {:?}", mcp_dir);
        
        match WeaviateManager::new(mcp_dir).await {
            Ok(manager) => {
                if let Err(e) = manager.start().await {
                    warn!("Failed to auto-start Weaviate: {}. RAG may be unavailable.", e);
                }
                manager.url()
            }
            Err(e) => {
                warn!("Failed to initialize Weaviate manager: {}. Using default URL.", e);
                weaviate::get_weaviate_url()
            }
        }
    } else {
        // Use stored or default URL
        weaviate::get_weaviate_url()
    };

    info!("Weaviate URL: {}", weaviate_url);

    // Create and run the MCP server
    let server = McpServer::new(weaviate_url, args.mech_crate_root)?;
    server.run().await?;

    Ok(())
}

/// Detect the MCP server directory (for docker-compose.yml)
fn detect_mcp_dir(mech_crate_root: &Option<String>) -> anyhow::Result<std::path::PathBuf> {
    // If explicit root provided
    if let Some(root) = mech_crate_root {
        let mcp_dir = std::path::PathBuf::from(root).join("mcp-server");
        if mcp_dir.join("docker-compose.yml").exists() {
            return Ok(mcp_dir);
        }
    }

    // Try to find from current executable location
    if let Ok(exe_path) = std::env::current_exe() {
        // Binary is in mcp-server/target/release/
        if let Some(mcp_dir) = exe_path.parent().and_then(|p| p.parent()).and_then(|p| p.parent()) {
            if mcp_dir.join("docker-compose.yml").exists() {
                return Ok(mcp_dir.to_path_buf());
            }
        }
    }

    // Try common locations
    let candidates = [
        std::env::current_dir().ok(),
        std::env::var("HOME").ok().map(|h| std::path::PathBuf::from(h).join("dev/mech-crate/mcp-server")),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.join("docker-compose.yml").exists() {
            return Ok(candidate);
        }
        // Check if we're in mech-crate root
        let mcp_dir = candidate.join("mcp-server");
        if mcp_dir.join("docker-compose.yml").exists() {
            return Ok(mcp_dir);
        }
    }

    anyhow::bail!("Could not find mcp-server directory with docker-compose.yml")
}
