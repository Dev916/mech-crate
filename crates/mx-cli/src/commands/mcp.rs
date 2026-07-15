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
    /// Show status (corpus backend + counts)
    Status,
    /// Alias for status
    Ps,
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
            McpSubcommand::Status | McpSubcommand::Ps => self.status(&mcp).await,
            McpSubcommand::Config => self.config(&mcp).await,
            McpSubcommand::Run => self.run_server(&mcp).await,
            McpSubcommand::Info => self.info(&mcp).await,
            McpSubcommand::Test => self.test(&mcp).await,
        }
    }

    async fn build(&self, mcp: &McpManager) -> Result<()> {
        println!("{} Building MCP server...", style("→").cyan().bold());

        mcp.build()?;

        println!(
            "{} MCP server built successfully!",
            style("✓").green().bold()
        );
        println!();

        if let Ok(bin) = mcp.mcp_binary() {
            println!("  Binaries:");
            println!("    {}", bin.display());
        }

        Ok(())
    }

    async fn status(&self, _mcp: &McpManager) -> Result<()> {
        let cfg = mx_lib::corpus::RagConfig::load();
        match mx_lib::corpus::CorpusStore::connect(&cfg).await {
            Ok(store) => {
                let st = store.status().await?;
                println!("{}", style("Techniques Corpus").bold());
                println!(
                    "  {} Backend: {}",
                    style("•").dim(),
                    st["backend"].as_str().unwrap_or("?")
                );
                println!(
                    "  {} Docs: {} / Chunks: {}",
                    style("•").dim(),
                    st["docs"],
                    st["chunks"]
                );
                println!(
                    "  {} Model: {}",
                    style("•").dim(),
                    st["embedding_model"].as_str().unwrap_or("?")
                );
            }
            Err(e) => {
                println!("{} Corpus offline: {}", style("✗").red().bold(), e);
                println!(
                    "  Configure ~/.mech-crate/config/rag.toml or start local pgvector (see mx rag status)."
                );
            }
        }
        Ok(())
    }

    async fn config(&self, mcp: &McpManager) -> Result<()> {
        // Ensure binary is built
        if mcp.needs_build() {
            println!(
                "{} MCP binary not built. Run 'mx mcp build' first.",
                style("!").yellow()
            );
            return Ok(());
        }

        let config_json = mcp.generate_config()?;

        println!();
        println!("{}", style("MCP Client Configuration").bold());
        println!();
        println!("Add this to your MCP client configuration:");
        println!();
        println!(
            "{}",
            style("Claude Desktop (~/.claude/claude_desktop_config.json):").cyan()
        );
        println!();
        println!("{}", config_json);
        println!();
        println!(
            "{}",
            style("Cursor IDE (mcp.json in workspace or ~/.cursor/mcp.json):").cyan()
        );
        println!();
        println!("{}", config_json);
        println!();

        let wrapper_path = mcp.state_dir().join("mx-mcp-wrapper.sh");
        println!(
            "{} Wrapper script: {}",
            style("ℹ").blue(),
            wrapper_path.display()
        );

        Ok(())
    }

    async fn run_server(&self, mcp: &McpManager) -> Result<()> {
        // Ensure binary is built
        mcp.ensure_binary()?;

        let mcp_binary = mcp.mcp_binary()?;

        println!("{} Starting MCP server...", style("→").cyan().bold());

        // Execute the MCP binary - this replaces the current process
        let err = exec::Command::new(&mcp_binary).exec();

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
            info.mcp_binary
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "not found".to_string())
        );
        println!(
            "  {} State Dir: {}",
            style("•").dim(),
            info.state_dir.display()
        );
        println!(
            "  {} Source Dir: {}",
            style("•").dim(),
            info.source_dir
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "not found".to_string())
        );
        println!();

        let binary_status = if info.binary_built {
            style("● MCP binary built").green()
        } else {
            style("○ MCP binary not built (run: mx mcp build)").yellow()
        };
        println!("  {}", binary_status);
        println!();

        Ok(())
    }

    async fn test(&self, mcp: &McpManager) -> Result<()> {
        // Ensure binary is built
        mcp.ensure_binary()?;

        let mcp_binary = mcp.mcp_binary()?;

        println!();
        println!("{} Testing MCP server...", style("→").cyan().bold());
        println!();

        // Send initialize request
        let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;

        let output = std::process::Command::new(&mcp_binary)
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
        println!(
            "{} MCP server responds correctly!",
            style("✓").green().bold()
        );

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

        #[allow(dead_code)]
        pub fn args<I, S>(mut self, args: I) -> Self
        where
            I: IntoIterator<Item = S>,
            S: AsRef<OsStr>,
        {
            self.args
                .extend(args.into_iter().map(|s| s.as_ref().to_os_string()));
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
