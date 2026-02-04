//! `mx new` command - Create a new MechCrate project

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Args;
use console::style;
use dialoguer::MultiSelect;
use walkdir::WalkDir;

use mx_lib::{templates_dir, is_initialized};
use mx_lib::recipe::RecipeInstaller;

/// Create a new MechCrate project
#[derive(Args, Debug)]
pub struct NewCommand {
    /// Project name
    name: String,

    /// Services to add (can be specified multiple times)
    #[arg(short, long = "with", value_name = "SERVICE")]
    services: Vec<String>,

    /// Infrastructure providers to include
    #[arg(long, value_name = "PROVIDER")]
    infra: Vec<String>,

    /// Skip interactive prompts
    #[arg(long)]
    no_prompt: bool,
}

impl NewCommand {
    pub async fn run(&self) -> Result<()> {
        let project_path = PathBuf::from(&self.name);

        if project_path.exists() {
            anyhow::bail!("Directory '{}' already exists", self.name);
        }

        println!(
            "{} Creating project: {}",
            style("→").cyan().bold(),
            style(&self.name).green()
        );

        // Check if MechCrate is initialized
        if !is_initialized() {
            anyhow::bail!(
                "MechCrate not initialized. Run 'mx init' first to install templates."
            );
        }

        // Get templates directory
        let templates_root = templates_dir()?;

        // Create project directory
        std::fs::create_dir_all(&project_path)?;

        // Copy base project template
        self.copy_project_template(&templates_root, &project_path)?;

        // Replace placeholders
        self.replace_placeholders(&project_path)?;

        // Add requested services
        if !self.services.is_empty() {
            let installer = RecipeInstaller::new(&templates_root)?;

            for service_spec in &self.services {
                // Parse service_spec as "recipe:name" or just "recipe" (uses recipe name as service name)
                let (recipe_name, service_name) = if let Some((r, s)) = service_spec.split_once(':') {
                    (r, s.to_string())
                } else {
                    (service_spec.as_str(), service_spec.clone())
                };

                println!(
                    "  {} Adding service: {} ({})",
                    style("→").cyan(),
                    style(&service_name).green(),
                    recipe_name
                );

                let recipe = installer.load_recipe(recipe_name)?;
                let options = HashMap::new();
                installer.install(&recipe, &project_path, &service_name, &options)?;
            }
        }

        // Handle infrastructure setup
        if !self.no_prompt && self.infra.is_empty() {
            self.prompt_infra_setup(&project_path)?;
        }

        // Success message
        println!();
        println!(
            "{} Project created: {}",
            style("✓").green().bold(),
            style(&self.name).green()
        );
        println!();
        println!("Next steps:");
        println!("  cd {}", self.name);
        if self.services.is_empty() {
            println!("  mx add <service> --recipe=<recipe>  # Add a service");
        }
        println!("  mx dev                              # Start development");

        Ok(())
    }

    fn copy_project_template(&self, templates_root: &Path, project_path: &Path) -> Result<()> {
        let project_template = templates_root.join("project");

        if !project_template.exists() {
            anyhow::bail!("Project template not found at: {}", project_template.display());
        }

        // Copy all files from template
        for entry in WalkDir::new(&project_template) {
            let entry: walkdir::DirEntry = entry?;
            let relative = entry.path().strip_prefix(&project_template)?;

            if relative.as_os_str().is_empty() {
                continue;
            }

            let dest = project_path.join(relative);

            if entry.file_type().is_dir() {
                std::fs::create_dir_all(&dest)?;
            } else if entry.file_type().is_file() {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(entry.path(), &dest)?;
            }
        }

        Ok(())
    }

    fn replace_placeholders(&self, project_path: &Path) -> Result<()> {
        let placeholders = [
            ("{{PROJECT_NAME}}", &self.name),
            ("{{project_name}}", &self.name),
        ];

        // Files that commonly have placeholders
        let files_to_process = ["Makefile", "README.md", "docker-compose.yml"];

        for file in files_to_process {
            let file_path = project_path.join(file);
            if file_path.exists() {
                let mut content = std::fs::read_to_string(&file_path)?;

                for (placeholder, value) in &placeholders {
                    content = content.replace(placeholder, value);
                }

                std::fs::write(&file_path, content)?;
            }
        }

        Ok(())
    }

    fn prompt_infra_setup(&self, _project_path: &Path) -> Result<()> {
        let providers = ["Cloudflare", "DigitalOcean", "AWS", "Hetzner", "None"];

        let selection = MultiSelect::new()
            .with_prompt("Select infrastructure providers (space to select, enter to confirm)")
            .items(&providers)
            .interact_opt()?;

        if let Some(selected) = selection {
            for idx in selected {
                if idx < providers.len() - 1 {
                    // Skip "None"
                    println!(
                        "  {} {} selected (configure with: mx infra setup {})",
                        style("→").cyan(),
                        providers[idx],
                        providers[idx].to_lowercase()
                    );
                }
            }
        }

        Ok(())
    }
}
