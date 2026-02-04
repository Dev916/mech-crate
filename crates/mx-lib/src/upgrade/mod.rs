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
            path if path.starts_with("scripts/") && (path.ends_with(".sh") || path.ends_with(".mjs")) => {
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

            let project_path = self.project_dir.join(self.template_to_project_path(rel_path));
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
