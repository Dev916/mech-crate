//! Infrastructure configuration management

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::MechCrateConfig;
use crate::error::{Error, Result};

/// Supported infrastructure providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InfraProvider {
    Cloudflare,
    DigitalOcean,
    Aws,
    Hetzner,
}

impl InfraProvider {
    /// Get all available providers
    pub fn all() -> &'static [InfraProvider] {
        &[
            InfraProvider::Cloudflare,
            InfraProvider::DigitalOcean,
            InfraProvider::Aws,
            InfraProvider::Hetzner,
        ]
    }

    /// Get the provider name as a string
    pub fn name(&self) -> &'static str {
        match self {
            InfraProvider::Cloudflare => "cloudflare",
            InfraProvider::DigitalOcean => "digitalocean",
            InfraProvider::Aws => "aws",
            InfraProvider::Hetzner => "hetzner",
        }
    }

    /// Parse a provider from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cloudflare" | "cf" => Some(InfraProvider::Cloudflare),
            "digitalocean" | "do" => Some(InfraProvider::DigitalOcean),
            "aws" | "amazon" => Some(InfraProvider::Aws),
            "hetzner" | "hz" => Some(InfraProvider::Hetzner),
            _ => None,
        }
    }

    /// Get the config file name for this provider
    pub fn config_filename(&self) -> &'static str {
        match self {
            InfraProvider::Cloudflare => "cloudflare.env",
            InfraProvider::DigitalOcean => "digitalocean.env",
            InfraProvider::Aws => "aws.env",
            InfraProvider::Hetzner => "hetzner.env",
        }
    }
}

/// Infrastructure configuration manager
#[derive(Debug)]
pub struct InfraConfig {
    global_config: MechCrateConfig,
    project_root: Option<PathBuf>,
}

impl InfraConfig {
    /// Create a new infrastructure config manager
    pub fn new(global_config: MechCrateConfig) -> Self {
        Self {
            global_config,
            project_root: None,
        }
    }

    /// Set the project root for project-level config resolution
    pub fn with_project(mut self, project_root: impl AsRef<Path>) -> Self {
        self.project_root = Some(project_root.as_ref().to_path_buf());
        self
    }

    /// Get the global config path for a provider
    pub fn global_config_path(&self, provider: InfraProvider) -> PathBuf {
        self.global_config
            .infra_dir()
            .join(provider.config_filename())
    }

    /// Get the project config path for a provider
    pub fn project_config_path(&self, provider: InfraProvider) -> Option<PathBuf> {
        self.project_root.as_ref().map(|root| {
            root.join("infra")
                .join(provider.name())
                .join(format!(".env.{}", provider.name()))
        })
    }

    /// Check if a provider is configured globally
    pub fn is_globally_configured(&self, provider: InfraProvider) -> bool {
        self.global_config_path(provider).exists()
    }

    /// Check if a provider is configured for the project
    pub fn is_project_configured(&self, provider: InfraProvider) -> bool {
        self.project_config_path(provider)
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Resolve the config path for a provider (project -> linked -> global)
    pub fn resolve_config_path(&self, provider: InfraProvider) -> Option<PathBuf> {
        // 1. Check project-level config
        if let Some(project_path) = self.project_config_path(provider) {
            if project_path.exists() {
                // Check if it's linked to global
                let marker = project_path.with_extension("linked");
                if marker.exists() {
                    // Use global config
                    let global_path = self.global_config_path(provider);
                    if global_path.exists() {
                        return Some(global_path);
                    }
                } else {
                    return Some(project_path);
                }
            }
        }

        // 2. Fall back to global config
        let global_path = self.global_config_path(provider);
        if global_path.exists() {
            return Some(global_path);
        }

        None
    }

    /// Load configuration for a provider as key-value pairs
    pub fn load_config(&self, provider: InfraProvider) -> Result<HashMap<String, String>> {
        let config_path = self
            .resolve_config_path(provider)
            .ok_or_else(|| Error::Config(format!("{} not configured", provider.name())))?;

        let content = std::fs::read_to_string(&config_path)?;
        let mut config = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
                config.insert(key, value);
            }
        }

        Ok(config)
    }

    /// Get a specific configuration value
    pub fn get_value(&self, provider: InfraProvider, key: &str) -> Result<Option<String>> {
        let config = self.load_config(provider)?;
        Ok(config.get(key).cloned())
    }

    /// List all configured providers
    pub fn list_configured(&self) -> Vec<(InfraProvider, bool, bool)> {
        InfraProvider::all()
            .iter()
            .map(|&p| (p, self.is_globally_configured(p), self.is_project_configured(p)))
            .collect()
    }
}
