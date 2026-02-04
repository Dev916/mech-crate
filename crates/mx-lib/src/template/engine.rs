//! Template engine implementation

use std::collections::HashMap;
use std::path::Path;

use tera::{Context, Tera};

use crate::error::Result;

/// Template engine for processing recipe templates
#[derive(Debug)]
pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();

        // Register custom filters
        tera.register_filter("slug", Self::filter_slug);
        tera.register_filter("upper_snake", Self::filter_upper_snake);
        tera.register_filter("rust_crate", Self::filter_rust_crate);
        tera.register_filter("ssr_bool", Self::filter_ssr_bool);

        Ok(Self { tera })
    }

    /// Render a template string with the given variables
    pub fn render_string(&self, template: &str, vars: &HashMap<String, String>) -> Result<String> {
        let mut context = Context::new();
        for (key, value) in vars {
            context.insert(key, value);
        }

        let result = Tera::one_off(template, &context, false)?;
        Ok(result)
    }

    /// Check if a file is binary (should skip template processing)
    pub fn is_binary_file(path: &Path) -> bool {
        let binary_extensions = [
            "png", "jpg", "jpeg", "gif", "ico", "webp", "svg", "woff", "woff2", "ttf", "eot",
            "otf", "mp3", "mp4", "wav", "avi", "mov", "zip", "tar", "gz", "rar", "7z", "pdf",
            "doc", "docx", "xls", "xlsx", "ppt", "pptx", "exe", "dll", "so", "dylib", "bin",
            "lock",
        ];

        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| binary_extensions.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    /// Slug filter: converts to lowercase kebab-case
    fn filter_slug(
        value: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = value
            .as_str()
            .ok_or_else(|| tera::Error::msg("slug filter requires a string"))?;

        let slug = s
            .to_lowercase()
            .replace(' ', "-")
            .replace('_', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();

        Ok(tera::Value::String(slug))
    }

    /// Upper snake case filter: MY_VARIABLE_NAME
    fn filter_upper_snake(
        value: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = value
            .as_str()
            .ok_or_else(|| tera::Error::msg("upper_snake filter requires a string"))?;

        let result = s
            .to_uppercase()
            .replace('-', "_")
            .replace(' ', "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>();

        Ok(tera::Value::String(result))
    }

    /// Rust crate name filter: my_crate_name
    fn filter_rust_crate(
        value: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = value
            .as_str()
            .ok_or_else(|| tera::Error::msg("rust_crate filter requires a string"))?;

        let result = s
            .to_lowercase()
            .replace('-', "_")
            .replace(' ', "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>();

        Ok(tera::Value::String(result))
    }

    /// SSR bool filter: converts "true"/"false" strings to Rust bool literals
    fn filter_ssr_bool(
        value: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = value
            .as_str()
            .ok_or_else(|| tera::Error::msg("ssr_bool filter requires a string"))?;

        let result = match s.to_lowercase().as_str() {
            "true" | "yes" | "1" | "on" => "true",
            _ => "false",
        };

        Ok(tera::Value::String(result.to_string()))
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default template engine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_string() {
        let engine = TemplateEngine::new().unwrap();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "my-project".to_string());

        let result = engine
            .render_string("Hello {{ name }}!", &vars)
            .unwrap();
        assert_eq!(result, "Hello my-project!");
    }

    #[test]
    fn test_is_binary_file() {
        assert!(TemplateEngine::is_binary_file(Path::new("image.png")));
        assert!(TemplateEngine::is_binary_file(Path::new("font.woff2")));
        assert!(!TemplateEngine::is_binary_file(Path::new("config.yml")));
        assert!(!TemplateEngine::is_binary_file(Path::new("main.rs")));
    }
}
