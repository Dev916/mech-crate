//! `mx doctor` command - Check project health

use std::path::Path;

use anyhow::Result;
use clap::Args;
use console::style;

use mx_lib::docker::Docker;
use mx_lib::project::ProjectDetector;
use mx_lib::selfupdate::notify::read_cache;
use mx_lib::selfupdate::{self, InstallKind, Version, BREW_UPGRADE};
use mx_lib::{home_dir, is_initialized, templates_dir};

use crate::commands::self_update::detect_kind;

/// Check project health
#[derive(Args, Debug)]
pub struct DoctorCommand;

impl DoctorCommand {
    pub async fn run(&self) -> Result<()> {
        println!("{}", style("MechCrate Health Check").bold());
        println!("{}", style("─".repeat(40)).dim());
        println!();

        // Check MechCrate installation
        println!("{}", style("MechCrate Installation").bold());

        let initialized = is_initialized();
        let init_status = if initialized {
            style("✓").green()
        } else {
            style("✗").red()
        };
        println!(
            "  {} Initialized: {}",
            init_status,
            if initialized {
                "yes"
            } else {
                "no (run 'mx init')"
            }
        );

        if let Ok(home) = home_dir() {
            println!("  {} Home: {}", style("•").dim(), home.display());
        }

        if let Ok(templates) = templates_dir() {
            println!("  {} Templates: {}", style("•").dim(), templates.display());
        }

        // How this mx was installed, and whether it is current.
        if let Ok(home) = home_dir() {
            println!();
            println!("{}", style("Install").bold());
            print_install(&home);
        }

        println!();

        // Check global dependencies
        println!("{}", style("Global Dependencies").bold());

        // Docker
        let docker_ok = Docker::is_available();
        let docker_status = if docker_ok {
            style("✓").green()
        } else {
            style("✗").red()
        };
        print!("  {} Docker: ", docker_status);
        if docker_ok {
            println!(
                "{}",
                Docker::version().unwrap_or_else(|_| "installed".into())
            );
        } else {
            println!("{}", style("not found").red());
        }

        // Docker running
        if docker_ok {
            let running = Docker::is_running();
            let status = if running {
                style("✓").green()
            } else {
                style("✗").red()
            };
            println!(
                "  {} Docker daemon: {}",
                status,
                if running { "running" } else { "not running" }
            );
        }

        // Make
        let make_ok = which::which("make").is_ok();
        let make_status = if make_ok {
            style("✓").green()
        } else {
            style("✗").red()
        };
        println!(
            "  {} Make: {}",
            make_status,
            if make_ok { "installed" } else { "not found" }
        );

        // Check if in a project
        let detector = ProjectDetector::new();

        match detector.find_root_from_cwd() {
            Ok(project_root) => {
                println!();
                println!("{}", style("Project Structure").bold());

                let project = detector.analyze(&project_root)?;

                // Project root
                println!(
                    "  {} Project: {}",
                    style("✓").green(),
                    style(&project.name).green()
                );

                // Check required directories
                let dirs_to_check = [
                    ("docker/", project_root.join("docker").is_dir()),
                    (
                        "docker/compose/",
                        project_root.join("docker/compose").is_dir(),
                    ),
                    (
                        "docker/.config/",
                        project_root.join("docker/.config").is_dir(),
                    ),
                    ("make/", project_root.join("make").is_dir()),
                    ("scripts/", project_root.join("scripts").is_dir()),
                ];

                for (name, exists) in dirs_to_check {
                    let status = if exists {
                        style("✓").green()
                    } else {
                        style("✗").red()
                    };
                    println!("  {} {}", status, name);
                }

                // Services
                if !project.services.is_empty() {
                    println!();
                    println!("{}", style("Services").bold());
                    for service in &project.services {
                        println!("  {} {}", style("•").cyan(), service);
                    }
                }

                // Check for secrets file
                let secrets_file = project_root.join("docker/.config/.env.secrets");
                if !secrets_file.exists() {
                    println!();
                    println!(
                        "{} Missing: docker/.config/.env.secrets",
                        style("!").yellow()
                    );
                    println!("  Create with: touch docker/.config/.env.secrets");
                }
            }
            Err(_) => {
                println!();
                println!("{} Not in a MechCrate project", style("!").yellow());
                println!("  Create one with: mx new <project-name>");
            }
        }

        Ok(())
    }
}

