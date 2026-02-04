//! Project Detection and Analysis
//!
//! Detects MechCrate projects and extracts information about services,
//! compose files, and available operations.

use std::path::{Path, PathBuf};
use tokio::fs;

use crate::error::{McpError, McpResult};

/// Information about a MechCrate project
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub root: PathBuf,
    pub name: String,
    pub services: Vec<ServiceInfo>,
    pub has_infra: bool,
    pub infra_providers: Vec<String>,
    pub compose_files: Vec<String>,
    pub make_targets: Vec<String>,
}

/// Information about a service in the project
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub has_dockerfile: bool,
    pub has_compose: bool,
    pub has_dev_compose: bool,
    pub app_dir: Option<PathBuf>,
}

/// Detects and analyzes MechCrate projects
pub struct ProjectDetector;

impl ProjectDetector {
    pub fn new() -> Self {
        Self
    }

    /// Check if a directory is a MechCrate project
    pub fn is_mech_crate_project(path: &Path) -> bool {
        path.join("Makefile").exists()
            && path.join("docker").exists()
            && path.join("make").exists()
            && path.join("scripts").exists()
    }

    /// Detect MechCrate project from a path (walks up to find root)
    pub fn find_project_root(start: &Path) -> Option<PathBuf> {
        let mut current = start.to_path_buf();
        
        loop {
            if Self::is_mech_crate_project(&current) {
                return Some(current);
            }
            
            if !current.pop() {
                return None;
            }
        }
    }

    /// Analyze a MechCrate project
    pub async fn analyze(&self, root: &Path) -> McpResult<ProjectInfo> {
        if !Self::is_mech_crate_project(root) {
            return Err(McpError::NotInProject);
        }

        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let services = self.discover_services(root).await?;
        let (has_infra, infra_providers) = self.discover_infra(root).await;
        let compose_files = self.discover_compose_files(root).await?;
        let make_targets = self.discover_make_targets(root).await?;

        Ok(ProjectInfo {
            root: root.to_path_buf(),
            name,
            services,
            has_infra,
            infra_providers,
            compose_files,
            make_targets,
        })
    }

