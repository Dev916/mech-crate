//! `mx mcp` command - MCP server management

use anyhow::Result;
use clap::{Args, Subcommand};
use console::style;

use mx_lib::config::MechCrateConfig;
use mx_lib::mcp::McpManager;

/// MCP server management
#[derive(Args, Debug)]
pub struct McpCommand {
    #[command(subcommand)]
    command: McpSubcommand,
}

#[derive(Subcommand, Debug)]
enum McpSubcommand {
    /// Build MCP server binaries
    Build,
    /// Start Weaviate backend
    Start,
    /// Alias for start
    Up,
    /// Stop Weaviate backend
    Stop,
    /// Alias for stop
    Down,
    /// Show status
    Status,
    /// Alias for status
    Ps,
    /// View logs
    Logs {
        /// Follow logs
        #[arg(short, long)]
        follow: bool,
    },
    /// Ingest documentation into Weaviate
    Ingest {
        /// Clear existing data first
        #[arg(long)]
        clear: bool,
    },
    /// Show MCP client configuration
    Config,
    /// Run MCP server interactively
    Run,
    /// Show MCP info
    Info,
    /// Test MCP server
    Test,
}

impl McpCommand {
    pub async fn run(&self) -> Result<()> {
        let config = MechCrateConfig::new()?;
        let mcp = McpManager::new(config);

        match &self.command {
            McpSubcommand::Build => self.build(&mcp).await,
            McpSubcommand::Start | McpSubcommand::Up => self.start(&mcp).await,
            McpSubcommand::Stop | McpSubcommand::Down => self.stop(&mcp).await,
            McpSubcommand::Status | McpSubcommand::Ps => self.status(&mcp).await,
            McpSubcommand::Logs { follow } => self.logs(&mcp, *follow).await,
            McpSubcommand::Ingest { clear } => self.ingest(&mcp, *clear).await,
            McpSubcommand::Config => self.config(&mcp).await,
            McpSubcommand::Run => self.run_server(&mcp).await,
            McpSubcommand::Info => self.info(&mcp).await,
            McpSubcommand::Test => self.test(&mcp).await,
        }
    }

    async fn build(&self, mcp: &McpManager) -> Result<()> {
        println!("{} Building MCP server...", style("→").cyan().bold());

        mcp.build()?;

        println!("{} MCP server built successfully!", style("✓").green().bold());
        println!();

        if let Ok(bin) = mcp.mcp_binary() {
            println!("  Binaries:");
            println!("    {}", bin.display());
        }
        if let Ok(bin) = mcp.ingest_binary() {
            println!("    {}", bin.display());
        }

        Ok(())
    }

    async fn start(&self, mcp: &McpManager) -> Result<()> {
        if mcp.is_weaviate_running() {
            let url = mcp.weaviate_url();
            println!(
                "{} Weaviate already running at {}",
                style("✓").green(),
                url
            );
            return Ok(());
        }

        println!("{} Starting Weaviate...", style("→").cyan().bold());

        mcp.start_weaviate()?;

        let url = mcp.weaviate_url();
        println!(
            "{} Weaviate is ready at {}",
            style("✓").green().bold(),
            url
        );

        Ok(())
    }

    async fn stop(&self, mcp: &McpManager) -> Result<()> {
        println!("{} Stopping Weaviate...", style("→").cyan().bold());

        mcp.stop_weaviate()?;

        println!("{} Weaviate stopped", style("✓").green().bold());
        Ok(())
    }