/// Print the `Install` block: how the running `mx` was installed, which
/// version is running, which one `<home>/version` records, and what the
/// passive notifier last saw on the release channel.
///
/// Doctor is a report, not a network client: the latest version comes from
/// the update-check cache alone (spec §3.6), so this never blocks and never
/// leaves the machine.
fn print_install(home: &Path) {
    let kind = detect_kind(home);
    let running = selfupdate::current();

    println!(
        "  {} Kind:     {} ({})",
        style("•").dim(),
        kind.name(),
        install_detail(&kind)
    );
    println!("  {} Running:  {running}", style("•").dim());
    print_recorded(home, &running, &kind);
    print_latest(home, &running, &kind);
    print_shim_path_hint(&kind);
}

/// The one path that tells the user where this install actually lives.
fn install_detail(kind: &InstallKind) -> String {
    match kind {
        InstallKind::Release { home, version } => {
            format!("{}, mx-v{version}", home.join("releases").display())
        }
        InstallKind::Homebrew { cellar } => cellar.display().to_string(),
        InstallKind::Source { repo } => repo.display().to_string(),
        InstallKind::Bare { exe } => exe.display().to_string(),
    }
}

/// `<home>/version` — what the last install or update wrote there.
///
/// A value that disagrees with the running binary means the home directory
/// was left behind by an install that did not finish (templates and the MCP
/// wrapper are stale too), so it is a warning with the command that fixes it.
fn print_recorded(home: &Path, running: &Version, kind: &InstallKind) {
    let recorded = std::fs::read_to_string(home.join("version"))
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|v| !v.is_empty());
    match recorded {
        None => println!("  {} Recorded: (none) — run: mx init", style("•").dim()),
        Some(v) if v == running.to_string() => {
            println!("  {} Recorded: {v}", style("•").dim())
        }
        Some(v) => println!(
            "  {} Recorded: {v} (differs from running {running}) — run: {}",
            style("⚠").yellow(),
            resync_command(kind)
        ),
    }
}

/// What the notifier cache last learned about the release channel.
fn print_latest(home: &Path, running: &Version, kind: &InstallKind) {
    match read_cache(home).and_then(|cache| cache.latest) {
        None => println!("  {} Latest:   not checked yet", style("•").dim()),
        Some(latest) if selfupdate::is_newer(&latest, running) => println!(
            "  {} Latest:   {latest} — update available: {}",
            style("⚠").yellow(),
            update_command(kind)
        ),
        Some(latest) => println!("  {} Latest:   {latest} (up to date)", style("•").dim()),
    }
}

/// The command that brings `<home>` back in step with the running binary.
/// Release and bare installs own their layout; source and Homebrew installs
/// are updated elsewhere and only need the home directory refreshed.
fn resync_command(kind: &InstallKind) -> &'static str {
    match kind {
        InstallKind::Release { .. } | InstallKind::Bare { .. } => "mx self-update",
        InstallKind::Source { .. } | InstallKind::Homebrew { .. } => "mx init --update",
    }
}

/// The command that installs a newer release for this install kind.
fn update_command(kind: &InstallKind) -> &'static str {
    match kind {
        InstallKind::Homebrew { .. } => BREW_UPGRADE,
        _ => "mx self-update",
    }
}

/// Release installs are reached through the `~/.local/bin` shims; if that
/// directory is not on PATH, `mx` keeps resolving to whatever else is
/// installed and an update appears to do nothing.
fn print_shim_path_hint(kind: &InstallKind) {
    if !matches!(kind, InstallKind::Release { .. }) {
        return;
    }
    let Some(bin) = dirs::home_dir().map(|h| h.join(".local").join("bin")) else {
        return;
    };
    let path = std::env::var("PATH").unwrap_or_default();
    if path
        .split(':')
        .filter(|entry| !entry.is_empty())
        .any(|entry| Path::new(entry) == bin)
    {
        return;
    }
    println!(
        "  {} {} is not on PATH. Add to your shell profile:",
        style("⚠").yellow(),
        bin.display()
    );
    println!("    export PATH=\"$HOME/.local/bin:$PATH\"");
}
