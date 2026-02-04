//! `mx doctor` command - Check project health

use anyhow::Result;
use clap::Args;
use console::style;

use mx_lib::{home_dir, templates_dir, is_initialized};
use mx_lib::docker::Docker;
use mx_lib::project::ProjectDetector;

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
            if initialized { "yes" } else { "no (run 'mx init')" }
        );

        if let Ok(home) = home_dir() {
            println!("  {} Home: {}", style("•").dim(), home.display());
        }

        if let Ok(templates) = templates_dir() {
            println!("  {} Templates: {}", style("•").dim(), templates.display());
        }

        // Check version
        let version_file = home_dir().map(|h| h.join("version")).ok();
        if let Some(vf) = version_file {
            if vf.exists() {
                if let Ok(version) = std::fs::read_to_string(&vf) {
                    println!("  {} Version: {}", style("•").dim(), version.trim());
                }
            }
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
            println!("{}", Docker::version().unwrap_or_else(|_| "installed".into()));
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
                    ("docker/compose/", project_root.join("docker/compose").is_dir()),
                    ("docker/.config/", project_root.join("docker/.config").is_dir()),
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
                println!(
                    "{} Not in a MechCrate project",
                    style("!").yellow()
                );
                println!("  Create one with: mx new <project-name>");
            }
        }

        Ok(())
    }
}
