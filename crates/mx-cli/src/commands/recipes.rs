//! `mx recipes` command - Manage recipes

use anyhow::Result;
use clap::{Args, Subcommand};
use console::style;

use mx_lib::{templates_dir, is_initialized};
use mx_lib::project::ProjectDetector;
use mx_lib::recipe::RecipeInstaller;
use mx_lib::unyform::UnyformClient;

/// Manage recipes
#[derive(Args, Debug)]
pub struct RecipesCommand {
    #[command(subcommand)]
    command: Option<RecipesSubcommand>,
}

#[derive(Subcommand, Debug)]
enum RecipesSubcommand {
    /// List available recipes
    List,
    /// Alias for list
    Ls,
    /// Show recipe details
    Info {
        /// Recipe name
        name: String,
    },
    /// Pull a recipe from Unyform
    Pull {
        /// Recipe name (optionally with @version)
        name: String,
    },
    /// Apply a recipe to the current project
    Apply {
        /// Recipe name (optionally with @version)
        name: String,
        /// Auto-fix dependency drift
        #[arg(long)]
        fix: bool,
    },
    /// List available versions for a recipe
    Versions {
        /// Recipe name
        name: String,
    },
    /// Manage cached recipes
    Cache {
        /// Action: list, clear
        action: Option<String>,
    },
}

