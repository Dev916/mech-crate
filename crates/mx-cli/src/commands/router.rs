//! `mx router` command - Manage global Traefik router

use anyhow::Result;
use clap::{Args, Subcommand};
use console::style;

use mx_lib::config::MechCrateConfig;
use mx_lib::router::visibility::{declared_routers, diagnose, docker_provider_routers, RouterView};
use mx_lib::router::Router;

/// Manage global Traefik router
#[derive(Args, Debug)]
pub struct RouterCommand {
    #[command(subcommand)]
    command: RouterSubcommand,
}

#[derive(Subcommand, Debug)]
enum RouterSubcommand {
    /// Install the router
    Install,
    /// Start the router
    Up,
    /// Alias for up
    Start,
    /// Stop the router
    Down,
    /// Alias for down
    Stop,
    /// Restart the router
    Restart,
    /// Show router status
    Status,
    /// Alias for status
    Ps,
    /// View router logs
    Logs {
        /// Follow logs
        #[arg(short, long)]
        follow: bool,
    },
    /// Show detailed router info
    Inspect,
    /// Alias for inspect
    Info,
    /// Ensure network exists
    Network,
    /// Uninstall the router
    Uninstall,
    /// Alias for uninstall
    Remove,
}

impl RouterCommand {
    pub async fn run(&self) -> Result<()> {
        let config = MechCrateConfig::new()?;
        let router = Router::new(config);

        match &self.command {
            RouterSubcommand::Install => self.install(&router).await,
            RouterSubcommand::Up | RouterSubcommand::Start => self.start(&router).await,
            RouterSubcommand::Down | RouterSubcommand::Stop => self.stop(&router).await,
            RouterSubcommand::Restart => self.restart(&router).await,
            RouterSubcommand::Status | RouterSubcommand::Ps => self.status(&router).await,
            RouterSubcommand::Logs { follow } => self.logs(&router, *follow).await,
            RouterSubcommand::Inspect | RouterSubcommand::Info => self.inspect(&router).await,
            RouterSubcommand::Network => self.network(&router).await,
            RouterSubcommand::Uninstall | RouterSubcommand::Remove => self.uninstall(&router).await,
        }
    }

    async fn install(&self, router: &Router) -> Result<()> {
        if router.is_installed() {
            println!(
                "{} Router already installed at: {}",
                style("✓").green(),
                router.install_dir().display()
            );
            println!();
            println!("  Use {} to reinstall", style("--force").cyan());
            return Ok(());
        }

        // Check if MechCrate is initialized
        if !mx_lib::is_initialized() {
            anyhow::bail!("MechCrate not initialized. Run 'mx init' first to install templates.");
        }

        println!(
            "{} Installing router to: {}",
            style("→").cyan().bold(),
            router.install_dir().display()
        );

        router.install()?;

        println!(
            "{} Router installed successfully!",
            style("✓").green().bold()
        );
        println!();
        println!("  Start with: {}", style("mx router up").cyan());

        Ok(())
    }

    async fn start(&self, router: &Router) -> Result<()> {
        if !router.is_installed() {
            anyhow::bail!("Router not installed. Run 'mx router install' first.");
        }

        println!("{} Starting router...", style("→").cyan().bold());
        router.start()?;

        let status = router.status()?;
        if let Some(port) = status.dashboard_port {
            println!(
                "{} Router started. Dashboard: http://localhost:{}",
                style("✓").green().bold(),
                port
            );
        }

        Ok(())
    }

    async fn stop(&self, router: &Router) -> Result<()> {
        println!("{} Stopping router...", style("→").cyan().bold());
        router.stop()?;
        println!("{} Router stopped", style("✓").green().bold());
        Ok(())
    }

    async fn restart(&self, router: &Router) -> Result<()> {
        self.stop(router).await?;
        self.start(router).await
    }

    async fn status(&self, router: &Router) -> Result<()> {
        let status = router.status()?;

        println!("{}", style("Router Status").bold());
        println!("{}", style("─".repeat(40)).dim());

        let installed = if status.installed {
            style("installed").green()
        } else {
            style("not installed").red()
        };
        println!("  Installed: {}", installed);

        let running = if status.running {
            style("running").green()
        } else {
            style("stopped").red()
        };
        println!("  Status: {}", running);

        println!("  Network: {}", status.network);

        if let Some(port) = status.dashboard_port {
            println!("  Dashboard: http://localhost:{}", port);
        }

        println!("  Location: {}", status.install_dir.display());

        // "running" above is the router container on the active Docker context.
        // It does not say that container owns host :80. Compare what this
        // daemon's containers declare with what the Traefik answering on the
        // host has loaded (bd:mech-crate-86o).
        if status.running {
            if let Some(port) = status.dashboard_port {
                let view = diagnose(
                    &declared_router_names(router.network_name()),
                    loaded_router_names(port).await,
                );
                print_router_view(&view, port);
                if view.is_fault() {
                    anyhow::bail!(
                        "the router is running but is not routing this Docker daemon's containers"
                    );
                }
            }
        }

        Ok(())
    }

