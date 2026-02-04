//! `mx init` command - Initialize MechCrate installation

use std::path::Path;

use anyhow::Result;
use clap::Args;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use walkdir::WalkDir;

use mx_lib::{home_dir, source_templates_dir, is_initialized};

/// Initialize MechCrate installation
#[derive(Args, Debug)]
pub struct InitCommand {
    /// Force re-initialization (overwrite existing)
    #[arg(short, long)]
    force: bool,

    /// Update templates only (keep config)
    #[arg(short, long)]
    update: bool,
}

impl InitCommand {
    pub async fn run(&self) -> Result<()> {
        let home = home_dir()?;

        // Check if already initialized
        if is_initialized() && !self.force && !self.update {
            println!(
                "{} MechCrate already initialized at: {}",
                style("✓").green(),
                home.display()
            );
            println!();
            println!("  Use {} to update templates", style("mx init --update").cyan());
            println!("  Use {} to force re-initialize", style("mx init --force").cyan());
            return Ok(());
        }

        // Find source templates
        let source = match source_templates_dir() {
            Ok(s) => s,
            Err(_) => {
                anyhow::bail!(
                    "Cannot find source templates. Make sure you're running mx from a valid installation."
                );
            }
        };

        println!(
            "{} Initializing MechCrate at: {}",
            style("→").cyan().bold(),
            style(home.display()).green()
        );
        println!("  Source: {}", source.display());
        println!();

        // Create directory structure
        let dirs = [
            "",           // root
            "config",
            "config/infra",
            "config/unyform",
            "recipes",    // cached recipes from Unyform
            "router",     // Traefik router installation
            "mcp",        // MCP server state
        ];

        for dir in dirs {
            let path = home.join(dir);
            if !path.exists() {
                std::fs::create_dir_all(&path)?;
                println!("  {} Created: {}", style("•").dim(), dir.is_empty().then(|| ".mech-crate/").unwrap_or(dir));
            }
        }

        // Copy templates
        let templates_dest = home.join("templates");
        
        if templates_dest.exists() {
            if self.force {
                println!("  {} Removing old templates...", style("→").cyan());
                std::fs::remove_dir_all(&templates_dest)?;
            } else if self.update {
                println!("  {} Updating templates...", style("→").cyan());
                std::fs::remove_dir_all(&templates_dest)?;
            }
        }

        // Count files for progress
        let file_count = WalkDir::new(&source)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .count();

        let pb = ProgressBar::new(file_count as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  {spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("█▓░"),
        );
        pb.set_message("Copying templates...");

        // Copy templates recursively
        self.copy_dir(&source, &templates_dest, &pb)?;

        pb.finish_with_message("Done!");

        // Create version file
        let version_file = home.join("version");
        std::fs::write(&version_file, env!("CARGO_PKG_VERSION"))?;

        println!();
        println!(
            "{} MechCrate initialized successfully!",
            style("✓").green().bold()
        );
        println!();
        println!("You can now use mx commands from anywhere:");
        println!("  {} - Create a new project", style("mx new <project-name>").cyan());
        println!("  {} - List available recipes", style("mx recipes list").cyan());
        println!("  {} - Check system health", style("mx doctor").cyan());

        Ok(())
    }

    fn copy_dir(&self, from: &Path, to: &Path, pb: &ProgressBar) -> Result<()> {
        for entry in WalkDir::new(from) {
            let entry = entry?;
            let relative = entry.path().strip_prefix(from)?;
            let dest = to.join(relative);

            if entry.file_type().is_dir() {
                std::fs::create_dir_all(&dest)?;
            } else if entry.file_type().is_file() {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(entry.path(), &dest)?;

                // Preserve executable permission
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let source_meta = std::fs::metadata(entry.path())?;
                    let mode = source_meta.permissions().mode();
                    if mode & 0o111 != 0 {
                        std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(mode))?;
                    }
                }

                pb.inc(1);
            }
        }

        Ok(())
    }
}