    /// Discover services in the project
    async fn discover_services(&self, root: &Path) -> McpResult<Vec<ServiceInfo>> {
        let mut services = Vec::new();
        
        // Check apps directory
        let apps_dir = root.join("apps");
        if apps_dir.exists() {
            if let Ok(mut entries) = fs::read_dir(&apps_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false) {
                        let name = entry.file_name().to_string_lossy().to_string();
                        
                        let dockerfile_path = root.join(format!("docker/dockerfiles/{}/app", name));
                        let compose_path = root.join(format!("docker/compose/{}.yml", name));
                        let dev_compose_path = root.join(format!("docker/compose/{}.dev.yml", name));
                        
                        services.push(ServiceInfo {
                            name: name.clone(),
                            has_dockerfile: dockerfile_path.exists(),
                            has_compose: compose_path.exists(),
                            has_dev_compose: dev_compose_path.exists(),
                            app_dir: Some(entry.path()),
                        });
                    }
                }
            }
        }
        
        // Also check compose files for services without app directories
        let compose_dir = root.join("docker/compose");
        if compose_dir.exists() {
            if let Ok(mut entries) = fs::read_dir(&compose_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    
                    // Skip dev overrides and shared services
                    if file_name.ends_with(".dev.yml") || file_name.starts_with("db.") || file_name.starts_with("redis.") {
                        continue;
                    }
                    
                    if file_name.ends_with(".yml") {
                        let service_name = file_name.trim_end_matches(".yml").to_string();
                        
                        // Skip if already found via apps
                        if !services.iter().any(|s| s.name == service_name) {
                            services.push(ServiceInfo {
                                name: service_name.clone(),
                                has_dockerfile: root.join(format!("docker/dockerfiles/{}/app", service_name)).exists(),
                                has_compose: true,
                                has_dev_compose: root.join(format!("docker/compose/{}.dev.yml", service_name)).exists(),
                                app_dir: None,
                            });
                        }
                    }
                }
            }
        }
        
        Ok(services)
    }

    /// Discover infrastructure configuration
    async fn discover_infra(&self, root: &Path) -> (bool, Vec<String>) {
        let infra_dir = root.join("infra");
        let mut providers = Vec::new();
        
        if !infra_dir.exists() {
            return (false, providers);
        }
        
        let known_providers = ["cloudflare", "digitalocean", "aws", "hetzner"];
        
        for provider in known_providers {
            if infra_dir.join(provider).exists() {
                providers.push(provider.to_string());
            }
        }
        
        (!providers.is_empty(), providers)
    }

    /// Discover compose files
    async fn discover_compose_files(&self, root: &Path) -> McpResult<Vec<String>> {
        let mut files = Vec::new();
        let compose_dir = root.join("docker/compose");
        
        if compose_dir.exists() {
            if let Ok(mut entries) = fs::read_dir(&compose_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.ends_with(".yml") {
                        files.push(name);
                    }
                }
            }
        }
        
        files.sort();
        Ok(files)
    }

    /// Discover make targets from make/*.mk files
    async fn discover_make_targets(&self, root: &Path) -> McpResult<Vec<String>> {
        let mut targets = Vec::new();
        let make_dir = root.join("make");
        
        // Standard targets from Makefile.template
        targets.extend(vec![
            "help".to_string(),
            "init".to_string(),
            "test".to_string(),
            "ps".to_string(),
            "doctor".to_string(),
            "make-key".to_string(),
        ]);
        
        if make_dir.exists() {
            if let Ok(mut entries) = fs::read_dir(&make_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.ends_with(".mk") {
                        // Extract targets from common .mk files
                        let base = name.trim_end_matches(".mk");
                        match base {
                            "dev" => targets.push("dev".to_string()),
                            "up" => targets.push("up".to_string()),
                            "down" => targets.push("down".to_string()),
                            "logs" => targets.push("logs".to_string()),
                            "sh" => {
                                targets.push("sh".to_string());
                                targets.push("bash".to_string());
                            }
                            "restart" => targets.push("restart".to_string()),
                            "build" => targets.push("build".to_string()),
                            "start" => targets.push("start".to_string()),
                            "stop" => targets.push("stop".to_string()),
                            "run" => targets.push("run".to_string()),
                            "cloudflare" => {
                                targets.extend(vec![
                                    "cf-setup".to_string(),
                                    "cf-init".to_string(),
                                    "cf-status".to_string(),
                                    "cf-deploy".to_string(),
                                    "cf-deploy-all".to_string(),
                                ]);
                            }
                            "release" => {
                                targets.push("release".to_string());
                                targets.push("release-patch".to_string());
                                targets.push("release-minor".to_string());
                                targets.push("release-major".to_string());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        targets.sort();
        targets.dedup();
        Ok(targets)
    }

    /// Get service information
    pub async fn get_service(&self, root: &Path, service_name: &str) -> McpResult<ServiceInfo> {
        let project = self.analyze(root).await?;
        
        project
            .services
            .into_iter()
            .find(|s| s.name == service_name)
            .ok_or_else(|| McpError::ServiceNotFound(service_name.to_string()))
    }

    /// List all projects in a directory
    pub async fn find_all_projects(&self, search_dir: &Path) -> McpResult<Vec<PathBuf>> {
        let mut projects = Vec::new();
        
        if Self::is_mech_crate_project(search_dir) {
            projects.push(search_dir.to_path_buf());
        }
        
        if let Ok(mut entries) = fs::read_dir(search_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false) {
                    let path = entry.path();
                    if Self::is_mech_crate_project(&path) {
                        projects.push(path);
                    }
                }
            }
        }
        
        Ok(projects)
    }
}

impl Default for ProjectDetector {
    fn default() -> Self {
        Self::new()
    }
}
