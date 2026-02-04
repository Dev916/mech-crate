//! `mx unyform/login/logout/whoami` commands - Unyform integration

use anyhow::Result;
use clap::{Args, Subcommand};
use console::style;
use dialoguer::{Input, Password, Select};

use mx_lib::unyform::UnyformClient;

/// Unyform integration
#[derive(Args, Debug)]
pub struct UnyformCommand {
    #[command(subcommand)]
    command: UnyformSubcommand,
}

#[derive(Subcommand, Debug)]
enum UnyformSubcommand {
    /// Login to Unyform
    Login(LoginCommand),
    /// Logout from Unyform
    Logout(LogoutCommand),
    /// Show current user
    Whoami(WhoamiCommand),
}

impl UnyformCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            UnyformSubcommand::Login(cmd) => cmd.run().await,
            UnyformSubcommand::Logout(cmd) => cmd.run().await,
            UnyformSubcommand::Whoami(cmd) => cmd.run().await,
        }
    }
}

/// Login to Unyform
#[derive(Args, Debug)]
pub struct LoginCommand {
    /// API key (for CI/automation)
    #[arg(long)]
    api_key: Option<String>,

    /// Custom Unyform URL
    #[arg(long)]
    url: Option<String>,

    /// Use browser OAuth (not implemented in CLI)
    #[arg(long)]
    browser: bool,
}

impl LoginCommand {
    pub async fn run(&self) -> Result<()> {
        let unyform = UnyformClient::new();

        if unyform.is_logged_in() {
            println!(
                "{} Already logged in. Use 'mx logout' first to change accounts.",
                style("!").yellow()
            );
            return Ok(());
        }

        // If API key provided via flag, use it directly
        if let Some(ref api_key) = self.api_key {
            return self.login_with_key(&unyform, api_key, self.url.as_deref()).await;
        }

        // Browser OAuth not fully implemented in Rust yet
        if self.browser {
            println!(
                "{} Browser OAuth not yet implemented in Rust CLI",
                style("!").yellow()
            );
            println!("  Use the bash version: mx login --browser");
            return Ok(());
        }

        // Interactive login
        let options = ["API Key", "Browser OAuth (requires bash mx)"];
        let selection = Select::new()
            .with_prompt("Select login method")
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            0 => {
                let api_key: String = Password::new()
                    .with_prompt("API Key")
                    .interact()?;

                let url: String = Input::new()
                    .with_prompt("Unyform URL")
                    .default(UnyformClient::DEFAULT_URL.to_string())
                    .interact_text()?;

                self.login_with_key(&unyform, &api_key, Some(&url)).await
            }
            _ => {
                println!(
                    "{} Use bash version for browser OAuth: mx login --browser",
                    style("→").cyan()
                );
                Ok(())
            }
        }
    }

    async fn login_with_key(&self, unyform: &UnyformClient, api_key: &str, url: Option<&str>) -> Result<()> {
        println!("{} Authenticating...", style("→").cyan().bold());

        let user = unyform.login_with_api_key(api_key, url).await?;

        println!(
            "{} Logged in as: {} ({})",
            style("✓").green().bold(),
            style(&user.name).green(),
            user.email
        );

        if !user.organizations.is_empty() {
            println!();
            println!("Organizations:");
            for org in &user.organizations {
                println!("  {} {} ({})", style("•").cyan(), org.name, org.role);
            }
        }

        Ok(())
    }
}

/// Logout from Unyform
#[derive(Args, Debug)]
pub struct LogoutCommand;

impl LogoutCommand {
    pub async fn run(&self) -> Result<()> {
        let unyform = UnyformClient::new();

        if !unyform.is_logged_in() {
            println!("{} Not logged in", style("!").yellow());
            return Ok(());
        }

        unyform.logout()?;
        println!("{} Logged out", style("✓").green().bold());

        Ok(())
    }
}

/// Show current Unyform user
#[derive(Args, Debug)]
pub struct WhoamiCommand;

impl WhoamiCommand {
    pub async fn run(&self) -> Result<()> {
        let unyform = UnyformClient::new();

        if !unyform.is_logged_in() {
            println!("{} Not logged in", style("!").yellow());
            println!("  Login with: mx login");
            return Ok(());
        }

        println!("{} Fetching user info...", style("→").cyan());

        let user = unyform.whoami().await?;

        println!();
        println!("{}", style("Unyform Account").bold());
        println!("{}", style("─".repeat(40)).dim());
        println!("  Email: {}", user.email);
        println!("  Name: {}", user.name);

        if !user.organizations.is_empty() {
            println!();
            println!("Organizations:");
            for org in &user.organizations {
                println!(
                    "  {} {} ({}) - {}",
                    style("•").cyan(),
                    org.name,
                    style(&org.slug).dim(),
                    org.role
                );
            }
        }

        Ok(())
    }
}
