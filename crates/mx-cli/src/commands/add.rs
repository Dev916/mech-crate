//! `mx add` command - Add a service to the project

use std::collections::HashMap;

use anyhow::Result;
use clap::Args;
use console::style;
use dialoguer::Select;

use mx_lib::{templates_dir, is_initialized};
use mx_lib::project::ProjectDetector;
use mx_lib::recipe::RecipeInstaller;

/// Add a service to the project
#[derive(Args, Debug)]
pub struct AddCommand {
    /// Service name
    name: String,

    /// Recipe to use
    #[arg(short, long)]
    recipe: Option<String>,

    /// Custom domain
    #[arg(long)]
    domain: Option<String>,

    /// Additional options (key=value)
    #[arg(long = "opt", value_name = "KEY=VALUE")]
    options: Vec<String>,
}

impl AddCommand {
    pub async fn run(&self) -> Result<()> {
        // Check if MechCrate is initialized
        if !is_initialized() {
            anyhow::bail!(
                "MechCrate not initialized. Run 'mx init' first to install templates."
            );
        }

        // Find project root
        let detector = ProjectDetector::new();
        let project_root = detector.find_root_from_cwd()?;

        println!(
            "{} Adding service: {}",
            style("→").cyan().bold(),
            style(&self.name).green()
        );

        // Get templates directory
        let templates_root = templates_dir()?;
        let installer = RecipeInstaller::new(&templates_root)?;

        // Get recipe
        let recipe_name = match &self.recipe {
            Some(r) => r.clone(),
            None => self.prompt_recipe(&installer)?,
        };

        let recipe = installer.load_recipe(&recipe_name)?;

        // Build options
        let mut option_values = HashMap::new();

        // Add domain if specified
        if let Some(domain) = &self.domain {
            option_values.insert("domain".to_string(), domain.clone());
        }

        // Parse additional options
        for opt in &self.options {
            if let Some((key, value)) = opt.split_once('=') {
                option_values.insert(key.to_string(), value.to_string());
            }
        }

        // Install the recipe
        let result = installer.install(&recipe, &project_root, &self.name, &option_values)?;

        // Print results
        println!();
        println!(
            "{} Service added: {}",
            style("✓").green().bold(),
            style(&self.name).green()
        );

        if !result.directories_created.is_empty() {
            println!();
            println!("Directories created:");
            for dir in &result.directories_created {
                println!("  {}", style(dir).dim());
            }
        }

        if !result.files_created.is_empty() {
            println!();
            println!("Files created:");
            for file in result.files_created.iter().take(10) {
                println!("  {}", style(file).dim());
            }
            if result.files_created.len() > 10 {
                println!("  ... and {} more", result.files_created.len() - 10);
            }
        }

        if !result.next_steps.is_empty() {
            println!();
            println!("Next steps:");
            for step in &result.next_steps {
                println!("  {}", step);
            }
        }

        Ok(())
    }

    fn prompt_recipe(&self, installer: &RecipeInstaller) -> Result<String> {
        let recipes = installer.list_recipes()?;

        if recipes.is_empty() {
            anyhow::bail!("No recipes found");
        }

        let items: Vec<String> = recipes
            .iter()
            .map(|r| format!("{} - {}", r.name, r.display_description()))
            .collect();

        let selection = Select::new()
            .with_prompt("Select a recipe")
            .items(&items)
            .default(0)
            .interact()?;

        Ok(recipes[selection].name.clone())
    }
}
