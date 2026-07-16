//! RAG corpus configuration: `~/.mech-crate/config/rag.toml` + env overrides.

use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RagConfig {
    pub database_url: Option<String>,
    pub fallback_database_url: String,
    pub embedding_base_url: String,
    pub embedding_model: String,
    pub embedding_api_key: Option<String>,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            database_url: None,
            fallback_database_url: "postgres://postgres@localhost:5432/mx_rag".to_string(),
            embedding_base_url: "https://api.openai.com/v1".to_string(),
            embedding_model: "text-embedding-3-small".to_string(),
            embedding_api_key: None,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RagConfigFile {
    database_url: Option<String>,
    fallback_database_url: Option<String>,
    embedding_base_url: Option<String>,
    embedding_model: Option<String>,
    embedding_api_key: Option<String>,
}

fn default_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".mech-crate/config/rag.toml"))
}

impl RagConfig {
    /// Load from the default path (`~/.mech-crate/config/rag.toml`) + env.
    pub fn load() -> Self {
        Self::load_from(default_path().as_deref())
    }

    /// Precedence: env var > config file > default.
    pub fn load_from(path: Option<&Path>) -> Self {
        let mut cfg = Self::default();
        if let Some(p) = path {
            if let Ok(s) = std::fs::read_to_string(p) {
                match toml::from_str::<RagConfigFile>(&s) {
                    Ok(f) => {
                        if f.database_url.is_some() {
                            cfg.database_url = f.database_url;
                        }
                        if let Some(v) = f.fallback_database_url {
                            cfg.fallback_database_url = v;
                        }
                        if let Some(v) = f.embedding_base_url {
                            cfg.embedding_base_url = v;
                        }
                        if let Some(v) = f.embedding_model {
                            cfg.embedding_model = v;
                        }
                        if f.embedding_api_key.is_some() {
                            cfg.embedding_api_key = f.embedding_api_key;
                        }
                    }
                    Err(e) => tracing::warn!("invalid rag.toml ({}): using defaults", e),
                }
            }
        }
        if let Ok(v) = std::env::var("MX_RAG_DATABASE_URL") {
            cfg.database_url = Some(v);
        }
        if let Ok(v) = std::env::var("MX_RAG_FALLBACK_DATABASE_URL") {
            cfg.fallback_database_url = v;
        }
        if let Ok(v) = std::env::var("MX_RAG_EMBEDDING_BASE_URL") {
            cfg.embedding_base_url = v;
        }
        if let Ok(v) = std::env::var("MX_RAG_EMBEDDING_MODEL") {
            cfg.embedding_model = v;
        }
        if let Ok(v) =
            std::env::var("MX_RAG_EMBEDDING_API_KEY").or_else(|_| std::env::var("OPENAI_API_KEY"))
        {
            cfg.embedding_api_key = Some(v);
        }
        cfg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_when_no_file() {
        let cfg = RagConfig::load_from(None);
        assert!(cfg.database_url.is_none() || std::env::var("MX_RAG_DATABASE_URL").is_ok());
        assert_eq!(cfg.embedding_model, "text-embedding-3-small");
        assert_eq!(cfg.embedding_base_url, "https://api.openai.com/v1");
        assert_eq!(
            cfg.fallback_database_url,
            "postgres://postgres@localhost:5432/mx_rag"
        );
    }

    #[test]
    fn file_then_env_precedence() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("rag.toml");
        std::fs::write(
            &p,
            "database_url = \"postgres://file/db\"\nembedding_model = \"file-model\"\n",
        )
        .unwrap();

        // File values applied
        std::env::remove_var("MX_RAG_DATABASE_URL");
        std::env::remove_var("MX_RAG_EMBEDDING_MODEL");
        let cfg = RagConfig::load_from(Some(&p));
        assert_eq!(cfg.database_url.as_deref(), Some("postgres://file/db"));
        assert_eq!(cfg.embedding_model, "file-model");

        // Env overrides file
        std::env::set_var("MX_RAG_DATABASE_URL", "postgres://env/db");
        std::env::set_var("MX_RAG_EMBEDDING_MODEL", "env-model");
        let cfg = RagConfig::load_from(Some(&p));
        assert_eq!(cfg.database_url.as_deref(), Some("postgres://env/db"));
        assert_eq!(cfg.embedding_model, "env-model");
        std::env::remove_var("MX_RAG_DATABASE_URL");
        std::env::remove_var("MX_RAG_EMBEDDING_MODEL");
    }
}
