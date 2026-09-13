//! Recipe installer
//!
//! Handles the installation of recipes into projects.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};
use crate::template::{expand_placeholders, TemplateEngine};

use super::{FileMapping, PostInstall, Recipe};

/// Recipe installer
#[derive(Debug)]
pub struct RecipeInstaller {
    /// Templates root directory
    templates_root: PathBuf,
}

impl RecipeInstaller {
    /// Create a new recipe installer
    pub fn new(templates_root: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            templates_root: templates_root.as_ref().to_path_buf(),
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

        // Scaffold the app FIRST. Framework scaffolders (`npm create astro`,
        // `nuxi init`, `zola init`) lay down the app tree themselves and refuse
        // a target that already holds files, so they have to run before the
        // recipe's own `directories` and template payload land. Creating the
        // directories first — which is what mx used to do — made the
        // `skip_if_exists` guard trip on a directory mx had just created, so the
        // scaffolder never ran on any `mx add` and apps shipped with nothing but
        // the recipe's health endpoint (bd:mech-crate-0uq).
        //
        // Recipe files are layered on top afterwards, so mx-specific wiring
        // (health endpoint, README, Docker glue) wins every collision with the
        // framework's starter files.
        if let Some(init_app) = &recipe.init_app {
            result.init_app = Some(self.run_init_app(init_app, project_root, &placeholders)?);
        }

        // Create directories
        for dir_template in &recipe.directories {
            let dir = self.interpolate(dir_template, &placeholders)?;
            let full_path = project_root.join(&dir);

            if !full_path.exists() {
                std::fs::create_dir_all(&full_path)?;
                result.directories_created.push(dir);
            }
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

    /// Substitute the recipe's placeholders in `template`.
    ///
    /// This is deliberately *not* a template rendering pass. Recipe payloads
    /// include app sources (Blade views, Vue SFCs, Zola themes) whose `{{ }}`
    /// and `{% %}` belong to another renderer; a general engine consumed them
    /// and `mx add` failed outright. Only known `{{PLACEHOLDER}}` tokens are
    /// replaced — see [`expand_placeholders`].
    fn interpolate(&mut self, template: &str, vars: &HashMap<String, String>) -> Result<String> {
        Ok(expand_placeholders(template, vars))
    }

    /// Run the recipe's app scaffolder, unless the target already holds an app.
    fn run_init_app(
        &mut self,
        init_app: &super::InitApp,
        project_root: &Path,
        placeholders: &HashMap<String, String>,
    ) -> Result<InitAppOutcome> {
        let target_rel = match &init_app.target_dir {
            Some(t) => Some(self.interpolate(t, placeholders)?),
            None => None,
        };
        let target = target_rel.as_ref().map(|rel| project_root.join(rel));

        if init_app_decision(init_app.skip_if_exists, target.as_deref())
            == InitAppDecision::SkipExisting
        {
            let target_dir = target_rel.unwrap_or_default();
            tracing::info!(
                "Skipping init_app: {} already holds an app",
                project_root.join(&target_dir).display()
            );
            return Ok(InitAppOutcome::SkippedExisting { target_dir });
        }

        // Determine working directory
        let cwd = if let Some(cwd_template) = &init_app.cwd {
            project_root.join(self.interpolate(cwd_template, placeholders)?)
        } else {
            project_root.to_path_buf()
        };

        // Ensure cwd exists. Only the scaffolder's *working* directory is
        // created here — creating its target too is what broke the guard.
        std::fs::create_dir_all(&cwd)?;

        // Interpolate and run command
        let command = self.interpolate(&init_app.command, placeholders)?;
        tracing::info!("Running init command: {}", command);

        let output = Command::new("sh")
            .args(["-c", &command])
            .current_dir(&cwd)
            .output()
            .map_err(|e| {
                Error::CommandFailed(format!("Failed to run init_app `{}`: {}", command, e))
            })?;

        if !output.status.success() {
            // Scaffolders report failures on both streams (npm splits them), and
            // the command + cwd are what a user needs to retry by hand or pass a
            // different `--opt init_cmd=...`.
            return Err(Error::CommandFailed(format!(
                "init_app failed: `{}` in {} exited {}\nstdout: {}\nstderr: {}",
                command,
                cwd.display(),
                output.status,
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim(),
            )));
        }

        // Exit 0 is not proof the scaffolder did anything. `create-astro` and
        // `nuxi init` prompt, and a prompt with no TTY makes create-astro exit 0
        // having written nothing at all — which is how the scaffolding gap
        // survived a "successful" `mx add`. Assert the effect, not the status.
        if let Some(target) = &target {
            if !path_is_occupied(target) {
                return Err(Error::CommandFailed(format!(
                    "init_app wrote nothing: `{}` in {} exited 0 but left {} empty. \
                     Framework scaffolders exit 0 when a prompt hits a non-interactive \
                     shell — the command needs its headless flags (create-astro: --yes, \
                     nuxi: --no-gitInit --packageManager). Override with \
                     `--opt init_cmd=...`.\nstdout: {}\nstderr: {}",
                    command,
                    cwd.display(),
                    target.display(),
                    String::from_utf8_lossy(&output.stdout).trim(),
                    String::from_utf8_lossy(&output.stderr).trim(),
                )));
            }
        }

        Ok(InitAppOutcome::Ran { command })
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
            return Ok(self
                .templates_root
                .join("recipes")
                .join("common")
                .join(rest));
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
                result
                    .files_created
                    .push(dest.to_string_lossy().to_string());
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
            // Read as bytes first, then check if valid UTF-8
            let bytes = std::fs::read(from)?;
            match std::str::from_utf8(&bytes) {
                Ok(content) => {
                    // Process as template
                    let processed = self.interpolate(content, placeholders)?;
                    std::fs::write(to, processed)?;
                }
                Err(_) => {
                    // Not valid UTF-8 — treat as binary, copy as-is
                    tracing::debug!(
                        "Binary content detected (not UTF-8), copying as-is: {}",
                        from.display()
                    );
                    std::fs::write(to, bytes)?;
                }
            }
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

/// Whether a recipe's `init_app` scaffolder should run for a given target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InitAppDecision {
    /// Run the scaffolder.
    Run,
    /// Leave the target alone: it already holds an app.
    SkipExisting,
}

/// `skip_if_exists` asks "is an app already there?", not "does the path exist?".
///
/// mx used to read it as the latter, which made the guard unsatisfiable: the
/// recipe's `directories` list pre-created `apps/<svc>/…`, the guard saw that
/// directory and skipped, so `npm create astro` / `nuxi init` / `zola init` never
/// ran on any `mx add` (bd:mech-crate-0uq). With the scaffolder moved ahead of
/// directory creation, the only question worth asking is whether the target holds
/// files we would clobber — an absent or empty directory is exactly what the
/// scaffolders want, and a populated one is an app a re-run must not overwrite.
fn init_app_decision(skip_if_exists: bool, target: Option<&Path>) -> InitAppDecision {
    if !skip_if_exists {
        return InitAppDecision::Run;
    }
    match target {
        Some(path) if path_is_occupied(path) => InitAppDecision::SkipExisting,
        _ => InitAppDecision::Run,
    }
}

/// True when `path` exists and is anything other than an empty directory.
fn path_is_occupied(path: &Path) -> bool {
    match std::fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_some(),
        // Unreadable or not a directory: a file sitting in the scaffolder's way
        // counts as occupied, a missing path does not.
        Err(_) => path.exists(),
    }
}

/// What the installer did with a recipe's `init_app` scaffolder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitAppOutcome {
    /// The scaffolder ran to completion; carries the command as executed.
    Ran {
        /// Fully interpolated command line handed to `sh -c`.
        command: String,
    },
    /// Skipped because the target already holds an app; carries the
    /// project-relative target directory.
    SkippedExisting {
        /// Project-relative `init_app.target_dir`, interpolated.
        target_dir: String,
    },
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
    /// What happened to the recipe's app scaffolder, when it declares one
    pub init_app: Option<InitAppOutcome>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ── init_app guard (bd:mech-crate-0uq) ───────────────────────────────────

    #[test]
    fn scaffolder_runs_when_the_target_does_not_exist() {
        let temp = TempDir::new().unwrap();
        let target = temp.path().join("apps/svc");
        assert_eq!(
            init_app_decision(true, Some(&target)),
            InitAppDecision::Run,
            "a fresh `mx add` must run the scaffolder"
        );
    }

    #[test]
    fn scaffolder_runs_when_the_target_is_an_empty_directory() {
        // This is the regression: mx pre-created `apps/<svc>` from the recipe's
        // `directories` list and then skipped the scaffolder because the path
        // existed. An empty directory is not an app.
        let temp = TempDir::new().unwrap();
        let target = temp.path().join("apps/svc");
        std::fs::create_dir_all(&target).unwrap();
        assert_eq!(
            init_app_decision(true, Some(&target)),
            InitAppDecision::Run,
            "an empty directory must not count as an existing app"
        );
    }

    #[test]
    fn scaffolder_is_skipped_when_the_target_holds_files() {
        let temp = TempDir::new().unwrap();
        let target = temp.path().join("apps/svc");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("package.json"), "{}").unwrap();
        assert_eq!(
            init_app_decision(true, Some(&target)),
            InitAppDecision::SkipExisting,
            "a populated app dir must never be re-scaffolded over"
        );
    }

