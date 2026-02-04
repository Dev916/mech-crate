//! Project detection and analysis
//!
//! Detects MechCrate projects and provides information about their structure.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Information about a MechCrate project
#[derive(Debug, Clone)]
pub struct Project {
    /// Project root directory
    pub root: PathBuf,
    /// Project name (directory name)
    pub name: String,
    /// Discovered services
    pub services: Vec<String>,
    /// Available compose files
    pub compose_files: Vec<PathBuf>,
    /// Available make targets
    pub make_targets: Vec<String>,
    /// Whether this is a full MechCrate project (has make/ and scripts/)
    pub is_full_project: bool,
}

/// Project detector for finding and analyzing MechCrate projects
#[derive(Debug, Default)]
pub struct ProjectDetector {
    /// Whether to require full project structure (make/ + scripts/)
    strict: bool,
}

impl ProjectDetector {
    /// Create a new project detector (lenient mode - just needs Makefile + docker/)
    pub fn new() -> Self {
        Self { strict: false }
    }

    /// Create a strict detector (requires full structure: Makefile + docker/ + make/ + scripts/)
    pub fn strict() -> Self {
        Self { strict: true }
    }

    /// Check if a directory is a MechCrate project (minimal: Makefile + docker/)
    pub fn is_project(&self, dir: &Path) -> bool {
        let has_makefile = dir.join("Makefile").exists();
        let has_docker = dir.join("docker").is_dir();
        let has_make = dir.join("make").is_dir();
        let has_scripts = dir.join("scripts").is_dir();

        if self.strict {
            // Full MechCrate project
            has_makefile && has_docker && has_make && has_scripts
        } else {
            // Minimal project (can use dev commands)
            has_makefile && has_docker
        }
    }

    /// Check if a directory is a full MechCrate project
    pub fn is_full_project(&self, dir: &Path) -> bool {
        dir.join("Makefile").exists()
            && dir.join("docker").is_dir()
            && dir.join("make").is_dir()
            && dir.join("scripts").is_dir()
    }

    /// Find the project root by walking up from the current directory
    pub fn find_root(&self, start: &Path) -> Result<PathBuf> {
        let mut current = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());