impl RecipesCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            Some(RecipesSubcommand::List) | Some(RecipesSubcommand::Ls) | None => {
                self.list_recipes().await
            }
            Some(RecipesSubcommand::Info { name }) => self.show_info(name).await,
            Some(RecipesSubcommand::Pull { name }) => self.pull_recipe(name).await,
            Some(RecipesSubcommand::Apply { name, fix }) => self.apply_recipe(name, *fix).await,
            Some(RecipesSubcommand::Versions { name }) => self.show_versions(name).await,
            Some(RecipesSubcommand::Cache { action }) => {
                self.manage_cache(action.as_deref().unwrap_or("list")).await
            }
        }
    }

    async fn list_recipes(&self) -> Result<()> {
        if !is_initialized() {
            anyhow::bail!(
                "MechCrate not initialized. Run 'mx init' first to install templates."
            );
        }

        let templates_root = templates_dir()?;
        let installer = RecipeInstaller::new(&templates_root)?;
        let recipes = installer.list_recipes()?;

        println!("{}", style("Local Recipes").bold());
        println!("{}", style("─".repeat(40)).dim());

        if recipes.is_empty() {
            println!("  No local recipes found");
        } else {
            for recipe in &recipes {
                println!(
                    "  {} {} - {}",
                    style("•").cyan(),
                    style(&recipe.name).green(),
                    recipe.display_description()
                );
            }
        }

        // Try to list remote recipes if logged in
        let unyform = UnyformClient::new();
        if unyform.is_logged_in() {
            println!();
            println!("{}", style("Organization Recipes").bold());
            println!("{}", style("─".repeat(40)).dim());

            match unyform.list_recipes().await {
                Ok(response) => {
                    if response.recipes.is_empty() {
                        println!("  No organization recipes found");
                    } else {
                        for recipe in &response.recipes {
                            println!(
                                "  {} {} @ v{} - {}",
                                style("•").magenta(),
                                style(&recipe.name).green(),
                                recipe.version,
                                recipe.description.as_deref().unwrap_or("No description")
                            );
                        }
                    }
                }
                Err(e) => {
                    println!("  {} Failed to fetch: {}", style("!").yellow(), e);
                }
            }
        }

        Ok(())
    }

    async fn show_info(&self, name: &str) -> Result<()> {
        if !is_initialized() {
            anyhow::bail!(
                "MechCrate not initialized. Run 'mx init' first to install templates."
            );
        }

        let templates_root = templates_dir()?;
        let installer = RecipeInstaller::new(&templates_root)?;
        let recipe = installer.load_recipe(name)?;

        println!("{}", style(&recipe.display_title()).bold());
        println!("{}", style("─".repeat(40)).dim());
        println!();
        println!("{}", recipe.display_description());

        if !recipe.features.is_empty() {
            println!();
            println!("{}", style("Features:").bold());
            for feature in &recipe.features {
                println!("  {} {}", style("•").cyan(), feature);
            }
        }

        if !recipe.options.is_empty() {
            println!();
            println!("{}", style("Options:").bold());
            for (name, opt) in &recipe.options {
                let default = opt.default.as_deref().unwrap_or("(none)");
                let desc = opt.description.as_deref().unwrap_or("");
                println!(
                    "  {} {} = {} - {}",
                    style("--").dim(),
                    style(name).green(),
                    style(default).yellow(),
                    desc
                );
            }
        }

        if !recipe.services.is_empty() {
            println!();
            println!("{}", style("Services:").bold());
            for service in &recipe.services {
                let desc = service.description.as_deref().unwrap_or("");
                println!("  {} {} - {}", style("•").cyan(), service.name, desc);
            }
        }

        Ok(())
    }

    async fn pull_recipe(&self, name_spec: &str) -> Result<()> {
        let (name, version) = Self::parse_name_version(name_spec);

        let unyform = UnyformClient::new();
        if !unyform.is_logged_in() {
            anyhow::bail!("Not logged in. Run 'mx login' first.");
        }

        println!(
            "{} Pulling recipe: {}{}",
            style("→").cyan().bold(),
            style(name).green(),
            version.map(|v| format!("@{}", v)).unwrap_or_default()
        );

        let recipe = unyform.get_recipe(name, version).await?;
        let org = unyform.get_default_org()?;
        let cache_path = unyform.cache_recipe(&org, &recipe)?;

        println!(
            "{} Recipe cached: {}",
            style("✓").green().bold(),
            cache_path.display()
        );

        Ok(())
    }

    async fn apply_recipe(&self, name_spec: &str, _fix: bool) -> Result<()> {
        let (name, version) = Self::parse_name_version(name_spec);

        let detector = ProjectDetector::new();
        let project_root = detector.find_root_from_cwd()?;

        let unyform = UnyformClient::new();
        if !unyform.is_logged_in() {
            anyhow::bail!("Not logged in. Run 'mx login' first.");
        }

        println!(
            "{} Applying recipe: {}{}",
            style("→").cyan().bold(),
            style(name).green(),
            version.map(|v| format!("@{}", v)).unwrap_or_default()
        );

        let recipe = unyform.get_recipe(name, version).await?;

        // Create coding rules from patterns
        let rules_dir = project_root.join(".cursor").join("rules");
        std::fs::create_dir_all(&rules_dir)?;

        let rules_file = rules_dir.join(format!("{}-patterns.md", name));
        let mut rules_content = format!("# {} Coding Patterns\n\n", recipe.name);
        rules_content.push_str("Generated from organizational recipe.\n\n");

        for pattern in &recipe.patterns {
            if let Some(obj) = pattern.as_object() {
                if let (Some(name), Some(desc)) = (obj.get("name"), obj.get("description")) {
                    rules_content.push_str(&format!(
                        "## {}\n\n{}\n\n",
                        name.as_str().unwrap_or(""),
                        desc.as_str().unwrap_or("")
                    ));

                    if let Some(rules) = obj.get("rules").and_then(|r| r.as_array()) {
                        rules_content.push_str("### Rules\n\n");
                        for rule in rules {
                            if let Some(r) = rule.as_str() {
                                rules_content.push_str(&format!("- {}\n", r));
                            }
                        }
                        rules_content.push('\n');
                    }
                }
            }
        }

        std::fs::write(&rules_file, &rules_content)?;

        println!(
            "{} Created: {}",
            style("✓").green(),
            rules_file.display()
        );
        println!(
            "{} Applied {} coding patterns",
            style("✓").green().bold(),
            recipe.patterns.len()
        );

        Ok(())
    }

    async fn show_versions(&self, name: &str) -> Result<()> {
        let unyform = UnyformClient::new();
        if !unyform.is_logged_in() {
            anyhow::bail!("Not logged in. Run 'mx login' first.");
        }

        let response = unyform.get_recipe_versions(name).await?;

        println!("{} {}", style("Versions for").bold(), style(name).green());
        println!("{}", style("─".repeat(40)).dim());

        for version in &response.versions {
            let latest = if version.is_latest {
                style(" (latest)").yellow()
            } else {
                style("").dim()
            };
            println!(
                "  {} v{}{} - {}",
                style("•").cyan(),
                version.version,
                latest,
                version.generated_at
            );
        }

        Ok(())
    }

    async fn manage_cache(&self, action: &str) -> Result<()> {
        let unyform = UnyformClient::new();

        match action {
            "clear" => {
                unyform.clear_cache()?;
                println!("{} Recipe cache cleared", style("✓").green().bold());
            }
            _ => {
                let cached = unyform.list_cached_recipes()?;

                println!("{}", style("Cached Recipes").bold());
                println!("{}", style("─".repeat(40)).dim());

                if cached.is_empty() {
                    println!("  No cached recipes");
                    println!();
                    println!("  Pull recipes with: mx recipes pull <name>");
                } else {
                    for (org, name, versions) in &cached {
                        println!(
                            "  {} {}/{}: {}",
                            style("•").cyan(),
                            org,
                            style(name).green(),
                            versions.join(", ")
                        );
                    }
                }
            }
        }

        Ok(())
    }

    fn parse_name_version(spec: &str) -> (&str, Option<&str>) {
        if let Some((name, version)) = spec.split_once('@') {
            (name, Some(version))
        } else {
            (spec, None)
        }
    }
}