    #[test]
    fn scaffolder_is_skipped_when_the_target_holds_only_subdirectories() {
        // Projects scaffolded by the *buggy* mx carry `apps/<svc>/src/...` trees.
        // Re-running `mx add` there must still not re-scaffold.
        let temp = TempDir::new().unwrap();
        let target = temp.path().join("apps/svc");
        std::fs::create_dir_all(target.join("src/pages")).unwrap();
        assert_eq!(
            init_app_decision(true, Some(&target)),
            InitAppDecision::SkipExisting
        );
    }

    #[test]
    fn scaffolder_runs_unconditionally_without_skip_if_exists_or_a_target() {
        let temp = TempDir::new().unwrap();
        let target = temp.path().join("apps/svc");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("package.json"), "{}").unwrap();

        assert_eq!(
            init_app_decision(false, Some(&target)),
            InitAppDecision::Run,
            "skip_if_exists: false means always run"
        );
        assert_eq!(
            init_app_decision(true, None),
            InitAppDecision::Run,
            "no target_dir means there is nothing to guard on"
        );
    }

    /// The ordering itself: the scaffolder must observe a target mx has *not*
    /// pre-created, and its output must survive the directory + template passes.
    #[test]
    fn install_runs_the_scaffolder_before_creating_directories() {
        let temp = TempDir::new().unwrap();
        let templates = temp.path().join("templates");
        let project = temp.path().join("project");
        std::fs::create_dir_all(templates.join("recipes/demo")).unwrap();
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(templates.join("recipes/demo/payload.txt"), "recipe wins\n").unwrap();

        // Recorded-invocation stub: the "scaffolder" writes down what it saw in
        // `apps/` before doing its job, then lays down a framework marker.
        let recipe: Recipe = serde_json::from_value(serde_json::json!({
            "name": "demo",
            "placeholders": { "SERVICE_NAME": { "source": "name" } },
            "init_app": {
                "cwd": "apps",
                "target_dir": "apps/{{SERVICE_NAME}}",
                "skip_if_exists": true,
                "command": "ls -A {{SERVICE_NAME}} > ../saw-before.txt 2>&1 || echo ABSENT > ../saw-before.txt; \
                            mkdir -p {{SERVICE_NAME}} && printf 'scaffolded\\n' > {{SERVICE_NAME}}/package.json"
            },
            "directories": ["apps/{{SERVICE_NAME}}/src/pages"],
            "templates": [{ "from": "payload.txt", "to": "apps/{{SERVICE_NAME}}/src/pages/health.txt" }]
        }))
        .unwrap();

        let mut installer = RecipeInstaller::new(&templates).unwrap();
        let result = installer
            .install(&recipe, &project, "svc", &HashMap::new())
            .unwrap();

        assert_eq!(
            result.init_app,
            Some(InitAppOutcome::Ran {
                command: "ls -A svc > ../saw-before.txt 2>&1 || echo ABSENT > ../saw-before.txt; \
                          mkdir -p svc && printf 'scaffolded\\n' > svc/package.json"
                    .to_string()
            }),
            "install must report that the scaffolder ran"
        );
        assert_eq!(
            std::fs::read_to_string(project.join("saw-before.txt")).unwrap(),
            "ABSENT\n",
            "the scaffolder must see a target mx has not pre-created"
        );
        assert_eq!(
            std::fs::read_to_string(project.join("apps/svc/package.json")).unwrap(),
            "scaffolded\n",
            "scaffolder output must survive the rest of the install"
        );
        assert_eq!(
            std::fs::read_to_string(project.join("apps/svc/src/pages/health.txt")).unwrap(),
            "recipe wins\n",
            "recipe files must layer on top of the scaffolded app"
        );
    }

    /// Re-running `mx add` over a scaffolded app leaves it alone.
    #[test]
    fn install_skips_the_scaffolder_when_the_app_is_already_there() {
        let temp = TempDir::new().unwrap();
        let templates = temp.path().join("templates");
        let project = temp.path().join("project");
        std::fs::create_dir_all(templates.join("recipes/demo")).unwrap();
        std::fs::create_dir_all(project.join("apps/svc")).unwrap();
        std::fs::write(project.join("apps/svc/package.json"), "mine\n").unwrap();

        let recipe: Recipe = serde_json::from_value(serde_json::json!({
            "name": "demo",
            "placeholders": { "SERVICE_NAME": { "source": "name" } },
            "init_app": {
                "cwd": "apps",
                "target_dir": "apps/{{SERVICE_NAME}}",
                "skip_if_exists": true,
                "command": "printf 'clobbered\\n' > {{SERVICE_NAME}}/package.json"
            }
        }))
        .unwrap();

        let mut installer = RecipeInstaller::new(&templates).unwrap();
        let result = installer
            .install(&recipe, &project, "svc", &HashMap::new())
            .unwrap();

        assert_eq!(
            result.init_app,
            Some(InitAppOutcome::SkippedExisting {
                target_dir: "apps/svc".to_string()
            })
        );
        assert_eq!(
            std::fs::read_to_string(project.join("apps/svc/package.json")).unwrap(),
            "mine\n",
            "the existing app must not be clobbered"
        );
    }

    /// A scaffolder that fails is an error, not a silent skip — and it fails
    /// before any recipe file lands, so there is no half-installed service.
    #[test]
    fn install_fails_loudly_when_the_scaffolder_fails() {
        let temp = TempDir::new().unwrap();
        let templates = temp.path().join("templates");
        let project = temp.path().join("project");
        std::fs::create_dir_all(templates.join("recipes/demo")).unwrap();
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(templates.join("recipes/demo/payload.txt"), "x\n").unwrap();

        let recipe: Recipe = serde_json::from_value(serde_json::json!({
            "name": "demo",
            "placeholders": { "SERVICE_NAME": { "source": "name" } },
            "init_app": {
                "cwd": "apps",
                "target_dir": "apps/{{SERVICE_NAME}}",
                "skip_if_exists": true,
                "command": "echo 'no scaffolder here' >&2; exit 3"
            },
            "directories": ["apps/{{SERVICE_NAME}}/src"],
            "templates": [{ "from": "payload.txt", "to": "apps/{{SERVICE_NAME}}/x.txt" }]
        }))
        .unwrap();

        let mut installer = RecipeInstaller::new(&templates).unwrap();
        let err = installer
            .install(&recipe, &project, "svc", &HashMap::new())
            .expect_err("a failing scaffolder must surface");
        let msg = err.to_string();
        assert!(
            msg.contains("init_app failed") && msg.contains("no scaffolder here"),
            "error must quote the scaffolder's own output: {msg}"
        );
        assert!(
            msg.contains('3'),
            "error must carry the scaffolder's exit status: {msg}"
        );
        assert!(
            !project.join("apps/svc/x.txt").exists(),
            "nothing should be installed after a failed scaffold"
        );
    }

    /// The silent-no-op class: `create-astro` hits its git prompt with no TTY and
    /// exits 0 without writing a byte. mx used to call that a success and carry on,
    /// so the app landed with only the recipe's health endpoint and the Docker
    /// build failed later on a missing `package.json`.
    #[test]
    fn install_fails_when_the_scaffolder_exits_zero_but_writes_nothing() {
        let temp = TempDir::new().unwrap();
        let templates = temp.path().join("templates");
        let project = temp.path().join("project");
        std::fs::create_dir_all(templates.join("recipes/demo")).unwrap();
        std::fs::create_dir_all(&project).unwrap();

        let recipe: Recipe = serde_json::from_value(serde_json::json!({
            "name": "demo",
            "placeholders": { "SERVICE_NAME": { "source": "name" } },
            "init_app": {
                "cwd": "apps",
                "target_dir": "apps/{{SERVICE_NAME}}",
                "skip_if_exists": true,
                "command": "echo 'would you like to init a git repo?'; exit 0"
            }
        }))
        .unwrap();

        let mut installer = RecipeInstaller::new(&templates).unwrap();
        let err = installer
            .install(&recipe, &project, "svc", &HashMap::new())
            .expect_err("a scaffolder that writes nothing is not a success");
        let msg = err.to_string();
        assert!(
            msg.contains("wrote nothing") && msg.contains("apps/svc"),
            "error must name the empty target: {msg}"
        );
    }

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
