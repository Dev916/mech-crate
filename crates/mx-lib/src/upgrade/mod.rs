//! Project upgrade functionality
//!
//! Handles upgrading MechCrate projects with latest scaffolding.

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::error::{Error, Result};
use crate::paths;

/// File category for upgrade decisions
#[derive(Debug, Clone, PartialEq)]
pub enum FileCategory {
    /// Tooling files - prompt for updates when different
    Tooling,
    /// Config files - add if missing, never update
    Config,
    /// Conditional files - only process if feature enabled
    Conditional(String),
    /// Skip these files
    Skip,
}

/// Upgrade action for a file
#[derive(Debug, Clone)]
pub enum UpgradeAction {
    /// Add missing file
    Add,
    /// Update existing file (differs from template)
    Update,
    /// File is current (matches template)
    Current,
    /// Skip this file (config exists, etc.)
    Skip,
}

/// An upgrade entry representing a file comparison
#[derive(Debug, Clone)]
pub struct UpgradeEntry {
    pub action: UpgradeAction,
    pub project_path: PathBuf,
    pub template_path: PathBuf,
    pub category: FileCategory,
}

/// Project upgrader
#[derive(Debug)]
pub struct ProjectUpgrader {
    templates_dir: PathBuf,
    project_dir: PathBuf,
}

impl ProjectUpgrader {
    /// Create a new upgrader
    pub fn new(project_dir: impl AsRef<Path>) -> Result<Self> {
        let templates_dir = paths::templates_dir()?;
        Ok(Self {
            templates_dir,
            project_dir: project_dir.as_ref().to_path_buf(),
        })
    }

    /// Categorize a template file path
    pub fn categorize_file(&self, rel_path: &str) -> FileCategory {
        match rel_path {
            // Tooling files - prompt for updates
            path if path.starts_with("make/") && path.ends_with(".mk") => {
                if path == "make/cloudflare.mk" {
                    FileCategory::Conditional("cloudflare".to_string())
                } else {
                    FileCategory::Tooling
                }
            }
            path if path.starts_with("scripts/")
                && (path.ends_with(".sh") || path.ends_with(".mjs")) =>
            {
                if path.starts_with("scripts/cf-") {
                    FileCategory::Conditional("cloudflare".to_string())
                } else {
                    FileCategory::Tooling
                }
            }
            "Makefile.template" => FileCategory::Tooling,

            // Config files - add only, never update
            path if path.starts_with("docker/compose/") => FileCategory::Config,
            path if path.starts_with("docker/config/") => FileCategory::Config,
            path if path.starts_with("docker/system/") => FileCategory::Config,
            path if path.starts_with("docker/dockerfiles/") => FileCategory::Config,

            // Infrastructure templates - conditional
            path if path.starts_with("infra/cloudflare/") => {
                FileCategory::Conditional("cloudflare".to_string())
            }

            // Skip recipes and other non-scaffold files
            path if path.starts_with("recipes/") => FileCategory::Skip,
            path if path.starts_with("router/") => FileCategory::Skip,
            path if path.starts_with("project/") => FileCategory::Skip,
            _ => FileCategory::Skip,
        }
    }