    async fn status(&self, mcp: &McpManager) -> Result<()> {
        let info = mcp.info();

        println!("{}", style("Weaviate RAG Backend Status").bold());
        println!("{}", style("─".repeat(40)).dim());
        println!();
        println!(
            "  {} HTTP Port: {}",
            style("•").dim(),
            info.http_port.map(|p| p.to_string()).unwrap_or_else(|| "not allocated".to_string())
        );
        println!("  {} URL: {}", style("•").dim(), info.weaviate_url);
        println!("  {} State Dir: {}", style("•").dim(), info.state_dir.display());
        println!();

        let status = if info.weaviate_running {
            style("● Running").green()
        } else {
            style("○ Not running").red()
        };
        println!("  {}", status);
        println!();

        // Show docker ps output
        if let Ok(output) = mcp.weaviate_status() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                print!("{}", stdout);
            }
        }

        Ok(())
    }

    async fn logs(&self, mcp: &McpManager, follow: bool) -> Result<()> {
        let output = mcp.weaviate_logs(follow)?;
        print!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        Ok(())
    }

    async fn ingest(&self, mcp: &McpManager, clear: bool) -> Result<()> {
        println!("{} Ingesting MechCrate documentation...", style("→").cyan().bold());

        mcp.ingest(clear)?;

        println!("{} Documentation ingested successfully!", style("✓").green().bold());
        Ok(())
    }

    async fn config(&self, mcp: &McpManager) -> Result<()> {
        // Ensure binary is built
        if mcp.needs_build() {
            println!("{} MCP binary not built. Run 'mx mcp build' first.", style("!").yellow());
            return Ok(());
        }

        let config_json = mcp.generate_config()?;

        println!();
        println!("{}", style("MCP Client Configuration").bold());
        println!();
        println!("Add this to your MCP client configuration:");
        println!();
        println!("{}", style("Claude Desktop (~/.claude/claude_desktop_config.json):").cyan());
        println!();
        println!("{}", config_json);
        println!();
        println!("{}", style("Cursor IDE (mcp.json in workspace or ~/.cursor/mcp.json):").cyan());
        println!();
        println!("{}", config_json);
        println!();

        let wrapper_path = mcp.state_dir().join("mx-mcp-wrapper.sh");
        println!(
            "{} Wrapper script: {}",
            style("ℹ").blue(),
            wrapper_path.display()
        );
        println!(
            "{} The wrapper auto-starts Weaviate when the MCP server starts.",
            style("ℹ").blue()
        );

        Ok(())
    }

    async fn run_server(&self, mcp: &McpManager) -> Result<()> {
        // Ensure binary is built
        mcp.ensure_binary()?;

        // Ensure Weaviate is running
        if !mcp.is_weaviate_running() {
            println!("{} Auto-starting Weaviate...", style("→").cyan());
            mcp.start_weaviate()?;
        }

        let mcp_binary = mcp.mcp_binary()?;
        let weaviate_url = mcp.weaviate_url();

        println!(
            "{} Starting MCP server with Weaviate at {}...",
            style("→").cyan().bold(),
            weaviate_url
        );

        // Execute the MCP binary - this replaces the current process
        let err = exec::Command::new(&mcp_binary)
            .args(&["--weaviate-url", &weaviate_url])
            .exec();

        // If we get here, exec failed
        anyhow::bail!("Failed to execute MCP server: {}", err);
    }

    async fn info(&self, mcp: &McpManager) -> Result<()> {
        let info = mcp.info();

        println!("{}", style("MechCrate MCP Server Info").bold());
        println!("{}", style("─".repeat(40)).dim());
        println!();
        println!(
            "  {} MCP Binary: {}",
            style("•").dim(),
            info.mcp_binary.map(|p| p.display().to_string()).unwrap_or_else(|| "not found".to_string())
        );
        println!(
            "  {} Ingest Binary: {}",
            style("•").dim(),
            info.ingest_binary.map(|p| p.display().to_string()).unwrap_or_else(|| "not found".to_string())
        );
        println!(
            "  {} State Dir: {}",
            style("•").dim(),
            info.state_dir.display()
        );
        println!(
            "  {} Source Dir: {}",
            style("•").dim(),
            info.source_dir.map(|p| p.display().to_string()).unwrap_or_else(|| "not found".to_string())
        );
        println!();
        println!(
            "  {} Weaviate URL: {}",
            style("•").dim(),
            info.weaviate_url
        );
        println!(
            "  {} HTTP Port: {}",
            style("•").dim(),
            info.http_port.map(|p| p.to_string()).unwrap_or_else(|| "not allocated".to_string())
        );
        println!(
            "  {} HTTP Range: {}-{}",
            style("•").dim(),
            info.http_port_range.0,
            info.http_port_range.1
        );
        println!(
            "  {} gRPC Range: {}-{}",
            style("•").dim(),
            info.grpc_port_range.0,
            info.grpc_port_range.1
        );
        println!();

        let binary_status = if info.binary_built {
            style("● MCP binary built").green()
        } else {
            style("○ MCP binary not built (run: mx mcp build)").yellow()
        };
        println!("  {}", binary_status);

        let weaviate_status = if info.weaviate_running {
            style("● Weaviate running").green()
        } else {
            style("○ Weaviate not running").red()
        };
        println!("  {}", weaviate_status);
        println!();

        Ok(())
    }

    async fn test(&self, mcp: &McpManager) -> Result<()> {
        // Ensure binary is built
        mcp.ensure_binary()?;

        // Ensure Weaviate is running
        if !mcp.is_weaviate_running() {
            println!("{} Starting Weaviate first...", style("→").cyan());
            mcp.start_weaviate()?;
        }

        let mcp_binary = mcp.mcp_binary()?;
        let weaviate_url = mcp.weaviate_url();

        println!();
        println!(
            "{} Testing MCP server with Weaviate at {}...",
            style("→").cyan().bold(),
            weaviate_url
        );
        println!();

        // Send initialize request
        let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;

        let output = std::process::Command::new(&mcp_binary)
            .args(&["--weaviate-url", &weaviate_url])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(ref mut stdin) = child.stdin {
                    stdin.write_all(init_request.as_bytes())?;
                }
                child.wait_with_output()
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(first_line) = stdout.lines().next() {
            // Pretty print the JSON response
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(first_line) {
                println!("{}", serde_json::to_string_pretty(&json)?);
            } else {
                println!("{}", first_line);
            }
        }

        println!();
        println!("{} MCP server responds correctly!", style("✓").green().bold());

        Ok(())
    }
}

// Re-export exec for the run command
mod exec {
    use std::ffi::OsStr;
    
    pub struct Command {
        program: std::path::PathBuf,
        args: Vec<std::ffi::OsString>,
    }

    impl Command {
        pub fn new(program: impl AsRef<std::path::Path>) -> Self {
            Self {
                program: program.as_ref().to_path_buf(),
                args: Vec::new(),
            }
        }

        pub fn args<I, S>(mut self, args: I) -> Self
        where
            I: IntoIterator<Item = S>,
            S: AsRef<OsStr>,
        {
            self.args.extend(args.into_iter().map(|s| s.as_ref().to_os_string()));
            self
        }

        #[cfg(unix)]
        pub fn exec(self) -> std::io::Error {
            use std::os::unix::process::CommandExt;
            std::process::Command::new(&self.program)
                .args(&self.args)
                .exec()
        }

        #[cfg(not(unix))]
        pub fn exec(self) -> std::io::Error {
            // On non-Unix, just run and exit
            match std::process::Command::new(&self.program)
                .args(&self.args)
                .status()
            {
                Ok(status) => {
                    std::process::exit(status.code().unwrap_or(1));
                }
                Err(e) => e,
            }
        }
    }
}
