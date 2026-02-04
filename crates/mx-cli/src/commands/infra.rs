//! `mx infra` command - Manage infrastructure providers

use anyhow::Result;
use clap::{Args, Subcommand};
use console::style;
use dialoguer::{Input, Password};

use mx_lib::config::MechCrateConfig;
use mx_lib::infra::{InfraConfig, InfraProvider};
use mx_lib::project::ProjectDetector;

/// Manage infrastructure providers
#[derive(Args, Debug)]
pub struct InfraCommand {
    #[command(subcommand)]
    command: InfraSubcommand,
}

#[derive(Subcommand, Debug)]
enum InfraSubcommand {
    /// Setup a provider
    Setup {
        /// Provider name
        provider: Option<String>,
    },
    /// List configured providers
    List,
    /// Alias for list
    Ls,
    /// Show provider details
    Inspect {
        /// Provider name
        provider: String,
    },
    /// Link project to global config
    Link {
        /// Provider name
        provider: String,
    },
    /// Unlink project from global config
    Unlink {
        /// Provider name
        provider: String,
    },
    /// Remove provider configuration
    Remove {
        /// Provider name
        provider: String,
    },
}

impl InfraCommand {
    pub async fn run(&self) -> Result<()> {
        let config = MechCrateConfig::new()?;

        // Try to get project root, but don't fail if not in a project
        let project_root = ProjectDetector::new().find_root_from_cwd().ok();

        let infra = if let Some(ref root) = project_root {
            InfraConfig::new(config).with_project(root)
        } else {
            InfraConfig::new(config)
        };

        match &self.command {
            InfraSubcommand::Setup { provider } => self.setup(&infra, provider.as_deref()).await,
            InfraSubcommand::List | InfraSubcommand::Ls => self.list(&infra).await,
            InfraSubcommand::Inspect { provider } => self.inspect(&infra, provider).await,
            InfraSubcommand::Link { provider } => self.link(&infra, provider).await,
            InfraSubcommand::Unlink { provider } => self.unlink(&infra, provider).await,
            InfraSubcommand::Remove { provider } => self.remove(&infra, provider).await,
        }
    }

    async fn setup(&self, infra: &InfraConfig, provider_name: Option<&str>) -> Result<()> {
        let provider = match provider_name {
            Some(name) => InfraProvider::from_str(name)
                .ok_or_else(|| anyhow::anyhow!("Unknown provider: {}", name))?,
            None => {
                // Prompt for provider selection
                let providers: Vec<&str> = InfraProvider::all()
                    .iter()
                    .map(|p| p.name())
                    .collect();

                let selection = dialoguer::Select::new()
                    .with_prompt("Select provider to configure")
                    .items(&providers)
                    .default(0)
                    .interact()?;

                InfraProvider::all()[selection]
            }
        };

        println!(
            "{} Setting up {}",
            style("→").cyan().bold(),
            style(provider.name()).green()
        );

        match provider {
            InfraProvider::Cloudflare => self.setup_cloudflare(infra).await,
            InfraProvider::DigitalOcean => self.setup_digitalocean(infra).await,
            InfraProvider::Aws => self.setup_aws(infra).await,
            InfraProvider::Hetzner => self.setup_hetzner(infra).await,
        }
    }

    async fn setup_cloudflare(&self, infra: &InfraConfig) -> Result<()> {
        let account_id: String = Input::new()
            .with_prompt("Cloudflare Account ID")
            .interact_text()?;

        let api_token: String = Password::new()
            .with_prompt("Cloudflare API Token")
            .interact()?;

        let config = format!(
            "# Cloudflare Configuration\n\
             CLOUDFLARE_ACCOUNT_ID={}\n\
             CLOUDFLARE_API_TOKEN={}\n",
            account_id, api_token
        );

        let path = infra.global_config_path(InfraProvider::Cloudflare);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, config)?;

        println!(
            "{} Cloudflare configured: {}",
            style("✓").green().bold(),
            path.display()
        );

