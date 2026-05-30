//! MechCrate CLI
//!
//! A CLI for project scaffolding, service management, and infrastructure automation.

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::style;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod commands;

use commands::{
    add::AddCommand, build::BuildCommand, cc_plugin::CcPluginCommand, dev::DevCommand,
    docs::DocsCommand, doctor::DoctorCommand, infra::InfraCommand, init::InitCommand,
    mcp::McpCommand, new::NewCommand, recipes::RecipesCommand, router::RouterCommand,
    self_update::SelfUpdateCommand, unyform::UnyformCommand,
};

/// MechCrate CLI - Project scaffolding and infrastructure automation
#[derive(Parser)]
#[command(name = "mx")]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize MechCrate (install templates to ~/.mech-crate)
    Init(InitCommand),

    /// Create a new MechCrate project
    New(NewCommand),

    /// Add a service to the project
    Add(AddCommand),

    /// Manage recipes
    Recipes(RecipesCommand),

    /// Start development environment
    Dev(DevCommand),

    /// Start services (production mode)
    Up(DevCommand),

    /// Stop services
    Down(DevCommand),

    /// View service logs
    Logs(DevCommand),

    /// Restart a service
    Restart(DevCommand),

    /// Open shell in a service container
    Sh(DevCommand),

    /// List running services
    Ps(DevCommand),

    /// Build service images
    Build(BuildCommand),

    /// Compile Markdown documents to PDF/HTML
    Docs(DocsCommand),

    /// Manage global Traefik router
    Router(RouterCommand),

    /// Manage infrastructure providers
    Infra(InfraCommand),

    /// MCP server management
    Mcp(McpCommand),

    /// Check project health
    Doctor(DoctorCommand),

    /// Unyform integration
    Unyform(UnyformCommand),

    /// Manage the Unyform Claude Code plugin (install / uninstall hooks)
    #[command(name = "cc-plugin")]
    CcPlugin(CcPluginCommand),

    /// Login to Unyform
    Login(commands::unyform::LoginCommand),

    /// Logout from Unyform
    Logout(commands::unyform::LogoutCommand),

    /// Show current Unyform user
    Whoami(commands::unyform::WhoamiCommand),

    /// Upgrade project scaffolding
    Upgrade(commands::upgrade::UpgradeCommand),

    /// Update the mx CLI itself
    #[command(name = "self-update")]
    SelfUpdate(SelfUpdateCommand),
}

fn setup_logging(verbose: bool) {
    let filter = if verbose {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"))
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"))
    };

    tracing_subscriber::registry()
        .with(fmt::layer().without_time().with_target(false))
        .with(filter)
        .init();
}

fn print_error(msg: &str) {
    eprintln!("{} {}", style("error:").red().bold(), msg);
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    setup_logging(cli.verbose);

    let result = match cli.command {
        Commands::Init(cmd) => cmd.run().await,
        Commands::New(cmd) => cmd.run().await,
        Commands::Add(cmd) => cmd.run().await,
        Commands::Recipes(cmd) => cmd.run().await,
        Commands::Dev(cmd) => cmd.run_dev().await,
        Commands::Up(cmd) => cmd.run_up().await,
        Commands::Down(cmd) => cmd.run_down().await,
        Commands::Logs(cmd) => cmd.run_logs().await,
        Commands::Restart(cmd) => cmd.run_restart().await,
        Commands::Sh(cmd) => cmd.run_sh().await,
        Commands::Ps(cmd) => cmd.run_ps().await,
        Commands::Build(cmd) => cmd.run().await,
        Commands::Docs(cmd) => cmd.run().await,
        Commands::Router(cmd) => cmd.run().await,
        Commands::Infra(cmd) => cmd.run().await,
        Commands::Mcp(cmd) => cmd.run().await,
        Commands::Doctor(cmd) => cmd.run().await,
        Commands::Unyform(cmd) => cmd.run().await,
        Commands::CcPlugin(cmd) => cmd.run().await,
        Commands::Login(cmd) => cmd.run().await,
        Commands::Logout(cmd) => cmd.run().await,
        Commands::Whoami(cmd) => cmd.run().await,
        Commands::Upgrade(cmd) => cmd.run().await,
        Commands::SelfUpdate(cmd) => cmd.run().await,
    };

    if let Err(e) = result {
        print_error(&e.to_string());
        std::process::exit(1);
    }

    Ok(())
}