    /// Check if a conditional feature is enabled in the project
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "cloudflare" => self.project_dir.join("infra/cloudflare").exists(),
            _ => false,
        }
    }

    /// Map template path to project path
    pub fn template_to_project_path(&self, template_rel: &str) -> PathBuf {
        match template_rel {
            "Makefile.template" => PathBuf::from("Makefile"),
            path if path.starts_with("docker/config/") => {
                let filename = Path::new(path).file_name().unwrap().to_str().unwrap();
                let target_name = filename.replace("env.", ".env.");
                PathBuf::from("docker/.config").join(target_name)
            }
            _ => PathBuf::from(template_rel),
        }
    }

    /// Check if two files differ
    pub fn files_differ(&self, project_file: &Path, template_file: &Path) -> bool {
        if !project_file.exists() || !template_file.exists() {
            return false;
        }

        match (std::fs::read(project_file), std::fs::read(template_file)) {
            (Ok(a), Ok(b)) => a != b,
            _ => false,
        }
    }

    /// Discover all upgrade entries
    pub fn discover_upgrades(&self) -> Result<Vec<UpgradeEntry>> {
        let mut entries = Vec::new();
        let templates_project_dir = self.templates_dir.join("project");

        if !templates_project_dir.exists() {
            return Err(Error::Config(format!(
                "Project templates not found at {}",
                templates_project_dir.display()
            )));
        }

        for entry in WalkDir::new(&templates_project_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let rel_path = entry
                .path()
                .strip_prefix(&templates_project_dir)
                .map_err(|e| Error::Other(e.to_string()))?
                .to_str()
                .ok_or_else(|| Error::Other("Invalid path".into()))?;

            let category = self.categorize_file(rel_path);

            // Skip files marked for skipping
            if category == FileCategory::Skip {
                continue;
            }

            // Check conditional features
            let effective_category = match &category {
                FileCategory::Conditional(feature) => {
                    if self.is_feature_enabled(feature) {
                        FileCategory::Tooling
                    } else {
                        continue; // Skip if feature not enabled
                    }
                }
                other => other.clone(),
            };

            let project_path = self
                .project_dir
                .join(self.template_to_project_path(rel_path));
            let template_path = entry.path().to_path_buf();

            let action = if !project_path.exists() {
                UpgradeAction::Add
            } else if effective_category == FileCategory::Tooling {
                if self.files_differ(&project_path, &template_path) {
                    UpgradeAction::Update
                } else {
                    UpgradeAction::Current
                }
            } else {
                UpgradeAction::Skip
            };

            entries.push(UpgradeEntry {
                action,
                project_path,
                template_path,
                category: effective_category,
            });
        }

        Ok(entries)
    }

    /// Copy a file from template to project
    pub fn copy_file(&self, template_path: &Path, project_path: &Path) -> Result<()> {
        if let Some(parent) = project_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::copy(template_path, project_path)?;

        // Make shell scripts executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if project_path.extension().map(|e| e == "sh").unwrap_or(false) {
                std::fs::set_permissions(project_path, std::fs::Permissions::from_mode(0o755))?;
            }
        }

        Ok(())
    }

    /// Backup path used before overwriting an existing project file.
    ///
    /// Appends `.bak` after the existing extension, so `setup.sh` backs up to
    /// `setup.sh.bak`. Extension-less files (e.g. `Makefile`) currently pick up
    /// a doubled dot — preserved verbatim from the original call site so this
    /// extraction changes no behavior.
    pub fn backup_path(&self, project_path: &Path) -> PathBuf {
        project_path.with_extension(format!(
            "{}.bak",
            project_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
        ))
    }

    /// Get the diff between two files
    pub fn file_diff(&self, project_path: &Path, template_path: &Path) -> Result<String> {
        let project_content = std::fs::read_to_string(project_path)?;
        let template_content = std::fs::read_to_string(template_path)?;

        let diff = similar::TextDiff::from_lines(&project_content, &template_content);
        let mut output = String::new();

        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                similar::ChangeTag::Delete => "-",
                similar::ChangeTag::Insert => "+",
                similar::ChangeTag::Equal => " ",
            };
            output.push_str(&format!("{}{}", sign, change));
        }

        Ok(output)
    }

    /// Get required directories for a project
    pub fn required_directories(&self) -> Vec<&'static str> {
        vec![
            "make",
            "scripts",
            "docker/.config",
            "docker/compose",
            "docker/system",
            "docker/dockerfiles",
            "tmp/up",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an upgrader over synthetic dirs, bypassing `paths::templates_dir()`
    /// (which resolves against the machine's install). The fields are private,
    /// so only this child module can do it — exactly the seam we want.
    fn upgrader(templates_dir: &Path, project_dir: &Path) -> ProjectUpgrader {
        ProjectUpgrader {
            templates_dir: templates_dir.to_path_buf(),
            project_dir: project_dir.to_path_buf(),
        }
    }

    fn write(root: &Path, rel: &str, contents: &str) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    /// A synthetic `templates/project/` tree covering every categorization arm.
    fn synthetic_templates(root: &Path) {
        let p = root.join("project");
        // Tooling
        write(&p, "make/dev.mk", "dev-mk-v2\n");
        write(&p, "scripts/setup.sh", "setup-v2\n");
        write(&p, "Makefile.template", "makefile-v2\n");
        // Conditional (cloudflare)
        write(&p, "make/cloudflare.mk", "cf-mk-v2\n");
        write(&p, "scripts/cf-deploy.sh", "cf-deploy-v2\n");
        write(&p, "infra/cloudflare/wrangler.toml", "cf-toml-v2\n");
        // Config (add-only)
        write(&p, "docker/compose/app.yml", "compose-v2\n");
        write(&p, "docker/config/env.app", "env-app-v2\n");
        write(&p, "docker/system/traefik.yml", "system-v2\n");
        write(&p, "docker/dockerfiles/Dockerfile.app", "dockerfile-v2\n");
        // Skip
        write(&p, "recipes/astro/recipe.json", "{}\n");
        write(&p, "router/docker-compose.yml", "router-v2\n");
        write(&p, "project/nested.txt", "nested\n");
        write(&p, "README.md", "readme\n");
    }

    fn entry_for<'a>(entries: &'a [UpgradeEntry], template_rel: &str) -> Option<&'a UpgradeEntry> {
        entries
            .iter()
            .find(|e| e.template_path.to_string_lossy().ends_with(template_rel))
    }

    // ── Categorization matrix ────────────────────────────────────────────

    #[test]
    fn upgrade_categorization_matrix() {
        let dir = tempfile::tempdir().unwrap();
        let up = upgrader(dir.path(), dir.path());

        let cf = FileCategory::Conditional("cloudflare".to_string());
        let cases: &[(&str, FileCategory)] = &[
            // tooling → prompt-update
            ("make/dev.mk", FileCategory::Tooling),
            ("make/docker.mk", FileCategory::Tooling),
            ("scripts/setup.sh", FileCategory::Tooling),
            ("scripts/gen.mjs", FileCategory::Tooling),
            ("Makefile.template", FileCategory::Tooling),
            // docker config → add-only
            ("docker/compose/app.yml", FileCategory::Config),
            ("docker/config/env.app", FileCategory::Config),
            ("docker/system/traefik.yml", FileCategory::Config),
            ("docker/dockerfiles/Dockerfile.app", FileCategory::Config),
            // cloudflare → conditional
            ("make/cloudflare.mk", cf.clone()),
            ("scripts/cf-deploy.sh", cf.clone()),
            ("infra/cloudflare/wrangler.toml", cf.clone()),
            // skip
            ("recipes/astro/recipe.json", FileCategory::Skip),
            ("router/docker-compose.yml", FileCategory::Skip),
            ("project/nested.txt", FileCategory::Skip),
            ("README.md", FileCategory::Skip),
            ("make/notes.txt", FileCategory::Skip),
            ("scripts/notes.txt", FileCategory::Skip),
        ];

        for (rel, expected) in cases {
            assert_eq!(
                up.categorize_file(rel),
                *expected,
                "categorize_file({rel:?}) mismatch"
            );
        }
    }

    // ── Path remaps ──────────────────────────────────────────────────────

    #[test]
    fn upgrade_path_remaps_makefile_and_docker_config() {
        let dir = tempfile::tempdir().unwrap();
        let up = upgrader(dir.path(), dir.path());

        assert_eq!(
            up.template_to_project_path("Makefile.template"),
            PathBuf::from("Makefile")
        );
        assert_eq!(
            up.template_to_project_path("docker/config/env.app"),
            PathBuf::from("docker/.config/.env.app")
        );
        assert_eq!(
            up.template_to_project_path("docker/config/env.local"),
            PathBuf::from("docker/.config/.env.local")
        );
        // Everything else passes through untouched.
        assert_eq!(
            up.template_to_project_path("make/dev.mk"),
            PathBuf::from("make/dev.mk")
        );
    }

    // ── Backup naming ────────────────────────────────────────────────────

    #[test]
    fn upgrade_backup_path_naming() {
        let dir = tempfile::tempdir().unwrap();
        let up = upgrader(dir.path(), dir.path());

        assert_eq!(
            up.backup_path(Path::new("/p/scripts/setup.sh")),
            PathBuf::from("/p/scripts/setup.sh.bak")
        );
        assert_eq!(
            up.backup_path(Path::new("/p/make/dev.mk")),
            PathBuf::from("/p/make/dev.mk.bak")
        );
        assert_eq!(
            up.backup_path(Path::new("/p/docker/.config/.env.app")),
            PathBuf::from("/p/docker/.config/.env.app.bak")
        );
        // Extension-less files: the historical `with_extension` construction
        // yields a doubled dot. Pinned as-is — this characterizes today's
        // behavior; changing the name is a separate, deliberate change.
        assert_eq!(
            up.backup_path(Path::new("/p/Makefile")),
            PathBuf::from("/p/Makefile..bak")
        );
    }

    // ── discover_upgrades over synthetic trees ───────────────────────────

    #[test]
    fn upgrade_discovery_skips_recipes_router_and_unknown() {
        let t = tempfile::tempdir().unwrap();
        let p = tempfile::tempdir().unwrap();
        synthetic_templates(t.path());

        let entries = upgrader(t.path(), p.path()).discover_upgrades().unwrap();

        for skipped in [
            "recipes/astro/recipe.json",
            "router/docker-compose.yml",
            "project/nested.txt",
            "README.md",
        ] {
            assert!(
                entry_for(&entries, skipped).is_none(),
                "{skipped} must never reach the upgrade set"
            );
        }
        assert!(entries.iter().all(|e| e.category != FileCategory::Skip));
    }

    #[test]
    fn upgrade_discovery_omits_conditionals_when_feature_disabled() {
        let t = tempfile::tempdir().unwrap();
        let p = tempfile::tempdir().unwrap();
        synthetic_templates(t.path());
        let up = upgrader(t.path(), p.path());

        assert!(!up.is_feature_enabled("cloudflare"));
        let entries = up.discover_upgrades().unwrap();

        for cond in [
            "make/cloudflare.mk",
            "scripts/cf-deploy.sh",
            "infra/cloudflare/wrangler.toml",
        ] {
            assert!(
                entry_for(&entries, cond).is_none(),
                "{cond} is cloudflare-conditional and the feature is off"
            );
        }
        // Non-conditional tooling is still there.
        assert!(entry_for(&entries, "make/dev.mk").is_some());
    }

    #[test]
    fn upgrade_discovery_promotes_conditionals_when_feature_enabled() {
        let t = tempfile::tempdir().unwrap();
        let p = tempfile::tempdir().unwrap();
        synthetic_templates(t.path());
        std::fs::create_dir_all(p.path().join("infra/cloudflare")).unwrap();
        let up = upgrader(t.path(), p.path());

        assert!(up.is_feature_enabled("cloudflare"));
        let entries = up.discover_upgrades().unwrap();

        for cond in [
            "make/cloudflare.mk",
            "scripts/cf-deploy.sh",
            "infra/cloudflare/wrangler.toml",
        ] {
            let e = entry_for(&entries, cond)
                .unwrap_or_else(|| panic!("{cond} missing with cloudflare enabled"));
            assert_eq!(
                e.category,
                FileCategory::Tooling,
                "{cond} should be promoted to Tooling once its feature is on"
            );
        }
    }

    #[test]
    fn upgrade_tooling_add_update_current_actions() {
        let t = tempfile::tempdir().unwrap();
        let p = tempfile::tempdir().unwrap();
        synthetic_templates(t.path());
        // `make/dev.mk` present but stale → Update.
        write(p.path(), "make/dev.mk", "dev-mk-v1\n");
        // `scripts/setup.sh` byte-identical → Current.
        write(p.path(), "scripts/setup.sh", "setup-v2\n");
        // `Makefile` absent → Add.

        let entries = upgrader(t.path(), p.path()).discover_upgrades().unwrap();

        assert!(matches!(
            entry_for(&entries, "make/dev.mk").unwrap().action,
            UpgradeAction::Update
        ));
        assert!(matches!(
            entry_for(&entries, "scripts/setup.sh").unwrap().action,
            UpgradeAction::Current
        ));
        let mk = entry_for(&entries, "Makefile.template").unwrap();
        assert!(matches!(mk.action, UpgradeAction::Add));
        assert_eq!(mk.project_path, p.path().join("Makefile"));
    }

    #[test]
    fn upgrade_config_files_are_add_only_never_updated() {
        let t = tempfile::tempdir().unwrap();
        let p = tempfile::tempdir().unwrap();
        synthetic_templates(t.path());
        // Existing config with *different* content must still not be updated.
        write(p.path(), "docker/compose/app.yml", "hand-edited-by-user\n");

        let entries = upgrader(t.path(), p.path()).discover_upgrades().unwrap();

        let existing = entry_for(&entries, "docker/compose/app.yml").unwrap();
        assert_eq!(existing.category, FileCategory::Config);
        assert!(
            matches!(existing.action, UpgradeAction::Skip),
            "config that already exists must be left alone, got {:?}",
            existing.action
        );

        let missing = entry_for(&entries, "docker/system/traefik.yml").unwrap();
        assert!(
            matches!(missing.action, UpgradeAction::Add),
            "missing config should be added, got {:?}",
            missing.action
        );
    }

    #[test]
    fn upgrade_discovery_applies_docker_config_remap_to_project_path() {
        let t = tempfile::tempdir().unwrap();
        let p = tempfile::tempdir().unwrap();
        synthetic_templates(t.path());

        let entries = upgrader(t.path(), p.path()).discover_upgrades().unwrap();

        let env = entry_for(&entries, "docker/config/env.app").unwrap();
        assert_eq!(env.project_path, p.path().join("docker/.config/.env.app"));
    }

    #[test]
    fn upgrade_discovery_errors_when_project_templates_missing() {
        let t = tempfile::tempdir().unwrap();
        let p = tempfile::tempdir().unwrap();

        let err = upgrader(t.path(), p.path())
            .discover_upgrades()
            .unwrap_err();
        assert!(
            err.to_string().contains("Project templates not found"),
            "unexpected error: {err}"
        );
    }

    // ── Known-broken lane (bd:mech-crate-z5i) ────────────────────────────

    /// `discover_upgrades()` reads `<templates>/project/`, but the shipped
    /// `templates/` tree has no `project/` subdirectory — scaffold files live
    /// at the top level (`make/`, `scripts/`, `docker/`, `Makefile.template`).
    /// So `mx upgrade` errors out for every real installation.
    ///
    /// This test asserts the FIXED behavior against the repo's real layout and
    /// is expected to be RED until bd:mech-crate-z5i lands.
    #[test]
    #[ignore = "bd:mech-crate-z5i fix mx upgrade: reads non-existent templates/project/"]
    fn upgrade_discovery_works_against_real_templates_layout() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("crates/mx-lib should sit two levels below the repo root")
            .to_path_buf();
        let templates = repo_root.join("templates");
        assert!(
            templates.is_dir(),
            "setup: real templates dir missing at {}",
            templates.display()
        );

        let project = tempfile::tempdir().unwrap();
        crate::test_support::scaffold_project(project.path());

        let result = upgrader(&templates, project.path()).discover_upgrades();

        assert!(
            result.is_ok(),
            "discover_upgrades() must succeed against the shipped templates/ layout, got: {:?}",
            result.as_ref().err()
        );
        assert!(
            !result.unwrap().is_empty(),
            "discover_upgrades() must find scaffold files in the shipped templates/ layout"
        );
    }
}