        Ok(())
    }

    async fn setup_digitalocean(&self, infra: &InfraConfig) -> Result<()> {
        let api_token: String = Password::new()
            .with_prompt("DigitalOcean API Token")
            .interact()?;

        let spaces_key: String = Input::new()
            .with_prompt("Spaces Access Key (optional)")
            .allow_empty(true)
            .interact_text()?;

        let spaces_secret: String = if !spaces_key.is_empty() {
            Password::new()
                .with_prompt("Spaces Secret Key")
                .interact()?
        } else {
            String::new()
        };

        let mut config = format!(
            "# DigitalOcean Configuration\n\
             DO_API_TOKEN={}\n",
            api_token
        );

        if !spaces_key.is_empty() {
            config.push_str(&format!(
                "DO_SPACES_KEY={}\n\
                 DO_SPACES_SECRET={}\n",
                spaces_key, spaces_secret
            ));
        }

        let path = infra.global_config_path(InfraProvider::DigitalOcean);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, config)?;

        println!(
            "{} DigitalOcean configured: {}",
            style("✓").green().bold(),
            path.display()
        );

        Ok(())
    }

    async fn setup_aws(&self, infra: &InfraConfig) -> Result<()> {
        let access_key: String = Input::new()
            .with_prompt("AWS Access Key ID")
            .interact_text()?;

        let secret_key: String = Password::new()
            .with_prompt("AWS Secret Access Key")
            .interact()?;

        let region: String = Input::new()
            .with_prompt("AWS Region")
            .default("us-east-1".to_string())
            .interact_text()?;

        let config = format!(
            "# AWS Configuration\n\
             AWS_ACCESS_KEY_ID={}\n\
             AWS_SECRET_ACCESS_KEY={}\n\
             AWS_REGION={}\n",
            access_key, secret_key, region
        );

        let path = infra.global_config_path(InfraProvider::Aws);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, config)?;

        println!(
            "{} AWS configured: {}",
            style("✓").green().bold(),
            path.display()
        );

        Ok(())
    }

    async fn setup_hetzner(&self, infra: &InfraConfig) -> Result<()> {
        let api_token: String = Password::new()
            .with_prompt("Hetzner API Token")
            .interact()?;

        let location: String = Input::new()
            .with_prompt("Default Location")
            .default("nbg1".to_string())
            .interact_text()?;

        let config = format!(
            "# Hetzner Configuration\n\
             HETZNER_API_TOKEN={}\n\
             HETZNER_LOCATION={}\n",
            api_token, location
        );

        let path = infra.global_config_path(InfraProvider::Hetzner);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, config)?;

        println!(
            "{} Hetzner configured: {}",
            style("✓").green().bold(),
            path.display()
        );

        Ok(())
    }

    async fn list(&self, infra: &InfraConfig) -> Result<()> {
        let providers = infra.list_configured();

        println!("{}", style("Infrastructure Providers").bold());
        println!("{}", style("─".repeat(40)).dim());

        for (provider, global, project) in &providers {
            let status = if *project {
                style("project").green()
            } else if *global {
                style("global").cyan()
            } else {
                style("not configured").dim()
            };

            println!(
                "  {} {} - {}",
                style("•").cyan(),
                style(provider.name()).bold(),
                status
            );
        }

        Ok(())
    }

    async fn inspect(&self, infra: &InfraConfig, provider_name: &str) -> Result<()> {
        let provider = InfraProvider::from_str(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown provider: {}", provider_name))?;

        let config = infra.load_config(provider)?;

        println!(
            "{} {}",
            style("Configuration for").bold(),
            style(provider.name()).green()
        );
        println!("{}", style("─".repeat(40)).dim());

        for (key, value) in &config {
            // Mask sensitive values
            let display_value = if key.contains("SECRET") || key.contains("TOKEN") || key.contains("KEY") {
                "********".to_string()
            } else {
                value.clone()
            };
            println!("  {} = {}", style(key).cyan(), display_value);
        }

        Ok(())
    }

    async fn link(&self, _infra: &InfraConfig, provider_name: &str) -> Result<()> {
        let _provider = InfraProvider::from_str(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown provider: {}", provider_name))?;

        // Linking requires being in a project
        let detector = ProjectDetector::new();
        let _project_root = detector.find_root_from_cwd()?;

        println!(
            "{} Linking {} (use bash version for now: mx infra link {})",
            style("!").yellow(),
            provider_name,
            provider_name
        );

        Ok(())
    }

    async fn unlink(&self, _infra: &InfraConfig, provider_name: &str) -> Result<()> {
        println!(
            "{} Unlinking {} (use bash version for now: mx infra unlink {})",
            style("!").yellow(),
            provider_name,
            provider_name
        );
        Ok(())
    }

    async fn remove(&self, infra: &InfraConfig, provider_name: &str) -> Result<()> {
        let provider = InfraProvider::from_str(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown provider: {}", provider_name))?;

        let path = infra.global_config_path(provider);
        if path.exists() {
            std::fs::remove_file(&path)?;
            println!(
                "{} Removed {} configuration",
                style("✓").green().bold(),
                provider.name()
            );
        } else {
            println!(
                "{} {} not configured",
                style("!").yellow(),
                provider.name()
            );
        }

        Ok(())
    }
}
