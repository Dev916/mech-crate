//! Recipe installer
//!
//! Handles the installation of recipes into projects.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};
use crate::template::TemplateEngine;

use super::{Recipe, FileMapping, PostInstall};

/// Recipe installer
#[derive(Debug)]
pub struct RecipeInstaller {
    /// Templates root directory
    templates_root: PathBuf,
    /// Template engine for interpolation
    engine: TemplateEngine,
}

impl RecipeInstaller {
    /// Create a new recipe installer
    pub fn new(templates_root: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            templates_root: templates_root.as_ref().to_path_buf(),
            engine: TemplateEngine::new()?,
        })
    }

    /// Get the recipe directory path
    pub fn recipe_dir(&self, recipe_name: &str) -> PathBuf {
        self.templates_root.join("recipes").join(recipe_name)
    }

    /// List available recipes
    pub fn list_recipes(&self) -> Result<Vec<Recipe>> {
        let recipes_dir = self.templates_root.join("recipes");
        let mut recipes = Vec::new();

        if !recipes_dir.exists() {
            return Ok(recipes);
        }

        for entry in std::fs::read_dir(&recipes_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let recipe_file = path.join("recipe.json");
                if recipe_file.exists() {
                    if let Ok(recipe) = Recipe::load(&recipe_file) {
                        recipes.push(recipe);
                    }
                }
            }
        }

        recipes.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(recipes)
    }

    /// Load a recipe by name
    pub fn load_recipe(&self, name: &str) -> Result<Recipe> {
        let recipe_file = self.recipe_dir(name).join("recipe.json");
        Recipe::load(&recipe_file)
    }

    /// Install a recipe into a project
    pub fn install(
        &mut self,
        recipe: &Recipe,
        project_root: &Path,
        service_name: &str,
        option_values: &HashMap<String, String>,
    ) -> Result<InstallResult> {
        let mut result = InstallResult::default();

        // Build placeholder values
        let placeholders = recipe.build_placeholders(service_name, option_values);

        // Create directories
        for dir_template in &recipe.directories {
            let dir = self.interpolate(dir_template, &placeholders)?;
            let full_path = project_root.join(&dir);

            if !full_path.exists() {
                std::fs::create_dir_all(&full_path)?;
                result.directories_created.push(dir);
            }
        }

        // Run init_app if configured
        if let Some(init_app) = &recipe.init_app {
            self.run_init_app(init_app, project_root, &placeholders)?;
        }

        // Copy template files
        let recipe_dir = self.recipe_dir(&recipe.name);
        for mapping in &recipe.templates {
            self.process_template(
                &recipe_dir,
                project_root,
                mapping,
                &placeholders,
                &mut result,
            )?;
        }

        // Run post-install actions
        if let Some(post_install) = &recipe.post_install {
            self.run_post_install(post_install, project_root, &placeholders)?;
        }

        // Interpolate next steps
        for step in &recipe.next_steps {
            let interpolated = self.interpolate(step, &placeholders)?;
            result.next_steps.push(interpolated);
        }

        Ok(result)
    }

    /// Interpolate template variables
    fn interpolate(&mut self, template: &str, vars: &HashMap<String, String>) -> Result<String> {
        self.engine.render_string(template, vars)
    }

    /// Run the init_app command
    fn run_init_app(
        &mut self,
        init_app: &super::InitApp,
        project_root: &Path,
        placeholders: &HashMap<String, String>,
    ) -> Result<()> {
        // Check if target exists and skip_if_exists is true
        if init_app.skip_if_exists {
            if let Some(target_dir) = &init_app.target_dir {
                let target = project_root.join(self.interpolate(target_dir, placeholders)?);
                if target.exists() {
                    tracing::info!("Skipping init_app: {} already exists", target.display());
                    return Ok(());
                }
            }
        }

        // Determine working directory
        let cwd = if let Some(cwd_template) = &init_app.cwd {
            project_root.join(self.interpolate(cwd_template, placeholders)?)
        } else {
            project_root.to_path_buf()
        };

        // Ensure cwd exists
        std::fs::create_dir_all(&cwd)?;

        // Interpolate and run command
        let command = self.interpolate(&init_app.command, placeholders)?;
        tracing::info!("Running init command: {}", command);

        let output = Command::new("sh")
            .args(["-c", &command])
            .current_dir(&cwd)
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to run init_app: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!(
                "init_app failed: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Process a single template mapping
    fn process_template(
        &mut self,
        recipe_dir: &Path,
        project_root: &Path,
        mapping: &FileMapping,
        placeholders: &HashMap<String, String>,
        result: &mut InstallResult,
    ) -> Result<()> {
        let from_path = self.resolve_template_source(recipe_dir, &mapping.from)?;
        let to_template = self.interpolate(&mapping.to, placeholders)?;
        let to_path = project_root.join(&to_template);

        if from_path.is_dir() {
            self.copy_directory(&from_path, &to_path, placeholders, result)?;
        } else if from_path.is_file() {
            self.copy_file(&from_path, &to_path, placeholders)?;
            result.files_created.push(to_template);
        } else {
            tracing::warn!("Template source not found: {}", from_path.display());
        }

        Ok(())
    }

    /// Resolve template source path, handling namespace references
    fn resolve_template_source(&self, recipe_dir: &Path, source: &str) -> Result<PathBuf> {
        // Handle namespace references like "common://path/to/file"
        if let Some(rest) = source.strip_prefix("common://") {
            return Ok(self.templates_root.join("recipes").join("common").join(rest));
        }

        // Regular path relative to recipe directory
        Ok(recipe_dir.join(source))
    }

    /// Copy a directory recursively
    fn copy_directory(
        &mut self,
        from: &Path,
        to: &Path,
        placeholders: &HashMap<String, String>,
        result: &mut InstallResult,
    ) -> Result<()> {
        std::fs::create_dir_all(to)?;

        for entry in walkdir::WalkDir::new(from) {
            let entry = entry.map_err(|e| Error::Io(e.into()))?;
            let relative = entry.path().strip_prefix(from).unwrap();

            // Interpolate the relative path
            let relative_str = relative.to_string_lossy();
            let interpolated_relative = self.interpolate(&relative_str, placeholders)?;
            let dest = to.join(&interpolated_relative);

            if entry.file_type().is_dir() {
                std::fs::create_dir_all(&dest)?;
            } else if entry.file_type().is_file() {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                self.copy_file(entry.path(), &dest, placeholders)?;
                result.files_created.push(dest.to_string_lossy().to_string());
            }
        }

        Ok(())
    }

    /// Copy a single file, optionally processing as template
    fn copy_file(
        &mut self,
        from: &Path,
        to: &Path,
        placeholders: &HashMap<String, String>,
    ) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Check if this is a binary file
        if TemplateEngine::is_binary_file(from) {
            std::fs::copy(from, to)?;
        } else {
            // Process as template
            let content = std::fs::read_to_string(from)?;
            let processed = self.interpolate(&content, placeholders)?;
            std::fs::write(to, processed)?;
        }

        // Preserve executable permission
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let source_meta = std::fs::metadata(from)?;
            let source_mode = source_meta.permissions().mode();
            if source_mode & 0o111 != 0 {
                let dest_perms = std::fs::Permissions::from_mode(source_mode);
                std::fs::set_permissions(to, dest_perms)?;
            }
        }

        Ok(())
    }

    /// Run post-install actions
    fn run_post_install(
        &mut self,
        post_install: &PostInstall,
        project_root: &Path,
        placeholders: &HashMap<String, String>,
    ) -> Result<()> {
        // Create files
        for create_file in &post_install.create_files {
            let path = project_root.join(self.interpolate(&create_file.path, placeholders)?);
            let content = self.interpolate(&create_file.content, placeholders)?;

            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, content)?;
        }

        // Rename files
        for rename in &post_install.rename {
            let from = project_root.join(self.interpolate(&rename.from, placeholders)?);
            let to = project_root.join(self.interpolate(&rename.to, placeholders)?);

            if from.exists() {
                std::fs::rename(&from, &to)?;
            }
        }

        // Make files executable
        #[cfg(unix)]
        for chmod in &post_install.chmod {
            let path = project_root.join(self.interpolate(&chmod.path, placeholders)?);
            if path.exists() {
                use std::os::unix::fs::PermissionsExt;
                let meta = std::fs::metadata(&path)?;
                let mut perms = meta.permissions();
                perms.set_mode(perms.mode() | 0o111);
                std::fs::set_permissions(&path, perms)?;
            }
        }

        // Create .gitkeep in empty directories
        for dir_template in &post_install.gitkeep {
            let dir = project_root.join(self.interpolate(dir_template, placeholders)?);
            std::fs::create_dir_all(&dir)?;
            let gitkeep = dir.join(".gitkeep");
            if !gitkeep.exists() {
                std::fs::write(&gitkeep, "")?;
            }
        }

        // Run commands
        for run in &post_install.run {
            let command = self.interpolate(&run.command, placeholders)?;
            let cwd = if let Some(cwd_template) = &run.cwd {
                project_root.join(self.interpolate(cwd_template, placeholders)?)
            } else {
                project_root.to_path_buf()
            };

            tracing::info!("Running post-install command: {}", command);
            let output = Command::new("sh")
                .args(["-c", &command])
                .current_dir(&cwd)
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!("Post-install command failed: {}", stderr);
            }
        }

        // Update .gitignore
        if !post_install.gitignore.is_empty() {
            let gitignore_path = project_root.join(".gitignore");
            let mut content = if gitignore_path.exists() {
                std::fs::read_to_string(&gitignore_path)?
            } else {
                String::new()
            };

            for pattern in &post_install.gitignore {
                let interpolated = self.interpolate(pattern, placeholders)?;
                if !content.contains(&interpolated) {
                    if !content.ends_with('\n') && !content.is_empty() {
                        content.push('\n');
                    }
                    content.push_str(&interpolated);
                    content.push('\n');
                }
            }

            std::fs::write(&gitignore_path, content)?;
        }

        Ok(())
    }
}

/// Result of a recipe installation
#[derive(Debug, Default)]
pub struct InstallResult {
    /// Directories that were created
    pub directories_created: Vec<String>,
    /// Files that were created
    pub files_created: Vec<String>,
    /// Next steps for the user
    pub next_steps: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_interpolate() {
        let temp = TempDir::new().unwrap();
        let mut installer = RecipeInstaller::new(temp.path()).unwrap();

        let mut vars = HashMap::new();
        vars.insert("SERVICE_NAME".to_string(), "my-app".to_string());

        let result = installer
            .interpolate("apps/{{ SERVICE_NAME }}/src", &vars)
            .unwrap();
        assert_eq!(result, "apps/my-app/src");
    }
}
