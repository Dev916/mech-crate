//! `mx self-update` command - Rebuild and reinstall the mx CLI

use anyhow::Result;
use clap::Args;
use console::style;
use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Rebuild and reinstall the mx CLI from source
#[derive(Args, Debug)]
pub struct SelfUpdateCommand {
    /// Pull latest changes from git before building
    #[arg(long)]
    pull: bool,

    /// Skip interactive prompts
    #[arg(short, long)]
    yes: bool,

    /// Show what would be done without making changes
    #[arg(short = 'n', long)]
    dry_run: bool,
}

impl SelfUpdateCommand {
    pub async fn run(&self) -> Result<()> {
        println!();
        println!(
            "{}{}{}",
            style("  ").on_cyan(),
            style(" 🦝 MechCrate Self-Update ").bold().on_cyan(),
            style("  ").on_cyan()
        );
        println!();

        // Find the mech-crate source directory
        let source_dir = self.find_source_dir()?;
        let bin_dir = source_dir.join("bin");

        println!(
            "{} Source: {}",
            style("→").cyan().bold(),
            source_dir.display()
        );
        println!("{} Target: {}", style("→").cyan().bold(), bin_dir.display());

        // Get current version
        let current_version = env!("CARGO_PKG_VERSION");
        println!(
            "{} Current version: {}",
            style("→").cyan().bold(),
            current_version
        );
        println!();

        if self.dry_run {
            println!("{}", style("[DRY RUN] Would perform the following:").blue());
            let mut step = 1;
            if self.pull {
                println!("  {}. git pull --rebase in {}", step, source_dir.display());
                step += 1;
            }
            println!(
                "  {}. cargo build --release -p mx-cli -p mx-mcp-server",
                step
            );
            step += 1;
            println!("  {}. Copy binaries to {}", step, bin_dir.display());
            step += 1;
            println!(
                "  {}. Ensure /usr/local/bin symlinks point to {}",
                step,
                bin_dir.display()
            );
            println!();
            return Ok(());
        }

        // Confirm if not --yes
        if !self.yes {
            use dialoguer::Confirm;
            let proceed = Confirm::new()
                .with_prompt("Rebuild and reinstall mx?")
                .default(true)
                .interact()?;

            if !proceed {
                println!("{} Cancelled.", style("ℹ").blue());
                return Ok(());
            }
            println!();
        }

        // Git pull if requested
        if self.pull {
            println!("{} Pulling latest changes...", style("→").cyan().bold());
            let status = Command::new("git")
                .args(["pull", "--rebase"])
                .current_dir(&source_dir)
                .status()?;

            if !status.success() {
                anyhow::bail!("git pull failed");
            }
            println!("  {} Git pull complete", style("✓").green());
            println!();
        }

        // Build release
        println!("{} Building release binaries...", style("→").cyan().bold());
        let status = Command::new("cargo")
            .args(["build", "--release", "-p", "mx-cli", "-p", "mx-mcp-server"])
            .current_dir(&source_dir)
            .status()?;

        if !status.success() {
            anyhow::bail!("cargo build failed");
        }
        println!("  {} Build complete", style("✓").green());
        println!();

        // Copy binaries to bin/
        println!("{} Installing binaries...", style("→").cyan().bold());
        let release_dir = source_dir.join("target/release");

        std::fs::create_dir_all(&bin_dir)?;

        let binaries = [("mx", true), ("mx-mcp", false), ("mx-ingest", false)];

        for (name, required) in binaries {
            let src = release_dir.join(name);

            if !src.exists() {
                if required {
                    anyhow::bail!("Binary not found: {}", src.display());
                }
                continue;
            }

            let dst = bin_dir.join(name);
            std::fs::copy(&src, &dst)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&dst)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&dst, perms)?;
            }

            println!("  {} {} -> bin/{}", style("✓").green(), name, name);
        }

        println!();

        // Ensure /usr/local/bin symlinks
        println!(
            "{} Checking /usr/local/bin symlinks...",
            style("→").cyan().bold()
        );

        let system_bin = PathBuf::from("/usr/local/bin");
        let mut needs_sudo = false;

        for (name, _) in binaries {
            let bin_path = bin_dir.join(name);
            if !bin_path.exists() {
                continue;
            }

            let system_path = system_bin.join(name);

            // Check if it's already a correct symlink
            if system_path.is_symlink() {
                if let Ok(target) = std::fs::read_link(&system_path) {
                    if target == bin_path {
                        println!("  {} {} (symlink ok)", style("✓").green(), name);
                        continue;
                    }
                }
            }

            // Need to create/update symlink
            needs_sudo = true;
            println!("  {} {} needs symlink update", style("→").yellow(), name);
        }

        if needs_sudo {
            println!();
            println!(
                "  {} Creating symlinks (requires sudo)...",
                style("→").cyan().bold()
            );

            for (name, _) in binaries {
                let bin_path = bin_dir.join(name);
                if !bin_path.exists() {
                    continue;
                }

                let system_path = system_bin.join(name);

                // Check if already correct
                if system_path.is_symlink() {
                    if let Ok(target) = std::fs::read_link(&system_path) {
                        if target == bin_path {
                            continue;
                        }
                    }
                }

                // Remove existing and create symlink
                let _ = Command::new("sudo")
                    .args(["rm", "-f"])
                    .arg(&system_path)
                    .status();

                let status = Command::new("sudo")
                    .args(["ln", "-sf"])
                    .arg(&bin_path)
                    .arg(&system_path)
                    .status()?;

                if status.success() {
                    println!(
                        "  {} /usr/local/bin/{} -> bin/{}",
                        style("✓").green(),
                        name,
                        name
                    );
                } else {
                    println!(
                        "  {} Failed to symlink /usr/local/bin/{}",
                        style("✗").red(),
                        name
                    );
                }
            }
        }

        println!();

        // Run mx init to refresh templates
        println!("{} Refreshing templates...", style("→").cyan().bold());
        let status = Command::new(bin_dir.join("mx"))
            .args(["init", "--force"])
            .env("MECH_CRATE_ROOT", &source_dir)
            .status()?;

        if !status.success() {
            println!(
                "  {} Template refresh failed (non-fatal)",
                style("⚠").yellow()
            );
        }

        // Verify
        println!();
        println!("{} Verifying...", style("→").cyan().bold());
        let output = Command::new(bin_dir.join("mx"))
            .args(["--version"])
            .output()?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("  {} {}", style("✓").green(), version.trim());
        }

        println!();
        println!(
            "{}",
            style("┌────────────────────────────────────────────────────────────┐").green()
        );
        println!(
            "{}  {} Update complete!                                       {}",
            style("│").green(),
            style("✓").green().bold(),
            style("│").green()
        );
        println!(
            "{}",
            style("└────────────────────────────────────────────────────────────┘").green()
        );
        println!();

        Ok(())
    }

    /// Find the mech-crate source directory
    fn find_source_dir(&self) -> Result<PathBuf> {
        // 1. Check MECH_CRATE_ROOT env var
        if let Ok(root) = env::var("MECH_CRATE_ROOT") {
            let path = PathBuf::from(&root);
            if self.is_mech_crate_root(&path) {
                return Ok(path);
            }
        }

        // 2. Resolve from current exe (follows symlink)
        if let Ok(exe) = env::current_exe() {
            // current_exe resolves symlinks, so if /usr/local/bin/mx -> repo/bin/mx,
            // exe will be repo/bin/mx. Go up two levels to get repo root.
            if let Some(bin_dir) = exe.parent() {
                if let Some(repo_dir) = bin_dir.parent() {
                    if self.is_mech_crate_root(repo_dir) {
                        return Ok(repo_dir.to_path_buf());
                    }
                }
            }
        }

        // 3. Check ~/.mech-crate/source marker
        if let Some(home) = dirs::home_dir() {
            let marker = home.join(".mech-crate/source");
            if marker.exists() {
                if let Ok(path_str) = std::fs::read_to_string(&marker) {
                    let path = PathBuf::from(path_str.trim());
                    if self.is_mech_crate_root(&path) {
                        return Ok(path);
                    }
                }
            }
        }

        // 4. Try common locations
        if let Some(home) = dirs::home_dir() {
            let common = [
                "dev/dev916/mech-crate",
                "dev/mech-crate",
                "code/mech-crate",
                "projects/mech-crate",
            ];

            for rel in common {
                let path = home.join(rel);
                if self.is_mech_crate_root(&path) {
                    return Ok(path);
                }
            }
        }

        anyhow::bail!(
            "Could not find mech-crate source directory.\n\n\
            Set the MECH_CRATE_ROOT environment variable:\n\
            \n\
              export MECH_CRATE_ROOT=/path/to/mech-crate\n\
            \n\
            Or run from the mech-crate directory:\n\
            \n\
              cd /path/to/mech-crate && make upgrade"
        );
    }

    fn is_mech_crate_root(&self, path: &std::path::Path) -> bool {
        path.join("Cargo.toml").exists() && path.join("crates/mx-cli").exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Known-broken lane (bd:mech-crate-gjl) ────────────────────────────

    /// `mx init` records the source repo root through
    /// [`mx_lib::paths::save_source_root`], which writes
    /// `~/.mech-crate/config/source-root`. `find_source_dir`'s marker fallback
    /// reads `~/.mech-crate/source` instead, so it can never see what init
    /// recorded — the two ends of one seam disagree on the path.
    ///
    /// Asserts the FIXED behavior: the resolver finds the root that `mx init`
    /// recorded. Expected RED until bd:mech-crate-gjl lands.
    #[test]
    #[ignore = "bd:mech-crate-gjl self-update reads ~/.mech-crate/source, init writes config/source-root"]
    fn kb_self_update_finds_the_source_root_recorded_by_init() {
        let home = tempfile::tempdir().expect("setup: home tempdir");
        let repo = tempfile::tempdir().expect("setup: repo tempdir");
        std::fs::write(repo.path().join("Cargo.toml"), "[workspace]\n")
            .expect("setup: write fake Cargo.toml");
        std::fs::create_dir_all(repo.path().join("crates/mx-cli"))
            .expect("setup: create fake crates/mx-cli");

        // Isolate from the developer's real environment: no MECH_CRATE_ROOT
        // short-circuit, and a home that holds nothing but what init records.
        // nextest runs one process per test, so these mutations are contained.
        env::set_var("HOME", home.path());
        env::remove_var("MECH_CRATE_ROOT");

        mx_lib::paths::save_source_root(repo.path()).expect("setup: record source root");
        assert!(
            mx_lib::paths::recorded_source_root().is_some(),
            "setup: mx init's recording did not round-trip"
        );

        let cmd = SelfUpdateCommand {
            pull: false,
            yes: false,
            dry_run: true,
        };

        let found = cmd
            .find_source_dir()
            .expect("self-update must find the source root that mx init recorded");
        assert_eq!(
            found.canonicalize().unwrap(),
            repo.path().canonicalize().unwrap(),
            "self-update resolved a different source root than mx init recorded"
        );
    }
}
