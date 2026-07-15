//! CorpusStore: pgvector-backed store with Neon->local fallback.

use std::sync::Arc;
use std::time::Duration;

use sha2::{Digest, Sha256};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::chunk::Chunk;
use super::config::RagConfig;
use super::embed::{EmbeddingProvider, OpenAiCompatEmbedder};

/// Which database the store connected to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Neon,
    Local,
}

impl BackendKind {
    pub fn label(&self) -> &'static str {
        match self {
            BackendKind::Neon => "neon",
            BackendKind::Local => "local",
        }
    }
}

/// Doc-level metadata written to `technique_docs`.
#[derive(Debug, Clone)]
pub struct DocMeta {
    pub path: String,
    pub title: String,
    pub category: String,
    pub languages: Vec<String>,
    pub complexity: String,
    pub use_cases: Vec<String>,
    pub summary: Option<String>,
}

pub struct CorpusStore {
    pool: PgPool,
    backend: BackendKind,
    embedder: Option<Arc<dyn EmbeddingProvider>>,
    model: String,
}

/// Hex-encoded SHA-256 of `data`.
pub fn sha256_hex(data: &str) -> String {
    hex::encode(Sha256::digest(data.as_bytes()))
}

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

impl CorpusStore {
    /// Connect (primary with 5s timeout, else fallback), run migrations,
    /// and build the embedder if an API key is configured.
    pub async fn connect(cfg: &RagConfig) -> anyhow::Result<Self> {
        let (pool, backend) = Self::connect_pool(cfg).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        let embedder: Option<Arc<dyn EmbeddingProvider>> =
            cfg.embedding_api_key.as_ref().map(|key| {
                Arc::new(OpenAiCompatEmbedder::new(
                    &cfg.embedding_base_url,
                    key,
                    &cfg.embedding_model,
                )) as Arc<dyn EmbeddingProvider>
            });
        Ok(Self {
            pool,
            backend,
            embedder,
            model: cfg.embedding_model.clone(),
        })
    }

