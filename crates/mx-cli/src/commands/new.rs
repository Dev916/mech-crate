//! `mx new` command - Create a new MechCrate project

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Args;
use console::style;
use dialoguer::MultiSelect;

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

        // Create project directory structure
        self.create_directory_structure(&project_path)?;

        // Copy templates
        self.copy_templates(&templates_root, &project_path)?;

        // Create .gitignore
        self.create_gitignore(&project_path)?;

        // Create README.md
        self.create_readme(&project_path)?;

        // Handle infrastructure setup
        let mut has_cloudflare = false;
        let infra_providers = if !self.no_prompt && self.infra.is_empty() {
            self.prompt_infra_setup()?
        } else {
            self.infra.clone()
        };

        for provider in &infra_providers {
            match provider.as_str() {
                "cloudflare" => {
                    has_cloudflare = true;
                    self.setup_cloudflare_infra(&templates_root, &project_path)?;
                }
                _ => {
                    println!(
                        "  {} {} infrastructure not yet implemented",
                        style("⚠").yellow(),
                        provider
                    );
                }
            }
        }

        // Remove cloudflare.mk if Cloudflare not selected
        if !has_cloudflare {
            let cf_mk = project_path.join("make/cloudflare.mk");
            if cf_mk.exists() {
                std::fs::remove_file(cf_mk)?;
            }
        }

        // Add requested services
        if !self.services.is_empty() {
            let mut installer = RecipeInstaller::new(&templates_root)?;

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
        println!("  make doctor                         # Check dependencies");
        if self.services.is_empty() {
            println!("  mx add <service> --recipe=<recipe>  # Add a service");
        }
        println!("  make dev                            # Start development");

        Ok(())
    }

    fn create_directory_structure(&self, project_path: &Path) -> Result<()> {
        println!("  {} Creating directory structure...", style("→").cyan());

        let dirs = [
            "apps",
            "make",
            "scripts",
            "docker/.config",
            "docker/compose",
            "docker/system",
            "docker/dockerfiles",
            "tmp/up",
        ];

        for dir in dirs {
            std::fs::create_dir_all(project_path.join(dir))?;
        }

        Ok(())
    }

    fn copy_templates(&self, templates_root: &Path, project_path: &Path) -> Result<()> {
        println!("  {} Copying templates...", style("→").cyan());

        // Copy Makefile.template as Makefile
        let makefile_template = templates_root.join("Makefile.template");
        if makefile_template.exists() {
            std::fs::copy(&makefile_template, project_path.join("Makefile"))?;
        }

        // Copy make modules
        let make_dir = templates_root.join("make");
        if make_dir.exists() {
            for entry in std::fs::read_dir(&make_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "mk") {
                    let filename = path.file_name().unwrap();
                    std::fs::copy(&path, project_path.join("make").join(filename))?;
                }
            }
        }

        // Copy scripts (including hidden files like .bashrc)
        let scripts_dir = templates_root.join("scripts");
        if scripts_dir.exists() {
            for entry in std::fs::read_dir(&scripts_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let filename = path.file_name().unwrap();
                    let dest = project_path.join("scripts").join(filename);
                    std::fs::copy(&path, &dest)?;

                    // Make shell scripts executable
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if path.extension().map_or(false, |ext| ext == "sh") {
                            let mut perms = std::fs::metadata(&dest)?.permissions();
                            perms.set_mode(0o755);
                            std::fs::set_permissions(&dest, perms)?;
                        }
                    }
                }
            }
        }

        // Copy shared Docker config files
        println!("  {} Copying shared Docker config...", style("→").cyan());
        let docker_config_dir = templates_root.join("docker/config");
        if docker_config_dir.exists() {
            for entry in std::fs::read_dir(&docker_config_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let filename = path.file_name().unwrap().to_string_lossy();
                    
                    // Only copy shared config files, skip service-specific ones
                    let target_name = match filename.as_ref() {
                        "env.shared" => Some(".env.shared"),
                        "env.secrets.template" => Some(".env.secrets.template"),
                        _ => None,
                    };

                    if let Some(target) = target_name {
                        std::fs::copy(&path, project_path.join("docker/.config").join(target))?;
                    }
                }
            }
        }

        Ok(())
    }

    fn create_gitignore(&self, project_path: &Path) -> Result<()> {
        let content = r#"# MechCrate
tmp/
docker/.compose/
docker/.config/.env.secrets
data/

# Dependencies
node_modules/
vendor/
target/

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log
logs/

# Build artifacts
dist/
build/

# Infrastructure secrets
infra/**/.env
infra/**/*.tfvars
infra/**/*.tfstate
infra/**/*.tfstate.*
infra/**/.terraform/
"#;

        std::fs::write(project_path.join(".gitignore"), content)?;
        Ok(())
    }

    fn create_readme(&self, project_path: &Path) -> Result<()> {
        let content = format!(
            r#"# {name}

A MechCrate project.

## Quick Start

```bash
# Check dependencies
make doctor

# Add a service (pick a recipe, or use the default template)
mx add api --recipe=nuxt

# Start development
make dev

# View logs
make logs

# Stop services
make down
```

## Project Structure

```
{name}/
├── Makefile              # Root makefile
├── apps/                 # Application source code
│   └── <service>/        # Each service's source
│       ├── src/          # Source code
│       ├── package.json  # Dependencies
│       └── ...
├── make/                 # Make modules
│   ├── common.mk         # Shared helpers
│   ├── dev.mk            # Development commands
│   ├── up.mk             # Service management
│   └── ...
├── scripts/              # Shell scripts
│   ├── .bashrc           # Helper functions
│   ├── dev.sh            # Development script
│   └── ...
└── docker/
    ├── .config/          # Environment files
    │   ├── .env.shared   # Shared config
    │   ├── .env.secrets.template  # Secrets template (gitignored secrets created on init)
    │   └── .env.<svc>             # Per-service config (created by mx add)
    ├── compose/          # Compose files
    │   └── <service>.yml / <service>.dev.yml  # Created by mx add / recipes
    ├── system/           # System-level files (configs, etc.)
    │   └── <service>/    # Maps to container /
    │       ├── etc/      # Config files
    │       └── var/      # Log directories
    └── dockerfiles/      # Dockerfiles
        └── <service>/
            └── app       # Dockerfile
```

## Commands

| Command | Description |
|---------|-------------|
| `make dev` | Start all services in dev mode |
| `make dev s=app` | Start specific service in dev mode |
| `make up` | Start services (production mode) |
| `make down` | Stop all services |
| `make logs` | Tail all logs |
| `make logs s=app` | Tail specific service logs |
| `make sh s=app` | Shell into service |
| `make build s=app` | Build service image |
| `make restart s=app` | Restart service |
| `make ps` | List running services |

---
🦝 Built with MechCrate
"#,
            name = self.name
        );

        std::fs::write(project_path.join("README.md"), content)?;
        Ok(())
    }

    fn prompt_infra_setup(&self) -> Result<Vec<String>> {
        let providers = ["Cloudflare", "DigitalOcean", "AWS", "Hetzner", "None"];

        let selection = MultiSelect::new()
            .with_prompt("Select infrastructure providers (space to select, enter to confirm)")
            .items(&providers)
            .interact_opt()?;

        let mut selected_providers = Vec::new();
        
        if let Some(indices) = selection {
            for idx in indices {
                if idx < providers.len() - 1 {
                    // Skip "None"
                    selected_providers.push(providers[idx].to_lowercase());
                }
            }
        }

        Ok(selected_providers)
    }

    fn setup_cloudflare_infra(&self, templates_root: &Path, project_path: &Path) -> Result<()> {
        println!("  {} Setting up Cloudflare infrastructure...", style("→").cyan());

        let cf_template_dir = templates_root.join("infra/cloudflare");
        let cf_project_dir = project_path.join("infra/cloudflare");

        if !cf_template_dir.exists() {
            println!(
                "  {} Cloudflare templates not found at {}",
                style("⚠").yellow(),
                cf_template_dir.display()
            );
            return Ok(());
        }

        // Create infra/cloudflare directory
        std::fs::create_dir_all(&cf_project_dir)?;

        // Copy cloudflare templates recursively
        self.copy_dir_recursive(&cf_template_dir, &cf_project_dir)?;

        // Replace placeholders
        let project_slug = self.name
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();

        self.replace_placeholders_in_dir(&cf_project_dir, "{{PROJECT_NAME}}", &project_slug)?;

        println!("  {} Cloudflare infrastructure added", style("✓").green());
        Ok(())
    }

    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        std::fs::create_dir_all(dst)?;

        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let dest_path = dst.join(entry.file_name());

            if path.is_dir() {
                self.copy_dir_recursive(&path, &dest_path)?;
            } else {
                std::fs::copy(&path, &dest_path)?;
            }
        }

        Ok(())
    }

    fn replace_placeholders_in_dir(&self, dir: &Path, placeholder: &str, value: &str) -> Result<()> {
        let extensions = ["ts", "toml", "json", "md"];

        for entry in walkdir::WalkDir::new(dir) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if extensions.contains(&ext.to_string_lossy().as_ref()) {
                        let content = std::fs::read_to_string(path)?;
                        if content.contains(placeholder) {
                            let new_content = content.replace(placeholder, value);
                            std::fs::write(path, new_content)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
