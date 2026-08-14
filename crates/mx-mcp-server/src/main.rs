//! MechCrate MCP Server

use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

use mx_mcp_server::mcp::server::McpServer;

#[derive(Parser, Debug)]
#[command(name = "mx-mcp")]
#[command(about = "MechCrate MCP Server - LLM-powered project management")]
#[command(version)]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// MechCrate root directory (auto-detected if not specified)
    #[arg(long, env = "MECH_CRATE_ROOT")]
    mech_crate_root: Option<String>,

    /// Skip the techniques corpus (RAG tools will report offline)
    #[arg(long)]
    no_rag: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let level = if args.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let filter = EnvFilter::from_default_env().add_directive(level.into());
    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    info!(
        "Starting MechCrate MCP Server v{}",
        env!("CARGO_PKG_VERSION")
    );

    let server = McpServer::new(args.mech_crate_root, args.no_rag)?;
    server.run().await?;

    Ok(())
}
