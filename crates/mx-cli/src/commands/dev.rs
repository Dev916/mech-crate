//! `mx dev/up/down/logs/restart/sh/ps` commands - Development workflow proxied to Make

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use clap::Args;
use console::style;

use mx_lib::{ensure_path, project::ProjectDetector};

/// Development commands (proxied to Make)
#[derive(Args, Debug)]
pub struct DevCommand {
    /// Service name (optional)
    service: Option<String>,

    /// Follow logs (for logs command)
    #[arg(short, long)]
    follow: bool,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
}

impl DevCommand {
    pub async fn run_dev(&self) -> Result<()> {
        self.run_make_target("dev").await
    }

    pub async fn run_up(&self) -> Result<()> {
        self.run_make_target("up").await
    }

    pub async fn run_down(&self) -> Result<()> {
        self.run_make_target("down").await
    }

    pub async fn run_logs(&self) -> Result<()> {
        self.run_make_target("logs").await
    }

    pub async fn run_restart(&self) -> Result<()> {
        self.run_make_target("restart").await
    }

    pub async fn run_sh(&self) -> Result<()> {
        self.run_make_target("sh").await
    }

    pub async fn run_ps(&self) -> Result<()> {
        self.run_make_target("ps").await
    }

    async fn run_make_target(&self, target: &str) -> Result<()> {
        let mut args = Vec::new();

        // Add service if specified
        if let Some(ref service) = self.service {
            args.push(format!("s={}", service));
        }

        // Add follow flag for logs
        if self.follow && target == "logs" {
            args.push("f=1".to_string());
        }

        run_make_in_project(&find_project_root()?, target, &args, self.verbose)
    }
}

/// The MechCrate project root above the current directory, or an error that
/// says what a project needs.
pub(crate) fn find_project_root() -> Result<PathBuf> {
    ProjectDetector::new()
        .find_root_from_cwd()
        .with_context(|| {
            format!(
                "Not in a MechCrate project.\n\n\
            A project needs at minimum:\n  \
            - Makefile\n  \
            - docker/ directory\n\n\
            Current directory: {}",
                std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "unknown".to_string())
            )
        })
}

/// Run `make <target> <args...>` interactively in `project_root`, exiting with
/// make's own status code when it fails. Shared by every make-proxy command.
pub(crate) fn run_make_in_project(
    project_root: &Path,
    target: &str,
    args: &[String],
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!(
            "{} Project root: {}",
            style("→").cyan(),
            project_root.display()
        );
    }

    // Build the make command
    let mut cmd = Command::new("make");

    // Set working directory to project root
    cmd.current_dir(project_root);

    // Ensure PATH includes common binary locations (docker, etc.)
    cmd.env("PATH", ensure_path());

    // Pass through stdin/stdout/stderr for interactive commands
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    cmd.arg(target);
    cmd.args(args);

    if verbose {
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy()).collect();
        println!("{} Running: make {}", style("→").cyan(), args.join(" "));
        println!("{} PATH includes: /usr/local/bin", style("→").cyan());
    }

    // Run interactively
    let status = cmd.status().with_context(|| {
        format!(
            "Failed to run 'make {}' in {}",
            target,
            project_root.display()
        )
    })?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
