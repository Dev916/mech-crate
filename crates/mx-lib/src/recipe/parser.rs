//! Recipe JSON parser
//!
//! Parses recipe.json files into strongly-typed structures.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// A MechCrate recipe definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Recipe identifier
    pub name: String,

    /// Human-readable title
    #[serde(default)]
    pub title: Option<String>,

    /// Description of what the recipe provides
    #[serde(default)]
    pub description: Option<String>,

    /// Recipe version
    #[serde(default)]
    pub version: Option<String>,

    /// Feature list for documentation
    #[serde(default)]
    pub features: Vec<String>,

    /// Services created by this recipe
    #[serde(default)]
    pub services: Vec<RecipeService>,

    /// Configuration options
    #[serde(default)]
    pub options: HashMap<String, RecipeOption>,

    /// Placeholder definitions for template interpolation
    #[serde(default)]
    pub placeholders: HashMap<String, PlaceholderDef>,

    /// App initialization command
    #[serde(default)]
    pub init_app: Option<InitApp>,

    /// Directories to create
    #[serde(default)]
    pub directories: Vec<String>,

    /// Template file mappings
    #[serde(default)]
    pub templates: Vec<FileMapping>,

    /// Post-installation actions
    #[serde(default)]
    pub post_install: Option<PostInstall>,

    /// Next steps to show after installation
    #[serde(default)]
    pub next_steps: Vec<String>,
}

/// A service defined by the recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeService {
    /// Service name (can contain <name> placeholder)
    pub name: String,

    /// Service description
    #[serde(default)]
    pub description: Option<String>,
}

/// A configuration option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeOption {
    /// CLI flag (e.g., "--domain")
    #[serde(default)]
    pub flag: Option<String>,

    /// Default value
    #[serde(default)]
    pub default: Option<String>,

    /// Description for help text
    #[serde(default)]
    pub description: Option<String>,

    /// Whether this option is required
    #[serde(default)]
    pub required: bool,

    /// Valid choices (for enum-like options)
    #[serde(default)]
    pub choices: Option<Vec<String>>,
}

/// Placeholder definition for template variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceholderDef {
    /// Source of the value: "name", "option:xyz", etc.
    pub source: String,

    /// Transformation to apply: "slug", "upper", "rust_crate", "ssr_bool"
    #[serde(default)]
    pub transform: Option<String>,
}

/// App initialization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitApp {
    /// Working directory for the command
    #[serde(default)]
    pub cwd: Option<String>,

    /// Target directory to check/create
    #[serde(default)]
    pub target_dir: Option<String>,

    /// Skip if target already exists
    #[serde(default)]
    pub skip_if_exists: bool,

    /// Command to run
    pub command: String,
}

/// File or directory mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMapping {
    /// Source path (relative to recipe directory)
    pub from: String,

    /// Destination path (relative to project root)
    pub to: String,

    /// Optional condition for inclusion
    #[serde(default)]
    pub condition: Option<String>,
}

/// Post-installation actions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostInstall {
    /// Files to create with content
    #[serde(default)]
    pub create_files: Vec<CreateFile>,

    /// Files to rename
    #[serde(default)]
    pub rename: Vec<RenameAction>,

    /// Files to make executable
    #[serde(default)]
    pub chmod: Vec<ChmodAction>,

    /// Empty directories to create with .gitkeep
    #[serde(default)]
    pub gitkeep: Vec<String>,

    /// Commands to run
    #[serde(default)]
    pub run: Vec<RunAction>,

    /// Patterns to add to .gitignore
    #[serde(default)]
    pub gitignore: Vec<String>,
}

/// Post-install action types (union for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PostInstallAction {
    CreateFile(CreateFile),
    Rename(RenameAction),
    Chmod(ChmodAction),
    Run(RunAction),
}

/// Create a file with content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFile {
    pub path: String,
    pub content: String,
}

/// Rename a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameAction {
    pub from: String,
    pub to: String,
}

/// Make a file executable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChmodAction {
    pub path: String,
    #[serde(default = "default_chmod_mode")]
    pub mode: String,
}

fn default_chmod_mode() -> String {
    "+x".to_string()
}

/// Run a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunAction {
    pub command: String,
    #[serde(default)]
    pub cwd: Option<String>,
}

