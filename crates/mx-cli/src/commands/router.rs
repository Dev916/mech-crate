//! `mx router` command - Manage global Traefik router

use anyhow::Result;
use clap::{Args, Subcommand};
use console::style;

use mx_lib::config::MechCrateConfig;
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
            anyhow::bail!(
                "MechCrate not initialized. Run 'mx init' first to install templates."
            );
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
