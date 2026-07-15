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

/// Parameters for a technique search.
pub struct TechQuery<'a> {
    pub text: &'a str,
    pub category: Option<&'a str>,
    pub language: Option<&'a str>,
    pub limit: i64,
}

/// How the search was executed.
#[derive(Debug, Clone, Copy)]
pub enum SearchMode {
    Hybrid,
    TrigramOnly,
}

/// One search hit joined with its doc metadata.
#[derive(Debug, Clone)]
pub struct TechHit {
    pub title: String,
    pub path: String,
    pub heading_path: String,
    pub category: String,
    pub languages: Vec<String>,
    pub summary: Option<String>,
    pub use_cases: Vec<String>,
    pub content: String,
    pub score: f64,
}

impl CorpusStore {
    /// Embed the query (when an embedder is configured) and search.
    /// Falls back to trigram-only when there is no embedder or embedding fails.
    pub async fn search(&self, q: &TechQuery<'_>) -> anyhow::Result<(Vec<TechHit>, SearchMode)> {
        let embedding = match &self.embedder {
            Some(e) => match e.embed(q.text).await {
                Ok(v) => Some(v),
                Err(err) => {
                    tracing::warn!("query embedding failed ({err}); trigram-only search");
                    None
                }
            },
            None => None,
        };
        self.search_with_embedding(q, embedding).await
    }

    /// Search with a pre-computed query embedding (testable without an embedder).
    pub async fn search_with_embedding(
        &self,
        q: &TechQuery<'_>,
        embedding: Option<Vec<f32>>,
    ) -> anyhow::Result<(Vec<TechHit>, SearchMode)> {
        let rows = match embedding {
            Some(v) => {
                let vector = pgvector::Vector::from(v);
                sqlx::query(
                    "SELECT d.title, d.path, d.summary, d.use_cases,
                            c.heading_path, c.category, c.languages, c.content,
                            (0.85 * (1 - (c.embedding <=> $1)) + 0.15 * similarity(c.content, $2))::float8 AS score
                       FROM technique_chunks c
                       JOIN technique_docs d ON d.id = c.doc_id
                      WHERE c.embedding IS NOT NULL
                        AND ($3::text IS NULL OR c.category = $3)
                        AND ($4::text IS NULL OR $4 = ANY(c.languages))
                      ORDER BY score DESC
                      LIMIT $5",
                )
                .bind(vector)
                .bind(q.text)
                .bind(q.category)
                .bind(q.language)
                .bind(q.limit)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                let rows = sqlx::query(
                    "SELECT d.title, d.path, d.summary, d.use_cases,
                            c.heading_path, c.category, c.languages, c.content,
                            similarity(c.content, $1)::float8 AS score
                       FROM technique_chunks c
                       JOIN technique_docs d ON d.id = c.doc_id
                      WHERE ($2::text IS NULL OR c.category = $2)
                        AND ($3::text IS NULL OR $3 = ANY(c.languages))
                      ORDER BY score DESC
                      LIMIT $4",
                )
                .bind(q.text)
                .bind(q.category)
                .bind(q.language)
                .bind(q.limit)
                .fetch_all(&self.pool)
                .await?;
                let hits = rows.iter().map(Self::row_to_hit).collect();
                return Ok((hits, SearchMode::TrigramOnly));
            }
        };
        let hits = rows.iter().map(Self::row_to_hit).collect();
        Ok((hits, SearchMode::Hybrid))
    }

    fn row_to_hit(row: &sqlx::postgres::PgRow) -> TechHit {
        TechHit {
            title: row.get("title"),
            path: row.get("path"),
            heading_path: row.get("heading_path"),
            category: row.get("category"),
            languages: row.get("languages"),
            summary: row.get("summary"),
            use_cases: row.get("use_cases"),
            content: row.get("content"),
            score: row.get::<f64, _>("score"),
        }
    }

