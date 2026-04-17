//! `mx build` command - Build Docker images

use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use clap::Args;
use console::style;

use mx_lib::{ensure_path, project::ProjectDetector};

/// Build Docker images
#[derive(Args, Debug)]
pub struct BuildCommand {
    /// Service to build
    service: String,

    /// Build production image
    #[arg(long)]
    prod: bool,

    /// Build development image
    #[arg(long)]
    dev: bool,

    /// Image tag
    #[arg(short, long)]
    tag: Option<String>,

    /// Push image after build
    #[arg(long)]
    push: bool,

    /// Don't use cache
    #[arg(long)]
    no_cache: bool,

    /// Target platform
    #[arg(long)]
    platform: Option<String>,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
}

impl BuildCommand {
    pub async fn run(&self) -> Result<()> {
        let detector = ProjectDetector::new();
        
        let project_root = detector.find_root_from_cwd().with_context(|| {
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
        })?;

        let mode = if self.prod { "prod" } else { "dev" };

        if self.verbose {
            println!(
                "{} Project root: {}",
                style("→").cyan(),
                project_root.display()
            );
        }

        println!(
            "{} Building {} ({})",
            style("→").cyan().bold(),
            style(&self.service).green(),
            mode
        );

        let mut cmd = Command::new("make");
        
        // Set working directory to project root
        cmd.current_dir(&project_root);
        
        // Ensure PATH includes common binary locations (docker, etc.)
        cmd.env("PATH", ensure_path());
        
        // Pass through stdin/stdout/stderr
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
        
        cmd.arg("_build");
        cmd.arg(format!("service={}", self.service));
        cmd.arg(format!("tag={}", self.tag.as_deref().unwrap_or("latest")));
        cmd.arg(format!("mode={}", mode));
        cmd.arg(format!("push={}", if self.push { "1" } else { "0" }));
        cmd.arg(format!("nocache={}", if self.no_cache { "1" } else { "0" }));

        if let Some(ref platform) = self.platform {
            cmd.arg(format!("platform={}", platform));
        }

        if self.verbose {
            let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy()).collect();
            println!(
                "{} Running: make {}",
                style("→").cyan(),
                args.join(" ")
            );
        }

        let status = cmd
            .status()
            .with_context(|| format!("Failed to run 'make _build' in {}", project_root.display()))?;

        if !status.success() {
            anyhow::bail!("Build failed");
        }

        println!(
            "{} Build complete: {}",
            style("✓").green().bold(),
            style(&self.service).green()
        );

        Ok(())
    }
}