impl Recipe {
    /// Load a recipe from a JSON file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| {
            Error::RecipeNotFound(format!("Failed to read recipe file {}: {}", path.display(), e))
        })?;

        Self::parse(&content)
    }

    /// Parse recipe JSON content
    pub fn parse(content: &str) -> Result<Self> {
        serde_json::from_str(content).map_err(|e| Error::InvalidRecipe(format!("JSON parse error: {}", e)))
    }

    /// Get the display title (falls back to name)
    pub fn display_title(&self) -> &str {
        self.title.as_deref().unwrap_or(&self.name)
    }

    /// Get the description or a default
    pub fn display_description(&self) -> &str {
        self.description.as_deref().unwrap_or("No description available")
    }

    /// Get an option by name
    pub fn get_option(&self, name: &str) -> Option<&RecipeOption> {
        self.options.get(name)
    }

    /// Get option default value
    pub fn get_option_default(&self, name: &str) -> Option<&str> {
        self.options
            .get(name)
            .and_then(|o| o.default.as_deref())
    }

    /// Build placeholder values from service name and option values
    pub fn build_placeholders(
        &self,
        service_name: &str,
        option_values: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut values = HashMap::new();

        for (key, def) in &self.placeholders {
            let raw_value = self.resolve_source(&def.source, service_name, option_values);

            let value = match def.transform.as_deref() {
                Some("slug") => Self::transform_slug(&raw_value),
                Some("upper") => Self::transform_upper(&raw_value),
                Some("rust_crate") => Self::transform_rust_crate(&raw_value),
                Some("ssr_bool") => Self::transform_ssr_bool(&raw_value),
                _ => raw_value,
            };

            values.insert(key.clone(), value);
        }

        // Always include SERVICE_NAME
        values.entry("SERVICE_NAME".to_string()).or_insert_with(|| service_name.to_string());

        values
    }

    /// Resolve a source reference
    fn resolve_source(
        &self,
        source: &str,
        service_name: &str,
        option_values: &HashMap<String, String>,
    ) -> String {
        if source == "name" {
            return service_name.to_string();
        }

        if let Some(option_name) = source.strip_prefix("option:") {
            // First check provided values, then defaults
            if let Some(value) = option_values.get(option_name) {
                return value.clone();
            }
            if let Some(default) = self.get_option_default(option_name) {
                return default.to_string();
            }
        }

        source.to_string()
    }

    /// Transform to slug (kebab-case)
    fn transform_slug(s: &str) -> String {
        s.to_lowercase()
            .replace(' ', "-")
            .replace('_', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect()
    }

    /// Transform to UPPER_SNAKE_CASE
    fn transform_upper(s: &str) -> String {
        s.to_uppercase()
            .replace('-', "_")
            .replace(' ', "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect()
    }

    /// Transform to rust_crate_name
    fn transform_rust_crate(s: &str) -> String {
        s.to_lowercase()
            .replace('-', "_")
            .replace(' ', "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect()
    }

    /// Transform to SSR bool
    fn transform_ssr_bool(s: &str) -> String {
        match s.to_lowercase().as_str() {
            "ssr" | "true" | "yes" | "1" | "on" => "true".to_string(),
            _ => "false".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_recipe() {
        let json = r#"{
            "name": "test",
            "title": "Test Recipe",
            "description": "A test recipe",
            "options": {
                "domain": {
                    "flag": "--domain",
                    "default": "test.localhost"
                }
            },
            "placeholders": {
                "SERVICE_NAME": { "source": "name" },
                "SERVICE_SLUG": { "source": "name", "transform": "slug" }
            },
            "templates": [
                { "from": "app", "to": "apps/{{SERVICE_NAME}}" }
            ]
        }"#;

        let recipe = Recipe::parse(json).unwrap();
        assert_eq!(recipe.name, "test");
        assert_eq!(recipe.display_title(), "Test Recipe");
        assert_eq!(recipe.templates.len(), 1);
    }

    #[test]
    fn test_transforms() {
        assert_eq!(Recipe::transform_slug("My Service"), "my-service");
        assert_eq!(Recipe::transform_upper("my-service"), "MY_SERVICE");
        assert_eq!(Recipe::transform_rust_crate("my-service"), "my_service");
        assert_eq!(Recipe::transform_ssr_bool("ssr"), "true");
        assert_eq!(Recipe::transform_ssr_bool("spa"), "false");
    }
}
