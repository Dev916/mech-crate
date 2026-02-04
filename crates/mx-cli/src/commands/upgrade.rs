//! `mx upgrade` command - Update project scaffolding

use anyhow::Result;
use clap::Args;
use console::style;
use dialoguer::{Confirm, Select};

use mx_lib::project::ProjectDetector;
use mx_lib::upgrade::{ProjectUpgrader, UpgradeAction};

/// Upgrade project scaffolding
#[derive(Args, Debug)]
pub struct UpgradeCommand {
    /// Show diff for each changed file
    #[arg(short, long)]
    diff: bool,

    /// Auto-accept all updates (non-interactive)
    #[arg(short, long)]
    yes: bool,

    /// Show what would be done without making changes
    #[arg(short = 'n', long)]
    dry_run: bool,
}

impl UpgradeCommand {
    pub async fn run(&self) -> Result<()> {
        // Check if MechCrate is initialized
        if !mx_lib::is_initialized() {
            anyhow::bail!(
                "MechCrate not initialized. Run 'mx init' first to install templates."
            );
        }

        // Find project root
        let detector = ProjectDetector::new();
        let project_root = detector.find_root_from_cwd()?;

        println!();
        println!(
            "{}{}{}",
            style("  ").on_cyan(),
            style(" 🦝 MechCrate Project Upgrade ").bold().on_cyan(),
            style("  ").on_cyan()
        );
        println!();
        println!(
            "{} Upgrading project at: {}",
            style("→").cyan().bold(),
            project_root.display()
        );
        println!();

        let upgrader = ProjectUpgrader::new(&project_root)?;

        // Ensure required directories exist
        println!("{}", style("Checking directories...").dim());
        for dir in upgrader.required_directories() {
            let path = project_root.join(dir);
            if !path.exists() {
                if self.dry_run {
                    println!(
                        "  {} Would create: {}",
                        style("[DRY RUN]").blue(),
                        dir
                    );
                } else {
                    std::fs::create_dir_all(&path)?;
                    println!("  {} Created: {}", style("✓").green(), dir);
                }
            }
        }

        // Discover upgrades
        println!();
        println!("{}", style("Scanning templates...").dim());

        let entries = upgrader.discover_upgrades()?;

        let mut added_count = 0;
        let mut updated_count = 0;
        let mut skipped_count = 0;
        let mut pending_updates = Vec::new();

        // Process entries
        for entry in &entries {
            let rel_path = entry.project_path.strip_prefix(&project_root).unwrap_or(&entry.project_path);

            match &entry.action {
                UpgradeAction::Add => {
                    if self.dry_run {
                        println!(
                            "  {} Would add: {}",
                            style("[DRY RUN]").blue(),
                            rel_path.display()
                        );
                    } else {
                        upgrader.copy_file(&entry.template_path, &entry.project_path)?;
                        println!("  {} Added: {}", style("✓").green(), rel_path.display());
                    }
                    added_count += 1;
                }
                UpgradeAction::Update => {
                    pending_updates.push(entry.clone());
                }
                UpgradeAction::Current | UpgradeAction::Skip => {
                    // Already current or skipped
                }
            }
        }

        // Handle pending updates
        if !pending_updates.is_empty() {
            println!();
            println!(
                "{}",
                style("┌────────────────────────────────────────────────────────────┐").cyan()
            );
            println!(
                "{}  {} Tooling Updates Available                              {}",
                style("│").cyan(),
                style("📝").bold(),
                style("│").cyan()
            );
            println!(
                "{}",
                style("└────────────────────────────────────────────────────────────┘").cyan()
            );
            println!();
            println!("The following tooling files have updates available:");
            println!();

            for entry in &pending_updates {
                let rel_path = entry.project_path.strip_prefix(&project_root).unwrap_or(&entry.project_path);
                println!("    • {}", rel_path.display());
            }

            println!();

            if self.dry_run {
                println!(
                    "  {} Would prompt to update {} file(s)",
                    style("[DRY RUN]").blue(),
                    pending_updates.len()
                );
            } else {
                let mut auto_yes = self.yes;

                for entry in &pending_updates {
                    let rel_path = entry
                        .project_path
                        .strip_prefix(&project_root)
                        .unwrap_or(&entry.project_path);

                    println!();
                    println!("{}", style("━".repeat(60)).yellow());
                    println!("{}", style(rel_path.display()).bold());
                    println!("{}", style("━".repeat(60)).yellow());

                    if self.diff {
                        if let Ok(diff) = upgrader.file_diff(&entry.project_path, &entry.template_path) {
                            println!();
                            for line in diff.lines().take(50) {
                                if line.starts_with('+') {
                                    println!("{}", style(line).green());
                                } else if line.starts_with('-') {
                                    println!("{}", style(line).red());
                                } else {
                                    println!("{}", style(line).dim());
                                }
                            }
                            println!();
                        }
                    }

                    let update_file = if auto_yes {
                        true
                    } else {
                        println!();
                        println!("Options:");
                        println!("    [y] Yes, update this file");
                        println!("    [n] No, skip this file");
                        println!("    [d] Show diff first");
                        println!("    [a] Accept all remaining updates");
                        println!("    [q] Quit (skip all remaining)");
                        println!();

                        let options = vec!["Yes", "No", "Show diff", "Accept all", "Quit"];
                        let selection = Select::new()
                            .with_prompt(format!("Update {}?", rel_path.display()))
                            .items(&options)
                            .default(0)
                            .interact()?;

                        match selection {
                            0 => true,  // Yes
                            1 => false, // No
                            2 => {
                                // Show diff and ask again
                                if let Ok(diff) = upgrader.file_diff(&entry.project_path, &entry.template_path) {
                                    println!();
                                    for line in diff.lines() {
                                        if line.starts_with('+') {
                                            println!("{}", style(line).green());
                                        } else if line.starts_with('-') {
                                            println!("{}", style(line).red());
                                        } else {
                                            println!("{}", style(line).dim());
                                        }
                                    }
                                    println!();
                                }
                                Confirm::new()
                                    .with_prompt("Update this file?")
                                    .default(true)
                                    .interact()?
                            }
                            3 => {
                                // Accept all
                                auto_yes = true;
                                true
                            }
                            4 => {
                                // Quit
                                println!("{} Skipping remaining updates.", style("ℹ").blue());
                                break;
                            }
                            _ => false,
                        }
                    };

                    if update_file {
                        // Backup original file
                        let backup_path = entry.project_path.with_extension(
                            format!(
                                "{}.bak",
                                entry
                                    .project_path
                                    .extension()
                                    .and_then(|e| e.to_str())
                                    .unwrap_or("")
                            ),
                        );
                        std::fs::copy(&entry.project_path, &backup_path)?;

                        upgrader.copy_file(&entry.template_path, &entry.project_path)?;

                        println!(
                            "  {} Updated: {} (backup: {})",
                            style("✓").green(),
                            rel_path.display(),
                            backup_path.file_name().unwrap().to_str().unwrap()
                        );
                        updated_count += 1;
                    } else {
                        println!("  {} Skipped: {}", style("○").yellow(), rel_path.display());
                        skipped_count += 1;
                    }
                }
            }
        }

        // Summary
        println!();
        println!(
            "{}",
            style("┌────────────────────────────────────────────────────────────┐").cyan()
        );
        println!(
            "{}  {} Upgrade Summary                                        {}",
            style("│").cyan(),
            style("📊").bold(),
            style("│").cyan()
        );
        println!(
            "{}",
            style("└────────────────────────────────────────────────────────────┘").cyan()
        );
        println!();

        if self.dry_run {
            println!("  {} No changes were made", style("[DRY RUN]").blue());
            println!();
        }

        if added_count > 0 {
            println!("  {} Added:   {} file(s)", style("✓").green(), added_count);
        }
        if updated_count > 0 {
            println!("  {} Updated: {} file(s)", style("✓").green(), updated_count);
        }
        if skipped_count > 0 {
            println!("  {} Skipped: {} file(s)", style("○").yellow(), skipped_count);
        }
        if added_count == 0 && updated_count == 0 && skipped_count == 0 && pending_updates.is_empty() {
            println!("  {} Project is up to date!", style("✓").green());
        }

        println!();
        println!("{} Your tooling is fresh!", style("🦝 Crate Raccoon says:").cyan());
        println!();

        Ok(())
    }
}