    /// Corpus health: backend, counts, per-category breakdown, model, last ingest.
    pub async fn status(&self) -> anyhow::Result<serde_json::Value> {
        let docs: i64 = sqlx::query_scalar("SELECT count(*) FROM technique_docs")
            .fetch_one(&self.pool)
            .await?;
        let chunks: i64 = sqlx::query_scalar("SELECT count(*) FROM technique_chunks")
            .fetch_one(&self.pool)
            .await?;
        let last: Option<chrono::DateTime<chrono::Utc>> =
            sqlx::query_scalar("SELECT max(updated_at) FROM technique_docs")
                .fetch_one(&self.pool)
                .await?;
        let by_cat: Vec<(String, i64)> = sqlx::query_as(
            "SELECT category, count(*) FROM technique_chunks GROUP BY category ORDER BY category",
        )
        .fetch_all(&self.pool)
        .await?;
        let stored_models: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT embedding_model FROM technique_chunks ORDER BY embedding_model",
        )
        .fetch_all(&self.pool)
        .await?;
        let model_mismatch =
            !stored_models.is_empty() && stored_models.iter().any(|m| m != &self.model);
        let mut by_category = serde_json::Map::new();
        for (cat, n) in by_cat {
            by_category.insert(cat, serde_json::Value::from(n));
        }
        Ok(serde_json::json!({
            "backend": self.backend.label(),
            "docs": docs,
            "chunks": chunks,
            "embedding_model": self.model,
            "stored_models": stored_models,
            "model_mismatch": model_mismatch,
            "last_ingest": last.map(|t| t.to_rfc3339()),
            "by_category": serde_json::Value::Object(by_category),
        }))
    }

    /// Re-embed every chunk with the configured embedder (provider switch).
    /// Returns the number of chunks updated.
    pub async fn reembed_all(&self) -> anyhow::Result<u64> {
        let embedder = self
            .embedder
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no embedding API key configured"))?;
        let rows: Vec<(Uuid, String)> =
            sqlx::query_as("SELECT id, content FROM technique_chunks ORDER BY created_at")
                .fetch_all(&self.pool)
                .await?;
        let mut updated = 0u64;
        for batch in rows.chunks(super::embed::EMBED_SUB_BATCH) {
            let texts: Vec<String> = batch.iter().map(|(_, c)| c.clone()).collect();
            let vectors = embedder.embed_batch(&texts).await?;
            for ((id, _), v) in batch.iter().zip(vectors) {
                sqlx::query(
                    "UPDATE technique_chunks SET embedding = $1, embedding_model = $2 WHERE id = $3",
                )
                .bind(pgvector::Vector::from(v))
                .bind(&self.model)
                .bind(id)
                .execute(&self.pool)
                .await?;
                updated += 1;
            }
        }
        Ok(updated)
    }
}

/// Serialize DB-touching tests across the whole test binary: they share one
/// Postgres database and each `clear()`s global state then asserts exact
/// counts, so they must not run concurrently. Shared (not per-module) so the
/// `store` and `ingest` DB tests serialize against each other too.
#[cfg(test)]
pub(crate) fn db_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
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
        let _guard = db_lock().lock().await;
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
        let _guard = db_lock().lock().await;
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

    fn sparse(idx: usize) -> Vec<f32> {
        let mut v = vec![0.0_f32; 1536];
        v[idx] = 1.0;
        v
    }

    #[tokio::test]
    async fn hybrid_search_ranks_and_filters() {
        let Some(cfg) = test_cfg() else { return };
        let _guard = db_lock().lock().await;
        let store = CorpusStore::connect(&cfg).await.unwrap();
        store.clear().await.unwrap();

        let m_rust = meta("docs/rust.md");
        let mut m_php = meta("docs/php.md");
        m_php.category = "frp".into();
        m_php.languages = vec!["php".into()];

        let id1 = store.upsert_doc(&m_rust, &sha256_hex("r")).await.unwrap();
        let id2 = store.upsert_doc(&m_php, &sha256_hex("p")).await.unwrap();
        let c1 = Chunk {
            heading_path: "T > Rust".into(),
            content: "T > Rust\n\nlock-free atomics".into(),
        };
        let c2 = Chunk {
            heading_path: "T > Php".into(),
            content: "T > Php\n\nsignals and streams".into(),
        };
        store
            .insert_chunk(id1, &c1, &m_rust, Some(sparse(0)))
            .await
            .unwrap();
        store
            .insert_chunk(id2, &c2, &m_php, Some(sparse(1)))
            .await
            .unwrap();

        // query vector == c1's vector -> c1 first (cosine 1.0)
        let (hits, mode) = store
            .search_with_embedding(
                &TechQuery {
                    text: "atomics",
                    category: None,
                    language: None,
                    limit: 5,
                },
                Some(sparse(0)),
            )
            .await
            .unwrap();
        assert!(matches!(mode, SearchMode::Hybrid));
        assert_eq!(hits[0].heading_path, "T > Rust");

        // category filter excludes patterns doc
        let (hits, _) = store
            .search_with_embedding(
                &TechQuery {
                    text: "x",
                    category: Some("frp"),
                    language: None,
                    limit: 5,
                },
                Some(sparse(1)),
            )
            .await
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].category, "frp");

        // language filter
        let (hits, _) = store
            .search_with_embedding(
                &TechQuery {
                    text: "x",
                    category: None,
                    language: Some("php"),
                    limit: 5,
                },
                Some(sparse(1)),
            )
            .await
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].languages, vec!["php"]);

        // status
        let st = store.status().await.unwrap();
        assert_eq!(st["docs"], 2);
        assert_eq!(st["chunks"], 2);
    }

    #[tokio::test]
    async fn trigram_only_when_no_embeddings() {
        let Some(cfg) = test_cfg() else { return };
        let _guard = db_lock().lock().await;
        let store = CorpusStore::connect(&cfg).await.unwrap();
        store.clear().await.unwrap();
        let m = meta("docs/lex.md");
        let id = store.upsert_doc(&m, &sha256_hex("l")).await.unwrap();
        let c = Chunk {
            heading_path: "T > Lex".into(),
            content: "T > Lex\n\ntrigram lexical matching".into(),
        };
        store.insert_chunk(id, &c, &m, None).await.unwrap();

        let (hits, mode) = store
            .search(&TechQuery {
                text: "trigram lexical",
                category: None,
                language: None,
                limit: 5,
            })
            .await
            .unwrap();
        assert!(matches!(mode, SearchMode::TrigramOnly)); // no embedder configured in test_cfg
        assert_eq!(hits.len(), 1);
    }
}