    async fn connect_pool(cfg: &RagConfig) -> anyhow::Result<(PgPool, BackendKind)> {
        if let Some(primary) = &cfg.database_url {
            let attempt = tokio::time::timeout(
                CONNECT_TIMEOUT,
                PgPoolOptions::new().max_connections(4).connect(primary),
            )
            .await;
            match attempt {
                Ok(Ok(pool)) => return Ok((pool, BackendKind::Neon)),
                Ok(Err(e)) => {
                    tracing::warn!("primary (neon) connect failed: {e}; trying local fallback")
                }
                Err(_) => tracing::warn!("primary (neon) connect timed out; trying local fallback"),
            }
        }
        let pool = tokio::time::timeout(
            CONNECT_TIMEOUT,
            PgPoolOptions::new()
                .max_connections(4)
                .connect(&cfg.fallback_database_url),
        )
        .await
        .map_err(|_| anyhow::anyhow!("local postgres connect timed out"))??;
        Ok((pool, BackendKind::Local))
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn backend(&self) -> BackendKind {
        self.backend
    }

    pub fn has_embedder(&self) -> bool {
        self.embedder.is_some()
    }

    pub fn embedder(&self) -> Option<Arc<dyn EmbeddingProvider>> {
        self.embedder.clone()
    }

    pub fn embedding_model(&self) -> &str {
        &self.model
    }

    /// Stored sha256 for a doc path, if the doc exists.
    pub async fn doc_sha(&self, path: &str) -> anyhow::Result<Option<String>> {
        let row = sqlx::query("SELECT sha256 FROM technique_docs WHERE path = $1")
            .bind(path)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.get::<String, _>("sha256")))
    }

    /// Upsert a doc by unique path; returns the doc id.
    pub async fn upsert_doc(&self, meta: &DocMeta, sha256: &str) -> anyhow::Result<Uuid> {
        let row = sqlx::query(
            "INSERT INTO technique_docs (path, title, category, languages, complexity, use_cases, summary, sha256)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT (path) DO UPDATE SET
                 title = EXCLUDED.title, category = EXCLUDED.category,
                 languages = EXCLUDED.languages, complexity = EXCLUDED.complexity,
                 use_cases = EXCLUDED.use_cases, summary = EXCLUDED.summary,
                 sha256 = EXCLUDED.sha256, updated_at = now()
             RETURNING id",
        )
        .bind(&meta.path)
        .bind(&meta.title)
        .bind(&meta.category)
        .bind(&meta.languages)
        .bind(&meta.complexity)
        .bind(&meta.use_cases)
        .bind(&meta.summary)
        .bind(sha256)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.get::<Uuid, _>("id"))
    }

    /// Remove all chunks for a doc (called before re-inserting changed content).
    pub async fn delete_doc_chunks(&self, doc_id: Uuid) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM technique_chunks WHERE doc_id = $1")
            .bind(doc_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Insert a chunk; returns false when deduped by content_sha256.
    pub async fn insert_chunk(
        &self,
        doc_id: Uuid,
        chunk: &Chunk,
        meta: &DocMeta,
        embedding: Option<Vec<f32>>,
    ) -> anyhow::Result<bool> {
        let csha = sha256_hex(&chunk.content);
        let vector = embedding.map(pgvector::Vector::from);
        let res = sqlx::query(
            "INSERT INTO technique_chunks
                 (doc_id, heading_path, content, embedding, embedding_model, content_sha256, category, languages)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT (content_sha256) DO NOTHING",
        )
        .bind(doc_id)
        .bind(&chunk.heading_path)
        .bind(&chunk.content)
        .bind(vector)
        .bind(&self.model)
        .bind(&csha)
        .bind(&meta.category)
        .bind(&meta.languages)
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected() == 1)
    }

    /// Delete all docs and chunks (chunks cascade).
    pub async fn clear(&self) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM technique_docs")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::chunk::Chunk;

    fn test_cfg() -> Option<RagConfig> {
        let url = std::env::var("MX_RAG_TEST_DATABASE_URL").ok()?;
        Some(RagConfig {
            database_url: None,
            fallback_database_url: url,
            embedding_api_key: None,
            ..RagConfig::default()
        })
    }

    fn meta(path: &str) -> DocMeta {
        DocMeta {
            path: path.into(),
            title: "T".into(),
            category: "patterns".into(),
            languages: vec!["rust".into()],
            complexity: "advanced".into(),
            use_cases: vec!["testing".into()],
            summary: Some("s".into()),
        }
    }

    #[tokio::test]
    async fn connect_migrates_and_falls_back() {
        let Some(mut cfg) = test_cfg() else { return };
        cfg.database_url = Some("postgres://postgres@127.0.0.1:1/nope".into()); // unreachable primary
        let store = CorpusStore::connect(&cfg).await.expect("fallback connect");
        assert_eq!(store.backend().label(), "local");
        // migration ran: table exists
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM technique_docs")
            .fetch_one(store.pool())
            .await
            .unwrap();
        assert!(n >= 0);
    }

    #[tokio::test]
    async fn doc_upsert_idempotent_and_chunk_dedup() {
        let Some(cfg) = test_cfg() else { return };
        let store = CorpusStore::connect(&cfg).await.unwrap();
        store.clear().await.unwrap();

        let m = meta("docs/x.md");
        let sha_v1 = sha256_hex("v1");
        let id = store.upsert_doc(&m, &sha_v1).await.unwrap();
        assert_eq!(
            store.doc_sha("docs/x.md").await.unwrap().as_deref(),
            Some(sha_v1.as_str())
        );

        let c = Chunk {
            heading_path: "T > A".into(),
            content: "T > A\n\nbody".into(),
        };
        assert!(store.insert_chunk(id, &c, &m, None).await.unwrap());
        assert!(!store.insert_chunk(id, &c, &m, None).await.unwrap()); // deduped

        // changed content: upsert same path keeps one doc row, new sha
        let sha_v2 = sha256_hex("v2");
        let id2 = store.upsert_doc(&m, &sha_v2).await.unwrap();
        assert_eq!(id, id2);
        store.delete_doc_chunks(id2).await.unwrap();
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM technique_chunks WHERE doc_id = $1")
            .bind(id2)
            .fetch_one(store.pool())
            .await
            .unwrap();
        assert_eq!(n, 0);
    }
}