    async fn logs(&self, router: &Router, follow: bool) -> Result<()> {
        let output = router.logs(follow)?;
        print!("{}", String::from_utf8_lossy(&output.stdout));
        Ok(())
    }

    async fn inspect(&self, router: &Router) -> Result<()> {
        self.status(router).await
    }

    async fn network(&self, router: &Router) -> Result<()> {
        router.ensure_network()?;
        println!(
            "{} Network ready: {}",
            style("✓").green().bold(),
            router.network_name()
        );
        Ok(())
    }

    async fn uninstall(&self, router: &Router) -> Result<()> {
        if !router.is_installed() {
            println!("{} Router not installed", style("!").yellow());
            return Ok(());
        }

        // Stop first
        if router.is_running() {
            router.stop()?;
        }

        // Remove directory
        std::fs::remove_dir_all(router.install_dir())?;
        println!("{} Router uninstalled", style("✓").green().bold());

        Ok(())
    }
}

/// Router names declared by the active daemon's running containers on the
/// router network. A docker failure yields an empty list, which the diagnosis
/// reports as unverified rather than as a fault.
fn declared_router_names(network: &str) -> Vec<String> {
    let ids = std::process::Command::new("docker")
        .args(["ps", "-q", "--filter", &format!("network={network}")])
        .output();
    let Ok(ids) = ids else { return Vec::new() };
    let ids: Vec<String> = String::from_utf8_lossy(&ids.stdout)
        .split_whitespace()
        .map(str::to_string)
        .collect();
    if ids.is_empty() {
        return Vec::new();
    }
    let out = std::process::Command::new("docker")
        .arg("inspect")
        .args(["--format", "{{json .Config.Labels}}"])
        .args(&ids)
        .output();
    let Ok(out) = out else { return Vec::new() };
    let mut names = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let Ok(labels) = serde_json::from_str::<std::collections::BTreeMap<String, String>>(line)
        else {
            continue;
        };
        names.extend(declared_routers(
            labels.iter().map(|(k, v)| (k.as_str(), v.as_str())),
        ));
    }
    names.sort();
    names.dedup();
    names
}

/// Docker-provider routers loaded by whichever Traefik answers on the host
/// dashboard port.
async fn loaded_router_names(port: u16) -> std::result::Result<Vec<String>, String> {
    let url = format!("http://127.0.0.1:{port}/api/http/routers");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("{url} answered {}", resp.status()));
    }
    let routers: Vec<serde_json::Value> = resp.json().await.map_err(|e| e.to_string())?;
    let names: Vec<&str> = routers
        .iter()
        .filter_map(|r| r.get("name").and_then(|n| n.as_str()))
        .collect();
    Ok(docker_provider_routers(names))
}

fn print_router_view(view: &RouterView, port: u16) {
    match view {
        RouterView::Consistent { routers } => println!(
            "  Routing: {} ({routers} router(s) declared on this Docker daemon, all loaded)",
            style("ok").green()
        ),
        RouterView::Unverified => println!(
            "  Routing: {} (no container on this Docker daemon declares a router yet)",
            style("not verified").yellow()
        ),
        RouterView::Blind { missing } => {
            println!(
                "  Routing: {}",
                style("the Traefik on the host ports cannot see this Docker daemon").red()
            );
            println!(
                "    It has loaded none of the {} router(s) declared here: {}",
                missing.len(),
                missing.join(", ")
            );
            println!(
                "    Another Docker daemon (Docker Desktop, OrbStack, colima) is likely running its own"
            );
            println!(
                "    mx-router and holding ports 80 and {port}. Stop that router or that daemon, then run"
            );
            println!("    `mx router restart`. Active context: `docker context show`.");
        }
        RouterView::Partial { missing, seen } => {
            println!(
                "  Routing: {} ({seen} loaded, {} missing: {})",
                style("partial").yellow(),
                missing.len(),
                missing.join(", ")
            );
            println!(
                "    A container may still be starting, or Traefik rejected its labels: `mx router logs`."
            );
        }
        RouterView::ApiUnreachable { detail } => {
            println!(
                "  Routing: {} (dashboard API on port {port} did not answer: {detail})",
                style("unknown").red()
            );
        }
    }
}
