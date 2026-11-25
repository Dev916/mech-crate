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
    /// Optional relevance stage behind retrieval (trial; off by default).
    pub rerank: RerankConfig,
}

/// Relevance-stage settings. Env only: nothing here is read from `rag.toml`.
#[derive(Debug, Clone)]
pub struct RerankConfig {
    /// `MX_RAG_RERANK`. The stage runs only when this is `jev` and a key is set.
    pub provider: Option<String>,
    /// `MX_RAG_JEV_API_KEY`, else `JEV_API_KEY`, else `TYPESAFE_API_KEY`.
    /// A key alone never switches the stage on.
    pub api_key: Option<String>,
    pub base_url: String,
    pub model: String,
    /// Chunks judged below this probability are dropped.
    pub threshold: f64,
    /// Doc-path prefixes whose chunks are never sent to the provider.
    pub private_prefixes: Vec<String>,
    /// `MX_RAG_JEV_TRACE`: append one JSON line per search (measurement).
    pub trace_path: Option<PathBuf>,
}

impl Default for RerankConfig {
    fn default() -> Self {
        Self {
            provider: None,
            api_key: None,
            base_url: "https://api.typesafe.ai/v1".to_string(),
            // Pinned, not an alias: a moving alias invalidates a tuned threshold.
            model: "jev-1.13.0".to_string(),
            // 0.3 lost no answerable query on the judged real-query sample;
            // 0.5 dropped 3 of 27. Higher is stricter.
            threshold: 0.3,
            private_prefixes: vec!["repos/".to_string()],
            trace_path: None,
        }
    }
}

impl RerankConfig {
    fn apply_env(&mut self) {
        if let Ok(v) = std::env::var("MX_RAG_RERANK") {
            self.provider = Some(v);
        }
        if let Ok(v) = std::env::var("MX_RAG_JEV_API_KEY")
            .or_else(|_| std::env::var("JEV_API_KEY"))
            .or_else(|_| std::env::var("TYPESAFE_API_KEY"))
        {
            self.api_key = Some(v);
        }
        if let Ok(v) = std::env::var("MX_RAG_JEV_BASE_URL") {
            self.base_url = v;
        }
        if let Ok(v) = std::env::var("MX_RAG_JEV_MODEL") {
            self.model = v;
        }
        if let Ok(v) = std::env::var("MX_RAG_JEV_THRESHOLD") {
            match v.parse::<f64>() {
                Ok(t) if (0.0..=1.0).contains(&t) => self.threshold = t,
                _ => tracing::warn!("invalid MX_RAG_JEV_THRESHOLD ({v}): using default"),
            }
        }
        if let Ok(v) = std::env::var("MX_RAG_JEV_PRIVATE_PREFIXES") {
            self.private_prefixes = v
                .split(',')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(String::from)
                .collect();
        }
        if let Ok(v) = std::env::var("MX_RAG_JEV_TRACE") {
            self.trace_path = Some(PathBuf::from(v));
        }
    }
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            database_url: None,
            fallback_database_url: "postgres://postgres@localhost:5432/mx_rag".to_string(),
            embedding_base_url: "https://api.openai.com/v1".to_string(),
            embedding_model: "text-embedding-3-small".to_string(),
            embedding_api_key: None,
            rerank: RerankConfig::default(),
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
        cfg.rerank.apply_env();
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

    #[test]
    fn rerank_is_off_by_default_and_private_prefixes_default_to_repos() {
        for k in [
            "MX_RAG_RERANK",
            "MX_RAG_JEV_API_KEY",
            "JEV_API_KEY",
            "TYPESAFE_API_KEY",
        ] {
            std::env::remove_var(k);
        }
        let cfg = RagConfig::load_from(None);
        assert!(cfg.rerank.provider.is_none());
        assert!(cfg.rerank.api_key.is_none());

        // A key in the environment is picked up but does not switch the stage on.
        std::env::set_var("JEV_API_KEY", "k");
        let cfg = RagConfig::load_from(None);
        assert_eq!(cfg.rerank.api_key.as_deref(), Some("k"));
        assert!(cfg.rerank.provider.is_none());
        assert!(crate::corpus::rerank::RerankStage::from_config(&cfg.rerank).is_none());
        std::env::remove_var("JEV_API_KEY");
        assert_eq!(cfg.rerank.private_prefixes, vec!["repos/".to_string()]);
        assert_eq!(cfg.rerank.threshold, 0.3);
        assert_eq!(cfg.rerank.model, "jev-1.13.0");
    }

    #[test]
    fn rerank_env_overrides() {
        std::env::set_var("MX_RAG_RERANK", "jev");
        std::env::set_var("MX_RAG_JEV_API_KEY", "k");
        std::env::set_var("MX_RAG_JEV_THRESHOLD", "0.7");
        std::env::set_var("MX_RAG_JEV_PRIVATE_PREFIXES", "repos/, internal/");
        let cfg = RagConfig::load_from(None);
        assert_eq!(cfg.rerank.provider.as_deref(), Some("jev"));
        assert_eq!(cfg.rerank.api_key.as_deref(), Some("k"));
        assert_eq!(cfg.rerank.threshold, 0.7);
        assert_eq!(cfg.rerank.private_prefixes, vec!["repos/", "internal/"]);

        // Out-of-range threshold keeps the default; empty prefix list sends everything.
        std::env::set_var("MX_RAG_JEV_THRESHOLD", "7");
        std::env::set_var("MX_RAG_JEV_PRIVATE_PREFIXES", "");
        let cfg = RagConfig::load_from(None);
        assert_eq!(cfg.rerank.threshold, 0.3);
        assert!(cfg.rerank.private_prefixes.is_empty());
        for k in [
            "MX_RAG_RERANK",
            "MX_RAG_JEV_API_KEY",
            "MX_RAG_JEV_THRESHOLD",
            "MX_RAG_JEV_PRIVATE_PREFIXES",
        ] {
            std::env::remove_var(k);
        }
    }
}
