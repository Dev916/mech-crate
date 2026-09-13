//! MechCrate CLI
//!
//! A CLI for project scaffolding, service management, and infrastructure automation.

use std::io::IsTerminal;
use std::path::Path;
use std::process::Stdio;

use anyhow::Result;
use chrono::Utc;
use clap::{Parser, Subcommand};
use console::style;
use mx_lib::selfupdate::notify::{self, Action};
use mx_lib::selfupdate::{InstallKind, Version};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod commands;

use commands::{
    add::AddCommand, build::BuildCommand, cc_plugin::CcPluginCommand, dev::DevCommand,
    docs::DocsCommand, doctor::DoctorCommand, infra::InfraCommand, init::InitCommand,
    mcp::McpCommand, new::NewCommand, rag::RagCommand, recipes::RecipesCommand,
    router::RouterCommand, self_update::SelfUpdateCommand, unyform::UnyformCommand,
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

    /// Techniques corpus (RAG) management
    Rag(RagCommand),

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

/// Hidden test seam: overrides the "is stderr a terminal?" answer, so the
/// notifier's TTY branch is reachable under a test harness's pipes.
const TTY_OVERRIDE_ENV: &str = "MX_UPDATE_CHECK_TTY";

/// Every subcommand gets the passive update check except two: `self-update`
/// (it *is* the update path, and `--refresh-cache` is the notifier's own
/// background half) and `mcp` (its server speaks JSON-RPC over stdio, and
/// even a stderr line has no business in that stream).
fn runs_update_check(command: &Commands) -> bool {
    !matches!(command, Commands::SelfUpdate(_) | Commands::Mcp(_))
}

/// The passive update notifier (spec §3.6).
///
/// Runs after the user's command, costs one small file read in the steady
/// state, never touches the network, never waits on anything, and swallows
/// every error: the command has already run and nothing here may change its
/// outcome or its exit code.
fn run_update_check() {
    // Cheap, allocation-free opt-outs first — these cost no IO at all.
    if !stderr_is_tty() || env_set(notify::DISABLE_ENV) || env_set("CI") {
        return;
    }
    let Ok(home) = mx_lib::home_dir() else {
        return;
    };
    let current = mx_lib::selfupdate::current();
    // `config/update.toml` is consulted only once something would actually
    // happen, which keeps the common path to a single file read. Because the
    // config can only ever force `Silent`, deciding first and filtering after
    // is the same answer.
    let ctx = notify::Context {
        now: Utc::now(),
        current: &current,
        stderr_is_tty: true,
        disabled_by_env: false,
        disabled_by_ci: false,
        disabled_by_config: false,
    };
    let cache = notify::read_cache(&home);
    let action = notify::decide(cache.as_ref(), &ctx);
    if matches!(action, Action::Silent) || notify::config_disables(&home) {
        return;
    }

    // Hint first, spawn second: the refresh carries the recorded hint
    // forward, so writing it before the child starts avoids a lost update.
    if let Action::Hint(latest) | Action::SpawnAndHint(latest) = &action {
        hint(&home, cache, latest, &current, ctx.now);
    }
    if matches!(action, Action::Spawn | Action::SpawnAndHint(_)) {
        spawn_refresh();
    }
}

/// True when the variable is set to anything non-empty.
fn env_set(key: &str) -> bool {
    std::env::var_os(key).is_some_and(|v| !v.is_empty())
}

/// Whether stderr is a terminal, honouring [`TTY_OVERRIDE_ENV`].
fn stderr_is_tty() -> bool {
    match std::env::var(TTY_OVERRIDE_ENV) {
        Ok(v) if !v.is_empty() => v != "0",
        _ => std::io::stderr().is_terminal(),
    }
}

/// Print the one-line hint and record it, so it is not repeated for a day.
fn hint(
    home: &Path,
    cache: Option<notify::Cache>,
    latest: &Version,
    current: &Version,
    now: chrono::DateTime<Utc>,
) {
    let homebrew = matches!(
        commands::self_update::detect_kind(home),
        InstallKind::Homebrew { .. }
    );
    eprintln!("{}", notify::hint_line(latest, current, homebrew));
    if let Some(mut cache) = cache {
        cache.hinted_at = Some(now);
        cache.hinted_version = Some(latest.clone());
        let _ = notify::write_cache(home, &cache);
    }
}

/// Launch `mx self-update --refresh-cache` detached and forget about it:
/// stdio on `/dev/null`, its own process group on unix so it outlives this
/// shell, and no wait.
fn spawn_refresh() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let mut cmd = std::process::Command::new(exe);
    cmd.args(["self-update", "--refresh-cache"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let _ = cmd.spawn();
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    setup_logging(cli.verbose);

    let notify_after = runs_update_check(&cli.command);

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
        Commands::Rag(cmd) => cmd.run().await,
        Commands::Doctor(cmd) => cmd.run().await,
        Commands::Unyform(cmd) => cmd.run().await,
        Commands::CcPlugin(cmd) => cmd.run().await,
        Commands::Login(cmd) => cmd.run().await,
        Commands::Logout(cmd) => cmd.run().await,
        Commands::Whoami(cmd) => cmd.run().await,
        Commands::Upgrade(cmd) => cmd.run().await,
        Commands::SelfUpdate(cmd) => cmd.run().await,
    };

    if notify_after {
        run_update_check();
    }

    if let Err(e) = result {
        print_error(&e.to_string());
        std::process::exit(1);
    }

    Ok(())
}
