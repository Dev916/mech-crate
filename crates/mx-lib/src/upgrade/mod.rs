//! Project upgrade functionality
//!
//! Handles upgrading MechCrate projects with latest scaffolding.

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::error::{Error, Result};
use crate::paths;

/// Template subtrees the upgrader owns, relative to `templates/`.
///
/// These mirror what `mx new` lays down (see
/// `crates/mx-cli/src/commands/new.rs::copy_templates`): the shipped layout has
/// no `templates/project/` wrapper — scaffold files sit at the top level.
///
/// Everything outside this scope belongs to someone else and must never be
/// force-fed into an existing project:
///
/// - `recipes/`, `router/` — not project scaffolding at all.
/// - `docker/compose/`, `docker/system/`, `docker/dockerfiles/` — owned by the
///   recipe that installs a service. `scripts/.bashrc` builds a service-less
///   `make dev` context by globbing `docker/compose/*.yml`, so seeding the
///   reference stack here would silently boot postgres, redis, nginx and
///   traefik for a project that never asked for them.
/// - `docker/config/env.<service>` — likewise recipe-owned; `mx new` copies
///   only the two shared files below.
/// - `infra/` — written by `mx infra setup`, which expands `{{PROJECT_NAME}}`
///   placeholders; copying the raw templates back would undo that expansion.
/// - `scripts/<subdir>/**` — `mx new` copies only the top-level files of
///   `templates/scripts/`, so nested helper bundles (e.g. `scripts/md2pdf/`)
///   are not part of the scaffold. They are walked here but categorized
///   [`FileCategory::Skip`], so discovery never offers to add them.
const SCAFFOLD_DIRS: &[&str] = &["make", "scripts"];