        loop {
            if self.is_project(&current) {
                return Ok(current);
            }

            match current.parent() {
                Some(parent) if parent != current => current = parent.to_path_buf(),
                _ => {
                    return Err(Error::NotAProject);
                }
            }
        }
    }

    /// Find the project root from the current working directory
    pub fn find_root_from_cwd(&self) -> Result<PathBuf> {
        let cwd = std::env::current_dir().map_err(|e| {
            Error::Config(format!("Failed to get current directory: {}", e))
        })?;
        
        tracing::debug!("Looking for project root from: {}", cwd.display());
        
        match self.find_root(&cwd) {
            Ok(root) => {
                tracing::debug!("Found project root: {}", root.display());
                Ok(root)
            }
            Err(e) => {
                tracing::debug!("No project found from {}: {}", cwd.display(), e);
                Err(e)
            }
        }
    }

    /// Analyze a project and return detailed information
    pub fn analyze(&self, root: &Path) -> Result<Project> {
        if !self.is_project(root) {
            return Err(Error::NotAProject);
        }

        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let services = self.discover_services(root)?;
        let compose_files = self.discover_compose_files(root)?;
        let make_targets = self.discover_make_targets(root)?;
        let is_full = self.is_full_project(root);

        Ok(Project {
            root: root.to_path_buf(),
            name,
            services,
            compose_files,
            make_targets,
            is_full_project: is_full,
        })
    }

    /// Discover services from docker/compose/ directory
    fn discover_services(&self, root: &Path) -> Result<Vec<String>> {
        let compose_dir = root.join("docker").join("compose");
        let mut services = Vec::new();

        if compose_dir.exists() {
            for entry in std::fs::read_dir(&compose_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                        // Skip dev/prod variants, get base service name
                        let base_name = name
                            .trim_end_matches(".dev")
                            .trim_end_matches(".prod")
                            .to_string();

                        if !services.contains(&base_name) && !base_name.is_empty() {
                            services.push(base_name);
                        }
                    }
                }
            }
        }

        services.sort();
        Ok(services)
    }

    /// Discover compose files
    fn discover_compose_files(&self, root: &Path) -> Result<Vec<PathBuf>> {
        let compose_dir = root.join("docker").join("compose");
        let mut files = Vec::new();

        if compose_dir.exists() {
            for entry in std::fs::read_dir(&compose_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "yml" || ext == "yaml" {
                            files.push(path);
                        }
                    }
                }
            }
        }

        files.sort();
        Ok(files)
    }

    /// Discover make targets from Makefile and make/*.mk
    fn discover_make_targets(&self, root: &Path) -> Result<Vec<String>> {
        let mut targets = Vec::new();

        // Parse main Makefile
        let makefile = root.join("Makefile");
        if makefile.exists() {
            targets.extend(self.parse_makefile_targets(&makefile)?);
        }

        // Parse make/*.mk files
        let make_dir = root.join("make");
        if make_dir.exists() {
            for entry in std::fs::read_dir(&make_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().map(|e| e == "mk").unwrap_or(false) {
                    targets.extend(self.parse_makefile_targets(&path)?);
                }
            }
        }

        targets.sort();
        targets.dedup();
        Ok(targets)
    }

    /// Parse targets from a Makefile
    fn parse_makefile_targets(&self, path: &Path) -> Result<Vec<String>> {
        let content = std::fs::read_to_string(path)?;
        let mut targets = Vec::new();

        for line in content.lines() {
            // Match lines like "target:" or "target: deps"
            if let Some(colon_pos) = line.find(':') {
                let before_colon = &line[..colon_pos];
                // Skip variables and special targets
                if !before_colon.contains('=')
                    && !before_colon.starts_with('.')
                    && !before_colon.starts_with('\t')
                    && !before_colon.starts_with(' ')
                    && !before_colon.is_empty()
                {
                    let target = before_colon.trim().to_string();
                    if !target.is_empty() && !target.contains('%') {
                        targets.push(target);
                    }
                }
            }
        }

        Ok(targets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_minimal_project(dir: &Path) {
        fs::create_dir_all(dir.join("docker/compose")).unwrap();
        fs::write(dir.join("Makefile"), "dev:\n\techo dev").unwrap();
    }

    fn create_full_project(dir: &Path) {
        create_minimal_project(dir);
        fs::create_dir_all(dir.join("make")).unwrap();
        fs::create_dir_all(dir.join("scripts")).unwrap();
    }

    #[test]
    fn test_not_a_project() {
        let detector = ProjectDetector::new();
        assert!(!detector.is_project(Path::new("/tmp")));
    }

    #[test]
    fn test_minimal_project_detection() {
        let temp = TempDir::new().unwrap();
        create_minimal_project(temp.path());

        let detector = ProjectDetector::new();
        assert!(detector.is_project(temp.path()));
        assert!(!detector.is_full_project(temp.path()));
    }

    #[test]
    fn test_full_project_detection() {
        let temp = TempDir::new().unwrap();
        create_full_project(temp.path());

        let detector = ProjectDetector::new();
        assert!(detector.is_project(temp.path()));
        assert!(detector.is_full_project(temp.path()));
    }

    #[test]
    fn test_strict_detector() {
        let temp = TempDir::new().unwrap();
        create_minimal_project(temp.path());

        let lenient = ProjectDetector::new();
        let strict = ProjectDetector::strict();

        assert!(lenient.is_project(temp.path()));
        assert!(!strict.is_project(temp.path())); // Missing make/ and scripts/
    }

    #[test]
    fn test_find_root_from_subdirectory() {
        let temp = TempDir::new().unwrap();
        create_minimal_project(temp.path());

        // Create a nested subdirectory
        let subdir = temp.path().join("src/lib/utils");
        fs::create_dir_all(&subdir).unwrap();

        let detector = ProjectDetector::new();
        let root = detector.find_root(&subdir).unwrap();
        assert_eq!(root, temp.path().canonicalize().unwrap());
    }

    #[test]
    fn test_service_discovery() {
        let temp = TempDir::new().unwrap();
        create_minimal_project(temp.path());

        // Add some compose files
        fs::write(temp.path().join("docker/compose/api.yml"), "").unwrap();
        fs::write(temp.path().join("docker/compose/api.dev.yml"), "").unwrap();
        fs::write(temp.path().join("docker/compose/web.yml"), "").unwrap();

        let detector = ProjectDetector::new();
        let project = detector.analyze(temp.path()).unwrap();

        assert_eq!(project.services.len(), 2);
        assert!(project.services.contains(&"api".to_string()));
        assert!(project.services.contains(&"web".to_string()));
    }
}