/// Individual template files the upgrader owns, relative to `templates/`.
const SCAFFOLD_FILES: &[&str] = &[
    "Makefile.template",
    "docker/config/env.shared",
    "docker/config/env.secrets.template",
];

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
            // `scripts/` — owned file-for-file with what `mx new` lays down.
            // `copy_templates` copies every TOP-LEVEL file of
            // `templates/scripts/`, extension or not, so the old `.sh`/`.mjs`
            // extension test silently skipped `scripts/.bashrc` — the helper
            // library every other script sources. `mx upgrade` would refresh
            // `dev.sh`/`up.sh` and never the library they depend on.
            //
            // Nested subtrees (e.g. `scripts/md2pdf/`) are NOT copied by
            // `mx new`, so upgrade must not add them either — that would break
            // the scope invariant pinned by
            // `upgrade_discovery_scope_mirrors_mx_new`.
            path if path.starts_with("scripts/") => {
                let rel = &path["scripts/".len()..];
                if rel.is_empty() || rel.contains('/') {
                    FileCategory::Skip
                } else if rel.starts_with("cf-") {
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

    /// Template-relative paths inside the upgrader's scope, in stable order.
    ///
    /// Reads the SHIPPED `templates/` layout — `Makefile.template`, `make/`,
    /// `scripts/` and the two shared `docker/config/` files — rather than a
    /// `templates/project/` subtree, which has never existed in the published
    /// tree. See [`SCAFFOLD_DIRS`] for what is deliberately left out.
    pub fn scaffold_template_files(&self) -> Result<Vec<String>> {
        let mut rels = Vec::new();

        for file in SCAFFOLD_FILES {
            if self.templates_dir.join(file).is_file() {
                rels.push((*file).to_string());
            }
        }

        for dir in SCAFFOLD_DIRS {
            let root = self.templates_dir.join(dir);
            if !root.is_dir() {
                continue;
            }

            for entry in WalkDir::new(&root)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let rel = entry
                    .path()
                    .strip_prefix(&self.templates_dir)
                    .map_err(|e| Error::Other(e.to_string()))?
                    .to_str()
                    .ok_or_else(|| Error::Other("Invalid path".into()))?
                    .replace(std::path::MAIN_SEPARATOR, "/");
                rels.push(rel);
            }
        }

        rels.sort();
        Ok(rels)
    }

    /// Discover all upgrade entries
    pub fn discover_upgrades(&self) -> Result<Vec<UpgradeEntry>> {
        let mut entries = Vec::new();

        if !self.templates_dir.is_dir() {
            return Err(Error::Config(format!(
                "Project templates not found at {}",
                self.templates_dir.display()
            )));
        }

        for rel_path in self.scaffold_template_files()? {
            let rel_path = rel_path.as_str();
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
            let template_path = self.templates_dir.join(rel_path);

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

    /// A synthetic `templates/` tree in the SHIPPED layout — scaffold files at
    /// the top level, no `project/` wrapper — covering every categorization arm
    /// plus the subtrees discovery must leave alone.
    fn synthetic_templates(root: &Path) {
        let p = root;
        // Tooling
        write(p, "make/dev.mk", "dev-mk-v2\n");
        write(p, "scripts/setup.sh", "setup-v2\n");
        write(p, "scripts/.bashrc", "bashrc-v2\n");
        write(p, "Makefile.template", "makefile-v2\n");
        // Conditional (cloudflare)
        write(p, "make/cloudflare.mk", "cf-mk-v2\n");
        write(p, "scripts/cf-deploy.sh", "cf-deploy-v2\n");
        // Config (add-only): the two shared env files `mx new` copies
        write(p, "docker/config/env.shared", "env-shared-v2\n");
        write(p, "docker/config/env.secrets.template", "env-secrets-v2\n");
        // Out of scope — recipe-owned service pieces and infra templates
        write(p, "docker/compose/app.yml", "compose-v2\n");
        write(p, "docker/config/env.app", "env-app-v2\n");
        write(p, "docker/system/traefik.yml", "system-v2\n");
        write(p, "docker/dockerfiles/Dockerfile.app", "dockerfile-v2\n");
        write(p, "infra/cloudflare/wrangler.toml", "cf-toml-v2\n");
        // Skip
        write(p, "recipes/astro/recipe.json", "{}\n");
        write(p, "router/docker-compose.yml", "router-v2\n");
        write(p, "project/nested.txt", "nested\n");
        write(p, "README.md", "readme\n");
        // Nested helper bundle under scripts/ — walked, never adopted.
        write(p, "scripts/md2pdf/md2pdf.ts", "md2pdf-v2\n");
        write(p, "scripts/md2pdf/package.json", "{}\n");
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
            // bd:mech-crate-12p — `mx new` copies every top-level file of
            // `templates/scripts/`, extension or not. The extension-based arm
            // used to drop `.bashrc` (and any other extension-less helper) into
            // Skip, so `mx upgrade` refreshed dev.sh/up.sh but never the helper
            // library they source — the COMPOSE_PROJECT_NAME pin could never
            // reach an existing project.
            ("scripts/.bashrc", FileCategory::Tooling),
            // `scripts/notes.txt` follows from the same rule: `mx new` ships it,
            // so upgrade owns it. (It used to be Skip, which is precisely the
            // bug above.)
            ("scripts/notes.txt", FileCategory::Tooling),
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
            // Nested bundles under scripts/ are not copied by `mx new`, so
            // upgrade must never add them.
            ("scripts/md2pdf/md2pdf.ts", FileCategory::Skip),
            ("scripts/md2pdf/package.json", FileCategory::Skip),
            ("scripts/md2pdf/README.md", FileCategory::Skip),
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
            "scripts/md2pdf/md2pdf.ts",
            "scripts/md2pdf/package.json",
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

        for cond in ["make/cloudflare.mk", "scripts/cf-deploy.sh"] {
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

        // `infra/cloudflare/**` is deliberately out of discovery scope even with
        // the feature on — see `upgrade_discovery_scope_mirrors_mx_new`.
        for cond in ["make/cloudflare.mk", "scripts/cf-deploy.sh"] {
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
        write(
            p.path(),
            "docker/.config/.env.shared",
            "hand-edited-by-user\n",
        );

        let entries = upgrader(t.path(), p.path()).discover_upgrades().unwrap();

        let existing = entry_for(&entries, "docker/config/env.shared").unwrap();
        assert_eq!(existing.category, FileCategory::Config);
        assert!(
            matches!(existing.action, UpgradeAction::Skip),
            "config that already exists must be left alone, got {:?}",
            existing.action
        );

        let missing = entry_for(&entries, "docker/config/env.secrets.template").unwrap();
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

        let env = entry_for(&entries, "docker/config/env.shared").unwrap();
        assert_eq!(
            env.project_path,
            p.path().join("docker/.config/.env.shared")
        );
    }

    /// Discovery owns exactly what `mx new` lays down. Recipe-owned service
    /// pieces and `mx infra setup`-owned templates must never be force-fed into
    /// an existing project: `scripts/.bashrc` builds a service-less `make dev`
    /// context by globbing `docker/compose/*.yml`, so seeding the reference
    /// stack would boot postgres/redis/nginx/traefik unasked, and the `infra/`
    /// templates still carry `{{PROJECT_NAME}}` placeholders that
    /// `mx infra setup` expands.
    #[test]
    fn upgrade_discovery_scope_mirrors_mx_new() {
        let t = tempfile::tempdir().unwrap();
        let p = tempfile::tempdir().unwrap();
        synthetic_templates(t.path());
        // Cloudflare on: even then, infra templates stay out of scope.
        std::fs::create_dir_all(p.path().join("infra/cloudflare")).unwrap();

        let entries = upgrader(t.path(), p.path()).discover_upgrades().unwrap();

        for out_of_scope in [
            "docker/compose/app.yml",
            "docker/system/traefik.yml",
            "docker/dockerfiles/Dockerfile.app",
            "docker/config/env.app",
            "infra/cloudflare/wrangler.toml",
        ] {
            assert!(
                entry_for(&entries, out_of_scope).is_none(),
                "{out_of_scope} is recipe/infra-owned and must stay out of the upgrade set"
            );
        }

        // The skeleton `mx new` copies is in scope — including the
        // extension-less helper library (bd:mech-crate-12p).
        for in_scope in [
            "Makefile.template",
            "make/dev.mk",
            "scripts/setup.sh",
            "scripts/.bashrc",
            "docker/config/env.shared",
            "docker/config/env.secrets.template",
        ] {
            assert!(
                entry_for(&entries, in_scope).is_some(),
                "{in_scope} is part of the project skeleton and must be discovered"
            );
        }
    }

    #[test]
    fn upgrade_discovery_errors_when_project_templates_missing() {
        let t = tempfile::tempdir().unwrap();
        let p = tempfile::tempdir().unwrap();
        let missing = t.path().join("no-such-templates");

        let err = upgrader(&missing, p.path())
            .discover_upgrades()
            .unwrap_err();
        assert!(
            err.to_string().contains("Project templates not found"),
            "unexpected error: {err}"
        );
    }

    // ── Real shipped layout (was bd:mech-crate-z5i) ──────────────────────

    /// Regression net for bd:mech-crate-z5i: `discover_upgrades()` used to read
    /// `<templates>/project/`, but the shipped `templates/` tree has no
    /// `project/` subdirectory — scaffold files live at the top level
    /// (`make/`, `scripts/`, `docker/`, `Makefile.template`), so `mx upgrade`
    /// errored out for every real installation.
    #[test]
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
        let entries = result.unwrap();
        assert!(
            !entries.is_empty(),
            "discover_upgrades() must find scaffold files in the shipped templates/ layout"
        );

        // The real skeleton: Makefile.template → Makefile, make modules, scripts.
        let mk = entry_for(&entries, "Makefile.template")
            .expect("the shipped Makefile.template must be discovered");
        assert_eq!(mk.project_path, project.path().join("Makefile"));
        assert!(
            entry_for(&entries, "make/dev.mk").is_some(),
            "the shipped make/ modules must be discovered"
        );
        assert!(
            entries
                .iter()
                .any(|e| e.template_path.to_string_lossy().contains("/scripts/")),
            "the shipped scripts/ must be discovered"
        );

        // And nothing recipe- or infra-owned rides along. `scripts/md2pdf/` is
        // a nested helper bundle `mx new` never copies — upgrade must not add
        // it either (bd:mech-crate-12p).
        for forbidden in [
            "/recipes/",
            "/router/",
            "/docker/compose/",
            "/infra/",
            "/scripts/md2pdf/",
        ] {
            assert!(
                !entries
                    .iter()
                    .any(|e| e.template_path.to_string_lossy().contains(forbidden)),
                "{forbidden} must stay out of the upgrade set"
            );
        }
    }

    /// bd:mech-crate-12p — `mx upgrade` must deliver the SHIPPED
    /// `templates/scripts/.bashrc`. It is the helper library every other script
    /// sources (`compose_context_files`, `run_service_in_context`, and the
    /// `COMPOSE_PROJECT_NAME` pin of bd:mech-crate-71u), but the old
    /// extension-based categorization arm classified it `Skip`, so upgrade
    /// refreshed `dev.sh`/`up.sh` and left the library they depend on frozen at
    /// whatever version the project was scaffolded with.
    #[test]
    fn upgrade_discovers_the_shipped_extensionless_script_helpers() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("crates/mx-lib should sit two levels below the repo root")
            .to_path_buf();
        let templates = repo_root.join("templates");
        assert!(
            templates.join("scripts/.bashrc").is_file(),
            "setup: templates/scripts/.bashrc must exist"
        );

        let project = tempfile::tempdir().unwrap();
        crate::test_support::scaffold_project(project.path());
        // A project scaffolded with a stale helper library: upgrade must offer
        // to update it, not skip it.
        std::fs::write(project.path().join("scripts/.bashrc"), "stale-helpers\n").unwrap();

        let entries = upgrader(&templates, project.path())
            .discover_upgrades()
            .unwrap();

        let bashrc = entry_for(&entries, "scripts/.bashrc")
            .expect("the shipped scripts/.bashrc must be discovered");
        assert_eq!(bashrc.category, FileCategory::Tooling);
        assert_eq!(
            bashrc.project_path,
            project.path().join("scripts/.bashrc"),
            "dotfiles map straight through, no path remap"
        );
        assert!(
            matches!(bashrc.action, UpgradeAction::Update),
            "a stale scripts/.bashrc must be offered as an Update, got {:?}",
            bashrc.action
        );

        // Every other extension-less top-level file in templates/scripts/ rides
        // the same rule — the test must not hard-code only `.bashrc`.
        for entry in std::fs::read_dir(templates.join("scripts"))
            .unwrap()
            .flatten()
        {
            let path = entry.path();
            if !path.is_file() || path.extension().is_some() {
                continue;
            }
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let rel = format!("scripts/{name}");
            assert_eq!(
                upgrader(&templates, project.path()).categorize_file(&rel),
                FileCategory::Tooling,
                "{rel} is copied by `mx new` and must be upgrade-owned"
            );
        }
    }
}
