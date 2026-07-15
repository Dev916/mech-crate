# Techniques RAG Library Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Weaviate RAG stack in mech-crate with a pgvector corpus (Neon primary, local Postgres fallback) of development techniques from `docs/development`, exposed via mx MCP tools, an `mx rag` CLI, a `techniques` skill, and devloop/writing-devloop-plans hooks.

**Architecture:** A new `corpus` module in `mx-lib` (shared by `mx-cli` and `mx-mcp-server`) ports the hq corpus pattern: `technique_docs`/`technique_chunks` tables with HNSW cosine + pg_trgm indexes, hybrid search `0.85*cosine + 0.15*trigram`, an `EmbeddingProvider` trait with an OpenAI-compatible first implementation, frontmatter-driven metadata, and idempotent ingestion. The MCP server's 7 `rag_*` tools are re-pointed at the corpus and one new `rag_context` tool is added. All Weaviate code is deleted.

**Tech Stack:** Rust 2021 workspace (mx-lib, mx-cli, mx-mcp-server), sqlx 0.8 (postgres, runtime-tokio-rustls, migrate, uuid, chrono), pgvector =0.4.1 (sqlx feature), serde_yaml, toml, sha2 + hex, reqwest, wiremock (tests), Neon Postgres 17, pgvector/pgvector:pg17 for local.

**Spec:** `docs/superpowers/specs/2026-07-15-techniques-rag-design.md`

**Compatible with:** devloop skill v0.1+

## Global Constraints

- All corpus code lives in `crates/mx-lib/src/corpus/`; internal error type is `anyhow::Result` (hq convention).
- NO sqlx compile-time macros (`query!`, `query_as!`) — runtime `sqlx::query`/`query_as` only, so no DATABASE_URL is needed at build time.
- Embedding dims are 1536 (`text-embedding-3-small`); `embedding_model` recorded per chunk; hybrid weights exactly `0.85` / `0.15`.
- Chunk size cap exactly 1200 chars (`DEFAULT_CHUNK_CHARS`).
- Config file: `~/.mech-crate/config/rag.toml`; env overrides `MX_RAG_DATABASE_URL`, `MX_RAG_FALLBACK_DATABASE_URL`, `MX_RAG_EMBEDDING_BASE_URL`, `MX_RAG_EMBEDDING_MODEL`, `MX_RAG_EMBEDDING_API_KEY` (falls back to `OPENAI_API_KEY`). Never log API keys.
- Default fallback DB URL: `postgres://postgres@localhost:5432/mx_rag`. Local test DB for integration tests: start with `docker run -d --name mx-rag-test -p 55433:5432 -e POSTGRES_DB=mx_rag -e POSTGRES_HOST_AUTH_METHOD=trust pgvector/pgvector:pg17` and set `MX_RAG_TEST_DATABASE_URL=postgres://postgres@localhost:55433/mx_rag`. DB-touching tests return early (skip) when that env var is unset.
- The MCP server must NEVER crash because the corpus is unreachable — tools degrade to an actionable offline message.
- Conventional commit per task. Run `cargo fmt` before each commit.
- Historical docs (`docs/architecture-review-2026-03-07.md`, `docs/unyform/*`) are exempt from the Weaviate-reference sweep; the spec and this plan are too.

---

### Task 1: Dependencies, migration, corpus module scaffold

**Acceptance Criteria (observable):**
- `cargo check --workspace` exits 0 with the new deps and empty corpus module in place.
- `crates/mx-lib/migrations/0001_technique_corpus.sql` exists and contains `CREATE EXTENSION IF NOT EXISTS vector`, a `technique_docs` table with `path TEXT UNIQUE NOT NULL`, a `technique_chunks` table with `embedding vector(1536)` and `content_sha256 TEXT UNIQUE`, an HNSW index, and a `gin_trgm_ops` index.

**Verify via:** cli

**Files:**
- Modify: `Cargo.toml` (workspace deps)
- Modify: `crates/mx-lib/Cargo.toml`
- Create: `crates/mx-lib/migrations/0001_technique_corpus.sql`
- Create: `crates/mx-lib/src/corpus/mod.rs`
- Modify: `crates/mx-lib/src/lib.rs` (add `pub mod corpus;`)

**Interfaces:**
- Produces: `mx_lib::corpus` module path; migration embedded later via `sqlx::migrate!("./migrations")` (path relative to mx-lib crate root).

- [ ] **Step 1: Add workspace dependencies**

In root `Cargo.toml` under `[workspace.dependencies]`, append:

```toml
# Database (pgvector corpus)
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "migrate"] }
pgvector = { version = "=0.4.1", features = ["sqlx"] }

# YAML frontmatter
serde_yaml = "0.9"

# TOML config
toml = "0.8"

# Hashing
sha2 = "0.10"
hex = "0.4"
```

- [ ] **Step 2: Add mx-lib dependencies**

In `crates/mx-lib/Cargo.toml` under `[dependencies]`, append:

```toml
# Techniques corpus (pgvector)
sqlx = { workspace = true }
pgvector = { workspace = true }
serde_yaml = { workspace = true }
toml = { workspace = true }
sha2 = { workspace = true }
hex = { workspace = true }
```

And under `[dev-dependencies]` append:

```toml
wiremock = { workspace = true }
```

- [ ] **Step 3: Create the migration**

Create `crates/mx-lib/migrations/0001_technique_corpus.sql`:

```sql
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE IF NOT EXISTS technique_docs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    path TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'other',
    languages TEXT[] NOT NULL DEFAULT '{}',
    complexity TEXT NOT NULL DEFAULT 'intermediate',
    use_cases TEXT[] NOT NULL DEFAULT '{}',
    summary TEXT,
    sha256 TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS technique_chunks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    doc_id UUID NOT NULL REFERENCES technique_docs(id) ON DELETE CASCADE,
    heading_path TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL,
    embedding vector(1536),
    embedding_model TEXT NOT NULL,
    content_sha256 TEXT UNIQUE NOT NULL,
    category TEXT NOT NULL DEFAULT 'other',
    languages TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS technique_chunks_embedding_hnsw
    ON technique_chunks USING hnsw (embedding vector_cosine_ops);
CREATE INDEX IF NOT EXISTS technique_chunks_content_trgm
    ON technique_chunks USING gin (content gin_trgm_ops);
CREATE INDEX IF NOT EXISTS technique_chunks_category_idx ON technique_chunks (category);
```

- [ ] **Step 4: Scaffold the module**

Create `crates/mx-lib/src/corpus/mod.rs`:

```rust
//! Techniques corpus: pgvector-backed RAG store for development techniques.
//!
//! Ported from the hq corpus pattern (~/dev/hq): doc/chunk tables with sha256
//! idempotency, hybrid (cosine + trigram) search, OpenAI-compatible embeddings
//! behind the `EmbeddingProvider` trait.

pub mod chunk;
pub mod config;
pub mod embed;
pub mod frontmatter;
pub mod ingest;
pub mod store;

pub use config::RagConfig;
pub use store::{CorpusStore, SearchMode, TechHit, TechQuery};
```

Create empty placeholder files so it compiles: `chunk.rs`, `config.rs`, `embed.rs`, `frontmatter.rs`, `ingest.rs`, `store.rs` each containing only a module doc comment (e.g. `//! Heading-aware markdown chunker.`) — their contents arrive in Tasks 2–8. Comment out the `pub use` lines and all `pub mod` lines except the ones whose files exist with content; simplest: create all six files with doc comments only and keep `mod.rs` exactly as above minus the `pub use` line (add the `pub use` back in Task 7 when the types exist).

In `crates/mx-lib/src/lib.rs` add `pub mod corpus;` alongside the existing module declarations.

- [ ] **Step 5: Verify compile and commit**

Run: `cargo check --workspace`
Expected: exit 0.

```bash
git add -A
git commit -m "feat(corpus): add pgvector deps, migration, and corpus module scaffold"
```

---

### Task 2: Frontmatter parser

**Acceptance Criteria (observable):**
- `cargo test -p mx-lib corpus::frontmatter` exits 0 with tests covering: valid frontmatter parsed, missing frontmatter returns None + full body, malformed YAML returns None + full body.

**Verify via:** cli

**Files:**
- Modify: `crates/mx-lib/src/corpus/frontmatter.rs`

**Interfaces:**
- Produces: `TechniqueMeta { title: Option<String>, category: Option<String>, languages: Vec<String>, complexity: Option<String>, use_cases: Vec<String>, summary: Option<String> }` and `parse_frontmatter(content: &str) -> (Option<TechniqueMeta>, &str)` where the `&str` is the body with frontmatter stripped.

- [ ] **Step 1: Write the failing tests**

Append to `crates/mx-lib/src/corpus/frontmatter.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_frontmatter() {
        let doc = "---\ntitle: Rust Concurrency\ncategory: concurrency\nlanguages: [rust]\ncomplexity: expert\nuse_cases:\n  - lock-free structures\nsummary: Deep dive.\n---\n\n# Body\n\ntext";
        let (meta, body) = parse_frontmatter(doc);
        let meta = meta.expect("meta");
        assert_eq!(meta.title.as_deref(), Some("Rust Concurrency"));
        assert_eq!(meta.category.as_deref(), Some("concurrency"));
        assert_eq!(meta.languages, vec!["rust"]);
        assert_eq!(meta.complexity.as_deref(), Some("expert"));
        assert_eq!(meta.use_cases, vec!["lock-free structures"]);
        assert!(body.starts_with("# Body"));
    }

    #[test]
    fn missing_frontmatter_returns_none_and_full_body() {
        let doc = "# Just a doc\n\ncontent";
        let (meta, body) = parse_frontmatter(doc);
        assert!(meta.is_none());
        assert_eq!(body, doc);
    }

    #[test]
    fn malformed_yaml_returns_none_and_full_body() {
        let doc = "---\ntitle: [unclosed\n---\nbody";
        let (meta, body) = parse_frontmatter(doc);
        assert!(meta.is_none());
        assert_eq!(body, doc);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p mx-lib corpus::frontmatter`
Expected: FAIL (compile error — `parse_frontmatter` not defined).

- [ ] **Step 3: Implement**

Write above the tests in `frontmatter.rs`:

```rust
//! YAML frontmatter parser for technique docs.

use serde::Deserialize;

/// Metadata parsed from a technique doc's YAML frontmatter.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct TechniqueMeta {
    pub title: Option<String>,
    pub category: Option<String>,
    pub languages: Vec<String>,
    pub complexity: Option<String>,
    pub use_cases: Vec<String>,
    pub summary: Option<String>,
}

/// Split a markdown document into (frontmatter, body).
///
/// Frontmatter must start at byte 0 with `---\n` and end at the next line
/// equal to `---`. Malformed YAML yields `(None, full_content)` — callers
/// warn and fall back to heuristics; ingestion never aborts on one bad doc.
pub fn parse_frontmatter(content: &str) -> (Option<TechniqueMeta>, &str) {
    let Some(rest) = content.strip_prefix("---\n") else {
        return (None, content);
    };
    let Some(end) = rest.find("\n---") else {
        return (None, content);
    };
    let yaml = &rest[..end];
    let after = &rest[end + 4..];
    let body = after.trim_start_matches('\n');
    match serde_yaml::from_str::<TechniqueMeta>(yaml) {
        Ok(meta) => (Some(meta), body),
        Err(_) => (None, content),
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p mx-lib corpus::frontmatter`
Expected: 3 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/mx-lib/src/corpus/frontmatter.rs
git commit -m "feat(corpus): YAML frontmatter parser with graceful fallback"
```

---

### Task 3: Heading-aware chunker

**Acceptance Criteria (observable):**
- `cargo test -p mx-lib corpus::chunk` exits 0 with tests covering: split on `##` headings, oversized sections sub-split under the cap at paragraph boundaries, every chunk content prefixed with `Doc Title > Heading`, preamble before the first `##` chunked under the doc title alone.

**Verify via:** cli

**Files:**
- Modify: `crates/mx-lib/src/corpus/chunk.rs`

**Interfaces:**
- Produces: `DEFAULT_CHUNK_CHARS: usize = 1200`, `Chunk { heading_path: String, content: String }`, `chunk_markdown(doc_title: &str, body: &str, max_chars: usize) -> Vec<Chunk>`.

- [ ] **Step 1: Write the failing tests**

Append to `crates/mx-lib/src/corpus/chunk.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_h2_headings() {
        let body = "intro para\n\n## First\n\nalpha\n\n## Second\n\nbeta";
        let chunks = chunk_markdown("Doc", body, 1200);
        let paths: Vec<_> = chunks.iter().map(|c| c.heading_path.as_str()).collect();
        assert_eq!(paths, vec!["Doc", "Doc > First", "Doc > Second"]);
    }

    #[test]
    fn chunk_content_is_prefixed_with_heading_path() {
        let body = "## First\n\nalpha";
        let chunks = chunk_markdown("Doc", body, 1200);
        assert!(chunks[0].content.starts_with("Doc > First\n\n"));
        assert!(chunks[0].content.contains("alpha"));
    }

    #[test]
    fn oversized_section_sub_splits_under_cap() {
        let para = "x".repeat(500);
        let body = format!("## Big\n\n{para}\n\n{para}\n\n{para}");
        let chunks = chunk_markdown("Doc", &body, 600);
        assert!(chunks.len() >= 3);
        assert!(chunks.iter().all(|c| c.content.len() <= 600 + "Doc > Big\n\n".len()));
        assert!(chunks.iter().all(|c| c.heading_path == "Doc > Big"));
    }

    #[test]
    fn empty_body_yields_no_chunks() {
        assert!(chunk_markdown("Doc", "   \n  ", 1200).is_empty());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p mx-lib corpus::chunk`
Expected: FAIL (compile error).

- [ ] **Step 3: Implement**

Write above the tests in `chunk.rs`:

```rust
//! Heading-aware markdown chunker.
//!
//! Splits on `##` headings; sections over `max_chars` are sub-split on
//! blank-line paragraph boundaries (hq `chunk_text` port). Every chunk's
//! content is prefixed with its `Doc Title > Heading` path so chunks are
//! self-contextualizing when retrieved in isolation.

/// Default chunk size cap in characters.
pub const DEFAULT_CHUNK_CHARS: usize = 1200;

/// One retrievable chunk of a technique doc.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub heading_path: String,
    pub content: String,
}

/// Chunk a markdown body under `doc_title`.
pub fn chunk_markdown(doc_title: &str, body: &str, max_chars: usize) -> Vec<Chunk> {
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut current_heading = String::new();
    let mut current = String::new();

    for line in body.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            sections.push((current_heading.clone(), std::mem::take(&mut current)));
            current_heading = h.trim().to_string();
        } else if line.starts_with("# ") {
            // Top-level heading: part of the preamble text, not a section break.
            current.push_str(line);
            current.push('\n');
        } else {
            current.push_str(line);
            current.push('\n');
        }
    }
    sections.push((current_heading, current));

    let mut chunks = Vec::new();
    for (heading, text) in sections {
        if text.trim().is_empty() {
            continue;
        }
        let heading_path = if heading.is_empty() {
            doc_title.to_string()
        } else {
            format!("{} > {}", doc_title, heading)
        };
        for piece in pack_paragraphs(&text, max_chars) {
            chunks.push(Chunk {
                heading_path: heading_path.clone(),
                content: format!("{}\n\n{}", heading_path, piece),
            });
        }
    }
    chunks
}

/// hq `chunk_text` port: pack paragraphs into pieces up to `max_chars`;
/// hard-split single oversized paragraphs.
fn pack_paragraphs(text: &str, max_chars: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for para in text.split("\n\n") {
        let para = para.trim();
        if para.is_empty() {
            continue;
        }
        if para.len() > max_chars {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
            out.extend(hard_split(para, max_chars));
            continue;
        }
        if current.len() + para.len() + 2 > max_chars && !current.is_empty() {
            out.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push_str("\n\n");
        }
        current.push_str(para);
    }
    if !current.trim().is_empty() {
        out.push(current);
    }
    out
}

fn hard_split(s: &str, max: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    for ch in s.chars() {
        if buf.len() + ch.len_utf8() > max {
            out.push(std::mem::take(&mut buf));
        }
        buf.push(ch);
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    out
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p mx-lib corpus::chunk`
Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/mx-lib/src/corpus/chunk.rs
git commit -m "feat(corpus): heading-aware chunker with paragraph packing"
```

---

### Task 4: EmbeddingProvider trait + OpenAI-compatible embedder

**Acceptance Criteria (observable):**
- `cargo test -p mx-lib corpus::embed` exits 0 with wiremock-backed tests covering: single embed parses `data[0].embedding`, batch embed returns vectors in input order (sorted by `index`), non-2xx yields Err. (Sub-batching at 256 is exercised implicitly by `embed_batch`'s chunking; no dedicated test.)

**Verify via:** cli

**Files:**
- Modify: `crates/mx-lib/src/corpus/embed.rs`

**Interfaces:**
- Produces:
  ```rust
  #[async_trait]
  pub trait EmbeddingProvider: Send + Sync {
      fn model(&self) -> &str;
      async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>>;
      async fn embed_batch(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>>;
  }
  pub struct OpenAiCompatEmbedder { /* http, base_url, api_key, model */ }
  impl OpenAiCompatEmbedder { pub fn new(base_url: &str, api_key: &str, model: &str) -> Self }
  pub const EMBED_SUB_BATCH: usize = 256;
  ```

- [ ] **Step 1: Write the failing tests**

Append to `crates/mx-lib/src/corpus/embed.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn embedding_response(n: usize) -> serde_json::Value {
        let data: Vec<_> = (0..n)
            .map(|i| serde_json::json!({ "index": n - 1 - i, "embedding": [ (n - 1 - i) as f64, 0.0 ] }))
            .collect();
        serde_json::json!({ "data": data })
    }

    #[tokio::test]
    async fn embed_single_parses_vector() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(embedding_response(1)))
            .mount(&server)
            .await;
        let e = OpenAiCompatEmbedder::new(&server.uri(), "test-key", "test-model");
        let v = e.embed("hello").await.unwrap();
        assert_eq!(v.len(), 2);
    }

    #[tokio::test]
    async fn embed_batch_orders_by_index() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(embedding_response(3)))
            .mount(&server)
            .await;
        let e = OpenAiCompatEmbedder::new(&server.uri(), "k", "m");
        let texts: Vec<String> = (0..3).map(|i| format!("t{i}")).collect();
        let vs = e.embed_batch(&texts).await.unwrap();
        // Response listed indices in reverse; output must be input-ordered.
        assert_eq!(vs[0][0], 0.0);
        assert_eq!(vs[2][0], 2.0);
    }

    #[tokio::test]
    async fn http_error_is_err() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;
        let e = OpenAiCompatEmbedder::new(&server.uri(), "k", "m");
        assert!(e.embed("x").await.is_err());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p mx-lib corpus::embed`
Expected: FAIL (compile error).

- [ ] **Step 3: Implement**

Write above the tests in `embed.rs` (hq `llm/openai.rs` embeddings port):

```rust
//! Embedding providers behind the `EmbeddingProvider` trait (adapter pattern).
//!
//! First implementation: any OpenAI-compatible `/embeddings` endpoint
//! (OpenAI, Ollama's /v1, LM Studio, ...). Model + dims are recorded per
//! chunk in the store, so a provider switch cannot silently mix vector spaces.

use async_trait::async_trait;
use serde_json::json;

/// Max inputs per `/embeddings` request (hq convention).
pub const EMBED_SUB_BATCH: usize = 256;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    fn model(&self) -> &str;
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>>;
    async fn embed_batch(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>>;
}

pub struct OpenAiCompatEmbedder {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAiCompatEmbedder {
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }

    fn parse_embedding(item: &serde_json::Value) -> anyhow::Result<Vec<f32>> {
        let arr = item["embedding"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("missing embedding in response"))?;
        Ok(arr.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect())
    }

    async fn post(&self, input: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let resp = self
            .http
            .post(format!("{}/embeddings", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&json!({ "model": self.model, "input": input }))
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("embeddings http {}", resp.status()));
        }
        Ok(resp.json().await?)
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAiCompatEmbedder {
    fn model(&self) -> &str {
        &self.model
    }

    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let v = self.post(json!(text)).await?;
        Self::parse_embedding(&v["data"][0])
    }

    /// Sub-batches at [`EMBED_SUB_BATCH`]; sorts each response by `data[i].index`
    /// so output order matches input order regardless of API ordering.
    async fn embed_batch(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
        let mut out: Vec<Vec<f32>> = Vec::with_capacity(texts.len());
        for sub in texts.chunks(EMBED_SUB_BATCH) {
            let v = self.post(json!(sub)).await?;
            let data = v["data"]
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("missing embeddings data"))?;
            let mut indexed: Vec<(usize, Vec<f32>)> = Vec::with_capacity(data.len());
            for (i, item) in data.iter().enumerate() {
                let idx = item["index"].as_u64().map(|x| x as usize).unwrap_or(i);
                indexed.push((idx, Self::parse_embedding(item)?));
            }
            indexed.sort_by_key(|(i, _)| *i);
            out.extend(indexed.into_iter().map(|(_, e)| e));
        }
        Ok(out)
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p mx-lib corpus::embed`
Expected: 3 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/mx-lib/src/corpus/embed.rs
git commit -m "feat(corpus): EmbeddingProvider trait + OpenAI-compatible embedder"
```

---

### Task 5: RagConfig loading

**Acceptance Criteria (observable):**
- `cargo test -p mx-lib corpus::config` exits 0 with tests covering: defaults when no file/env, file values applied, env vars override file values.

**Verify via:** cli

**Files:**
- Modify: `crates/mx-lib/src/corpus/config.rs`

**Interfaces:**
- Produces:
  ```rust
  pub struct RagConfig {
      pub database_url: Option<String>,          // Neon primary
      pub fallback_database_url: String,         // default postgres://postgres@localhost:5432/mx_rag
      pub embedding_base_url: String,            // default https://api.openai.com/v1
      pub embedding_model: String,               // default text-embedding-3-small
      pub embedding_api_key: Option<String>,
  }
  impl RagConfig {
      pub fn load() -> Self;                             // ~/.mech-crate/config/rag.toml + env
      pub fn load_from(path: Option<&Path>) -> Self;     // testable entry
  }
  ```

- [ ] **Step 1: Write the failing tests**

Append to `crates/mx-lib/src/corpus/config.rs` (note: env-var tests mutate process env — keep them in ONE test function to avoid parallel-test races):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_when_no_file() {
        let cfg = RagConfig::load_from(None);
        assert!(cfg.database_url.is_none() || std::env::var("MX_RAG_DATABASE_URL").is_ok());
        assert_eq!(cfg.embedding_model, "text-embedding-3-small");
        assert_eq!(cfg.embedding_base_url, "https://api.openai.com/v1");
        assert_eq!(cfg.fallback_database_url, "postgres://postgres@localhost:5432/mx_rag");
    }

    #[test]
    fn file_then_env_precedence() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("rag.toml");
        std::fs::write(&p, "database_url = \"postgres://file/db\"\nembedding_model = \"file-model\"\n").unwrap();

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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p mx-lib corpus::config`
Expected: FAIL (compile error).

- [ ] **Step 3: Implement**

Write above the tests in `config.rs`:

```rust
//! RAG corpus configuration: ~/.mech-crate/config/rag.toml + env overrides.

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
        if let Ok(v) = std::env::var("MX_RAG_EMBEDDING_API_KEY").or_else(|_| std::env::var("OPENAI_API_KEY")) {
            cfg.embedding_api_key = Some(v);
        }
        cfg
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p mx-lib corpus::config`
Expected: 2 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/mx-lib/src/corpus/config.rs
git commit -m "feat(corpus): RagConfig with file + env precedence"
```

---

### Task 6: CorpusStore — connect, migrate, upserts

**Acceptance Criteria (observable):**
- With `MX_RAG_TEST_DATABASE_URL` set (local pgvector container per Global Constraints), `cargo test -p mx-lib corpus::store` exits 0, covering: connect runs migrations (tables exist), fallback used when primary URL is bad, doc upsert by path is idempotent (same sha skipped, changed sha replaces chunks), chunk insert dedups on content_sha256.
- Without `MX_RAG_TEST_DATABASE_URL`, the same command exits 0 (tests skip early).

**Verify via:** cli

**Files:**
- Modify: `crates/mx-lib/src/corpus/store.rs`

**Interfaces:**
- Consumes: `RagConfig` (Task 5), `EmbeddingProvider`/`OpenAiCompatEmbedder` (Task 4), `Chunk` (Task 3).
- Produces:
  ```rust
  pub enum BackendKind { Neon, Local }           // .label() -> "neon" | "local"
  pub struct DocMeta { pub path: String, pub title: String, pub category: String,
                       pub languages: Vec<String>, pub complexity: String,
                       pub use_cases: Vec<String>, pub summary: Option<String> }
  pub struct CorpusStore { /* pool, backend, embedder: Option<Arc<dyn EmbeddingProvider>>, model */ }
  impl CorpusStore {
      pub async fn connect(cfg: &RagConfig) -> anyhow::Result<Self>;
      pub fn backend(&self) -> BackendKind;
      pub fn has_embedder(&self) -> bool;
      pub async fn doc_sha(&self, path: &str) -> anyhow::Result<Option<String>>;
      pub async fn upsert_doc(&self, meta: &DocMeta, sha256: &str) -> anyhow::Result<uuid::Uuid>;
      pub async fn delete_doc_chunks(&self, doc_id: uuid::Uuid) -> anyhow::Result<()>;
      pub async fn insert_chunk(&self, doc_id: uuid::Uuid, chunk: &Chunk, meta: &DocMeta,
                                embedding: Option<Vec<f32>>) -> anyhow::Result<bool>; // false if deduped
      pub async fn clear(&self) -> anyhow::Result<()>;
      pub fn embedder(&self) -> Option<std::sync::Arc<dyn EmbeddingProvider>>;
      pub fn embedding_model(&self) -> &str;
  }
  pub fn sha256_hex(data: &str) -> String;
  ```

- [ ] **Step 1: Write the failing tests**

Append to `crates/mx-lib/src/corpus/store.rs`:

```rust
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
        assert_eq!(store.doc_sha("docs/x.md").await.unwrap().as_deref(), Some(sha_v1.as_str()));

        let c = Chunk { heading_path: "T > A".into(), content: "T > A\n\nbody".into() };
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p mx-lib corpus::store`
Expected: FAIL (compile error).

- [ ] **Step 3: Implement**

Write above the tests in `store.rs`:

```rust
//! CorpusStore: pgvector-backed store with Neon→local fallback.

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
        let embedder: Option<Arc<dyn EmbeddingProvider>> = cfg.embedding_api_key.as_ref().map(|key| {
            Arc::new(OpenAiCompatEmbedder::new(&cfg.embedding_base_url, key, &cfg.embedding_model))
                as Arc<dyn EmbeddingProvider>
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
                Ok(Err(e)) => tracing::warn!("primary (neon) connect failed: {e}; trying local fallback"),
                Err(_) => tracing::warn!("primary (neon) connect timed out; trying local fallback"),
            }
        }
        let pool = tokio::time::timeout(
            CONNECT_TIMEOUT,
            PgPoolOptions::new().max_connections(4).connect(&cfg.fallback_database_url),
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
        sqlx::query("DELETE FROM technique_docs").execute(&self.pool).await?;
        Ok(())
    }
}
```

- [ ] **Step 4: Start the test DB and run tests**

```bash
docker rm -f mx-rag-test 2>/dev/null; docker run -d --name mx-rag-test -p 55433:5432 -e POSTGRES_DB=mx_rag -e POSTGRES_HOST_AUTH_METHOD=trust pgvector/pgvector:pg17
sleep 5
MX_RAG_TEST_DATABASE_URL=postgres://postgres@localhost:55433/mx_rag cargo test -p mx-lib corpus::store
```
Expected: 2 passed. Also run `cargo test -p mx-lib corpus::store` WITHOUT the env var: passes (skips).

- [ ] **Step 5: Commit**

```bash
git add crates/mx-lib/src/corpus/store.rs
git commit -m "feat(corpus): CorpusStore with Neon->local fallback, migrations, idempotent upserts"
```

---

### Task 7: Hybrid search, status, reembed

**Acceptance Criteria (observable):**
- With `MX_RAG_TEST_DATABASE_URL` set, `cargo test -p mx-lib corpus::store` exits 0, additionally covering: hybrid search ranks an exact-cosine-match chunk first, category and language filters exclude non-matching chunks, trigram-only mode returns results when chunks have no embeddings, status reports doc/chunk counts and per-category breakdown.

**Verify via:** cli

**Files:**
- Modify: `crates/mx-lib/src/corpus/store.rs`
- Modify: `crates/mx-lib/src/corpus/mod.rs` (enable the `pub use` re-exports from Task 1)

**Interfaces:**
- Produces:
  ```rust
  pub struct TechQuery<'a> { pub text: &'a str, pub category: Option<&'a str>,
                             pub language: Option<&'a str>, pub limit: i64 }
  pub enum SearchMode { Hybrid, TrigramOnly }
  pub struct TechHit { pub title: String, pub path: String, pub heading_path: String,
                       pub category: String, pub languages: Vec<String>,
                       pub summary: Option<String>, pub use_cases: Vec<String>,
                       pub content: String, pub score: f64 }
  impl CorpusStore {
      pub async fn search(&self, q: &TechQuery<'_>) -> anyhow::Result<(Vec<TechHit>, SearchMode)>;
      pub async fn status(&self) -> anyhow::Result<serde_json::Value>;
      pub async fn reembed_all(&self) -> anyhow::Result<u64>;
  }
  ```

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module in `store.rs` (sparse-vector trick from hq: disjoint hot indices are orthogonal):

```rust
    fn sparse(idx: usize) -> Vec<f32> {
        let mut v = vec![0.0_f32; 1536];
        v[idx] = 1.0;
        v
    }

    #[tokio::test]
    async fn hybrid_search_ranks_and_filters() {
        let Some(cfg) = test_cfg() else { return };
        let store = CorpusStore::connect(&cfg).await.unwrap();
        store.clear().await.unwrap();

        let m_rust = meta("docs/rust.md");
        let mut m_php = meta("docs/php.md");
        m_php.category = "frp".into();
        m_php.languages = vec!["php".into()];

        let id1 = store.upsert_doc(&m_rust, &sha256_hex("r")).await.unwrap();
        let id2 = store.upsert_doc(&m_php, &sha256_hex("p")).await.unwrap();
        let c1 = Chunk { heading_path: "T > Rust".into(), content: "T > Rust\n\nlock-free atomics".into() };
        let c2 = Chunk { heading_path: "T > Php".into(), content: "T > Php\n\nsignals and streams".into() };
        store.insert_chunk(id1, &c1, &m_rust, Some(sparse(0))).await.unwrap();
        store.insert_chunk(id2, &c2, &m_php, Some(sparse(1))).await.unwrap();

        // query vector == c1's vector -> c1 first (cosine 1.0)
        let (hits, mode) = store
            .search_with_embedding(&TechQuery { text: "atomics", category: None, language: None, limit: 5 }, Some(sparse(0)))
            .await
            .unwrap();
        assert!(matches!(mode, SearchMode::Hybrid));
        assert_eq!(hits[0].heading_path, "T > Rust");

        // category filter excludes patterns doc
        let (hits, _) = store
            .search_with_embedding(&TechQuery { text: "x", category: Some("frp"), language: None, limit: 5 }, Some(sparse(1)))
            .await
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].category, "frp");

        // language filter
        let (hits, _) = store
            .search_with_embedding(&TechQuery { text: "x", category: None, language: Some("php"), limit: 5 }, Some(sparse(1)))
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
        let store = CorpusStore::connect(&cfg).await.unwrap();
        store.clear().await.unwrap();
        let m = meta("docs/lex.md");
        let id = store.upsert_doc(&m, &sha256_hex("l")).await.unwrap();
        let c = Chunk { heading_path: "T > Lex".into(), content: "T > Lex\n\ntrigram lexical matching".into() };
        store.insert_chunk(id, &c, &m, None).await.unwrap();

        let (hits, mode) = store
            .search(&TechQuery { text: "trigram lexical", category: None, language: None, limit: 5 })
            .await
            .unwrap();
        assert!(matches!(mode, SearchMode::TrigramOnly)); // no embedder configured in test_cfg
        assert_eq!(hits.len(), 1);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `MX_RAG_TEST_DATABASE_URL=postgres://postgres@localhost:55433/mx_rag cargo test -p mx-lib corpus::store`
Expected: FAIL (compile error — `search_with_embedding`, `TechQuery`, etc. missing).

- [ ] **Step 3: Implement**

Add to `store.rs` (inside `impl CorpusStore` plus the new types):

```rust
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
        let model_mismatch = !stored_models.is_empty()
            && stored_models.iter().any(|m| m != &self.model);
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
                sqlx::query("UPDATE technique_chunks SET embedding = $1, embedding_model = $2 WHERE id = $3")
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
```

In `crates/mx-lib/src/corpus/mod.rs`, ensure the re-export line reads:

```rust
pub use config::RagConfig;
pub use store::{BackendKind, CorpusStore, DocMeta, SearchMode, TechHit, TechQuery};
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `MX_RAG_TEST_DATABASE_URL=postgres://postgres@localhost:55433/mx_rag cargo test -p mx-lib corpus::store`
Expected: all pass (4 tests total in this module now).

- [ ] **Step 5: Commit**

```bash
git add crates/mx-lib/src/corpus/store.rs crates/mx-lib/src/corpus/mod.rs
git commit -m "feat(corpus): hybrid search with filters, trigram-only fallback, status, reembed"
```

---

### Task 8: Ingestion pipeline

**Acceptance Criteria (observable):**
- `cargo test -p mx-lib corpus::ingest` exits 0 covering: `scan_dir` parses frontmatter into DocMeta, applies heuristics for docs without frontmatter (warning recorded), skips `INDEX.md`, and chunks bodies; with `MX_RAG_TEST_DATABASE_URL` set, `ingest` writes docs+chunks, skips unchanged docs on re-run, and replaces chunks when a file changes.

**Verify via:** cli

**Files:**
- Modify: `crates/mx-lib/src/corpus/ingest.rs`

**Interfaces:**
- Consumes: `parse_frontmatter` (Task 2), `chunk_markdown`/`DEFAULT_CHUNK_CHARS` (Task 3), `CorpusStore`/`DocMeta`/`sha256_hex` (Task 6).
- Produces:
  ```rust
  pub struct ParsedDoc { pub path: std::path::PathBuf, pub rel_path: String, pub sha256: String,
                         pub meta: DocMeta, pub chunks: Vec<Chunk> }
  pub struct IngestSummary { pub docs_seen: usize, pub docs_skipped: usize, pub docs_ingested: usize,
                             pub chunks_seen: usize, pub chunks_new: usize, pub warnings: Vec<String> }
  pub fn scan_dir(dir: &std::path::Path) -> anyhow::Result<(Vec<ParsedDoc>, Vec<String>)>; // (docs, warnings)
  pub struct IngestOptions { pub clear: bool, pub force: bool }
  pub async fn ingest(store: &CorpusStore, docs: &[ParsedDoc], opts: &IngestOptions)
      -> anyhow::Result<IngestSummary>;
  ```

- [ ] **Step 1: Write the failing tests**

Append to `crates/mx-lib/src/corpus/ingest.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_fixture(dir: &std::path::Path) {
        fs::write(
            dir.join("with-fm.md"),
            "---\ntitle: FM Doc\ncategory: concurrency\nlanguages: [rust]\ncomplexity: expert\nuse_cases: [locks]\nsummary: s\n---\n\n# FM Doc\n\nintro\n\n## Alpha\n\nbody a",
        )
        .unwrap();
        fs::write(dir.join("no-fm.md"), "# Plain Doc\n\n## Beta\n\nbody b").unwrap();
        fs::write(dir.join("INDEX.md"), "# Index\n\nrouting doc").unwrap();
    }

    #[test]
    fn scan_dir_parses_heuristics_and_skips_index() {
        let dir = tempfile::tempdir().unwrap();
        write_fixture(dir.path());
        let (docs, warnings) = scan_dir(dir.path()).unwrap();
        assert_eq!(docs.len(), 2); // INDEX.md skipped

        let fm = docs.iter().find(|d| d.rel_path == "with-fm.md").unwrap();
        assert_eq!(fm.meta.title, "FM Doc");
        assert_eq!(fm.meta.category, "concurrency");
        assert_eq!(fm.meta.languages, vec!["rust"]);
        assert!(fm.chunks.iter().any(|c| c.heading_path == "FM Doc > Alpha"));

        let plain = docs.iter().find(|d| d.rel_path == "no-fm.md").unwrap();
        assert_eq!(plain.meta.title, "Plain Doc"); // heuristic: first # heading
        assert_eq!(plain.meta.category, "other");  // heuristic default
        assert!(warnings.iter().any(|w| w.contains("no-fm.md")));
    }

    #[tokio::test]
    async fn ingest_idempotent_and_replaces_changed() {
        let Some(url) = std::env::var("MX_RAG_TEST_DATABASE_URL").ok() else { return };
        let cfg = crate::corpus::RagConfig {
            database_url: None,
            fallback_database_url: url,
            embedding_api_key: None,
            ..crate::corpus::RagConfig::default()
        };
        let store = crate::corpus::CorpusStore::connect(&cfg).await.unwrap();
        store.clear().await.unwrap();

        let dir = tempfile::tempdir().unwrap();
        write_fixture(dir.path());
        let (docs, _) = scan_dir(dir.path()).unwrap();
        let s1 = ingest(&store, &docs, &IngestOptions { clear: false, force: false }).await.unwrap();
        assert_eq!(s1.docs_ingested, 2);
        assert!(s1.chunks_new > 0);

        // unchanged re-run: all skipped
        let (docs, _) = scan_dir(dir.path()).unwrap();
        let s2 = ingest(&store, &docs, &IngestOptions { clear: false, force: false }).await.unwrap();
        assert_eq!(s2.docs_skipped, 2);
        assert_eq!(s2.chunks_new, 0);

        // change one file: it re-ingests
        fs::write(dir.path().join("no-fm.md"), "# Plain Doc\n\n## Beta\n\nCHANGED body").unwrap();
        let (docs, _) = scan_dir(dir.path()).unwrap();
        let s3 = ingest(&store, &docs, &IngestOptions { clear: false, force: false }).await.unwrap();
        assert_eq!(s3.docs_ingested, 1);
        assert_eq!(s3.docs_skipped, 1);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p mx-lib corpus::ingest`
Expected: FAIL (compile error).

- [ ] **Step 3: Implement**

Write above the tests in `ingest.rs`:

```rust
//! Ingestion pipeline: walk markdown -> frontmatter -> chunk -> embed -> upsert.

use std::path::{Path, PathBuf};

use super::chunk::{chunk_markdown, Chunk, DEFAULT_CHUNK_CHARS};
use super::frontmatter::parse_frontmatter;
use super::store::{sha256_hex, CorpusStore, DocMeta};

/// A scanned, parsed, chunked doc — not yet written to the store.
pub struct ParsedDoc {
    pub path: PathBuf,
    pub rel_path: String,
    pub sha256: String,
    pub meta: DocMeta,
    pub chunks: Vec<Chunk>,
}

#[derive(Debug, Default)]
pub struct IngestSummary {
    pub docs_seen: usize,
    pub docs_skipped: usize,
    pub docs_ingested: usize,
    pub chunks_seen: usize,
    pub chunks_new: usize,
    pub warnings: Vec<String>,
}

pub struct IngestOptions {
    pub clear: bool,
    pub force: bool,
}

/// Heuristic category from a path when frontmatter lacks one (legacy port).
fn categorize_path(path: &Path) -> &'static str {
    let p = path.to_string_lossy().to_lowercase();
    if p.contains("docker") || p.contains("compose") {
        "docker"
    } else if p.contains("shell") {
        "shell"
    } else if p.contains("infra") || p.contains("deploy") {
        "infra"
    } else if p.contains("database") || p.contains("db-") {
        "database"
    } else {
        "other"
    }
}

/// Walk `dir` for `*.md` (recursive), skipping `INDEX.md`. Returns parsed docs
/// plus warnings for docs missing/failing frontmatter (heuristics applied).
pub fn scan_dir(dir: &Path) -> anyhow::Result<(Vec<ParsedDoc>, Vec<String>)> {
    let mut docs = Vec::new();
    let mut warnings = Vec::new();
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() || path.extension().map(|e| e != "md").unwrap_or(true) {
            continue;
        }
        if path.file_name().map(|n| n == "INDEX.md").unwrap_or(false) {
            continue;
        }
        let content = std::fs::read_to_string(path)?;
        let rel_path = path
            .strip_prefix(dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        let sha256 = sha256_hex(&content);
        let (fm, body) = parse_frontmatter(&content);
        if fm.is_none() {
            warnings.push(format!("{}: no valid frontmatter, using heuristics", rel_path));
        }
        let fm = fm.unwrap_or_default();
        let title = fm
            .title
            .or_else(|| {
                body.lines()
                    .find_map(|l| l.strip_prefix("# ").map(|t| t.trim().to_string()))
            })
            .unwrap_or_else(|| {
                path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()
            });
        let meta = DocMeta {
            path: rel_path.clone(),
            title: title.clone(),
            category: fm.category.unwrap_or_else(|| categorize_path(path).to_string()),
            languages: fm.languages,
            complexity: fm.complexity.unwrap_or_else(|| "intermediate".to_string()),
            use_cases: fm.use_cases,
            summary: fm.summary,
        };
        let chunks = chunk_markdown(&title, body, DEFAULT_CHUNK_CHARS);
        docs.push(ParsedDoc { path: path.to_path_buf(), rel_path, sha256, meta, chunks });
    }
    docs.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    Ok((docs, warnings))
}

/// Write parsed docs to the store. Unchanged docs (same sha) are skipped
/// unless `force`. Changed docs get their chunks deleted and re-inserted.
/// Chunks are batch-embedded when the store has an embedder.
pub async fn ingest(
    store: &CorpusStore,
    docs: &[ParsedDoc],
    opts: &IngestOptions,
) -> anyhow::Result<IngestSummary> {
    let mut summary = IngestSummary::default();
    if opts.clear {
        store.clear().await?;
    }
    for doc in docs {
        summary.docs_seen += 1;
        if !opts.force {
            if let Some(existing) = store.doc_sha(&doc.meta.path).await? {
                if existing == doc.sha256 {
                    summary.docs_skipped += 1;
                    continue;
                }
            }
        }
        let doc_id = store.upsert_doc(&doc.meta, &doc.sha256).await?;
        store.delete_doc_chunks(doc_id).await?;

        let texts: Vec<String> = doc.chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings: Vec<Option<Vec<f32>>> = match store.embedder() {
            Some(e) => e.embed_batch(&texts).await?.into_iter().map(Some).collect(),
            None => {
                if summary.docs_ingested == 0 {
                    summary
                        .warnings
                        .push("no embedding API key: chunks stored without embeddings (trigram-only search)".into());
                }
                vec![None; texts.len()]
            }
        };
        for (chunk, embedding) in doc.chunks.iter().zip(embeddings) {
            summary.chunks_seen += 1;
            if store.insert_chunk(doc_id, chunk, &doc.meta, embedding).await? {
                summary.chunks_new += 1;
            }
        }
        summary.docs_ingested += 1;
        tracing::info!("ingested {} ({} chunks)", doc.meta.path, doc.chunks.len());
    }
    Ok(summary)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `MX_RAG_TEST_DATABASE_URL=postgres://postgres@localhost:55433/mx_rag cargo test -p mx-lib corpus::ingest`
Expected: 2 passed. Also passes without the env var (DB test skips).

- [ ] **Step 5: Commit**

```bash
git add crates/mx-lib/src/corpus/ingest.rs
git commit -m "feat(corpus): idempotent frontmatter-aware ingestion pipeline"
```

---

### Task 9: `mx rag` CLI command

**Acceptance Criteria (observable):**
- `mx rag ingest --dry-run` (run from the mech-crate repo root, via `cargo run -p mx-cli -- rag ingest --dry-run`) exits 0 and prints a summary line containing docs and chunks counts plus any frontmatter warnings, WITHOUT needing a database or API key.
- `mx rag status` with the local test DB configured exits 0 and prints backend `local`, doc count, chunk count, and embedding model.
- `mx rag --help` exits 0 and lists `ingest` and `status` subcommands.

**Verify via:** cli

**Files:**
- Create: `crates/mx-cli/src/commands/rag.rs`
- Modify: `crates/mx-cli/src/commands/mod.rs` (add `pub mod rag;`)
- Modify: `crates/mx-cli/src/main.rs` (add `Rag` variant + match arm)

**Interfaces:**
- Consumes: `mx_lib::corpus::{RagConfig, CorpusStore, ingest::{scan_dir, ingest, IngestOptions}}`, `mx_lib::paths::mech_crate_root`.
- Produces: `RagCommand` clap Args struct with `pub async fn run(&self) -> anyhow::Result<()>`.

- [ ] **Step 1: Write the command**

Create `crates/mx-cli/src/commands/rag.rs`:

```rust
//! `mx rag` command - techniques corpus management (pgvector)

use anyhow::Result;
use clap::{Args, Subcommand};
use console::style;
use std::path::PathBuf;

use mx_lib::corpus::ingest::{ingest, scan_dir, IngestOptions};
use mx_lib::corpus::{CorpusStore, RagConfig};

/// Techniques corpus (RAG) management
#[derive(Args, Debug)]
pub struct RagCommand {
    #[command(subcommand)]
    command: RagSubcommand,
}

#[derive(Subcommand, Debug)]
enum RagSubcommand {
    /// Ingest technique docs into the corpus
    Ingest {
        /// Directory to ingest (default: <mech-crate root>/docs/development)
        #[arg(long)]
        path: Option<PathBuf>,
        /// Delete all existing docs/chunks first
        #[arg(long)]
        clear: bool,
        /// Re-ingest docs even if unchanged
        #[arg(long)]
        force: bool,
        /// Re-embed all chunks with the configured model (after provider switch)
        #[arg(long)]
        reembed: bool,
        /// Parse and chunk only; no database or embeddings needed
        #[arg(long)]
        dry_run: bool,
    },
    /// Show corpus status (backend, counts, model)
    Status,
}

impl RagCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            RagSubcommand::Ingest { path, clear, force, reembed, dry_run } => {
                self.ingest(path.clone(), *clear, *force, *reembed, *dry_run).await
            }
            RagSubcommand::Status => self.status().await,
        }
    }

    fn default_docs_dir() -> Result<PathBuf> {
        Ok(mx_lib::paths::mech_crate_root()?.join("docs/development"))
    }

    async fn ingest(&self, path: Option<PathBuf>, clear: bool, force: bool, reembed: bool, dry_run: bool) -> Result<()> {
        let dir = match path {
            Some(p) => p,
            None => Self::default_docs_dir()?,
        };
        println!("{} Scanning {}", style("→").cyan().bold(), dir.display());
        let (docs, warnings) = scan_dir(&dir)?;
        let total_chunks: usize = docs.iter().map(|d| d.chunks.len()).sum();
        for w in &warnings {
            println!("  {} {}", style("⚠").yellow(), w);
        }
        if dry_run {
            println!(
                "{} Dry run: {} docs, {} chunks, {} warnings",
                style("✓").green().bold(),
                docs.len(),
                total_chunks,
                warnings.len()
            );
            return Ok(());
        }

        let cfg = RagConfig::load();
        let store = CorpusStore::connect(&cfg).await?;
        println!("{} Connected to {} backend", style("→").cyan().bold(), store.backend().label());
        if !store.has_embedder() {
            println!(
                "  {} no embedding API key configured — chunks stored without embeddings (trigram-only search)",
                style("⚠").yellow()
            );
        }
        let summary = ingest(&store, &docs, &IngestOptions { clear, force }).await?;
        for w in &summary.warnings {
            println!("  {} {}", style("⚠").yellow(), w);
        }
        println!(
            "{} Ingested {} docs ({} skipped unchanged), {} new chunks",
            style("✓").green().bold(),
            summary.docs_ingested,
            summary.docs_skipped,
            summary.chunks_new
        );
        if reembed {
            println!("{} Re-embedding all chunks...", style("→").cyan().bold());
            let n = store.reembed_all().await?;
            println!("{} Re-embedded {} chunks", style("✓").green().bold(), n);
        }
        Ok(())
    }

    async fn status(&self) -> Result<()> {
        let cfg = RagConfig::load();
        let store = match CorpusStore::connect(&cfg).await {
            Ok(s) => s,
            Err(e) => {
                println!("{} Corpus offline: {}", style("✗").red().bold(), e);
                println!("  Start local pgvector: docker run -d --name mx-rag -p 5432:5432 -e POSTGRES_DB=mx_rag -e POSTGRES_HOST_AUTH_METHOD=trust pgvector/pgvector:pg17");
                return Ok(());
            }
        };
        let st = store.status().await?;
        println!("{}", style("Techniques Corpus Status").bold());
        println!("  {} Backend: {}", style("•").dim(), st["backend"].as_str().unwrap_or("?"));
        println!("  {} Docs: {}", style("•").dim(), st["docs"]);
        println!("  {} Chunks: {}", style("•").dim(), st["chunks"]);
        println!("  {} Embedding model: {}", style("•").dim(), st["embedding_model"].as_str().unwrap_or("?"));
        if st["model_mismatch"].as_bool() == Some(true) {
            println!(
                "  {} stored embeddings use {} — run `mx rag ingest --reembed` to re-embed with the configured model",
                style("⚠").yellow(),
                st["stored_models"]
            );
        }
        if let Some(t) = st["last_ingest"].as_str() {
            println!("  {} Last ingest: {}", style("•").dim(), t);
        }
        println!("  {} By category: {}", style("•").dim(), st["by_category"]);
        Ok(())
    }
}
```

- [ ] **Step 2: Wire into the CLI**

In `crates/mx-cli/src/commands/mod.rs` add `pub mod rag;` (alphabetical position, after `pub mod new;` / before `pub mod recipes;`).

In `crates/mx-cli/src/main.rs`:
- Add to the `Commands` enum (near the other management commands): `/// Techniques corpus (RAG) management` then `Rag(commands::rag::RagCommand),`
- Add to the match: `Commands::Rag(cmd) => cmd.run().await,`

- [ ] **Step 3: Verify dry-run and help**

```bash
cargo run -p mx-cli -- rag --help
cargo run -p mx-cli -- rag ingest --dry-run
```
Expected: help lists `ingest`/`status`; dry-run prints `Dry run: N docs, M chunks, W warnings` and exits 0 (docs/development currently has no frontmatter, so expect ~50 warnings — that's correct until Task 12).

- [ ] **Step 4: Verify status against the test DB**

```bash
MX_RAG_FALLBACK_DATABASE_URL=postgres://postgres@localhost:55433/mx_rag cargo run -p mx-cli -- rag status
```
Expected: prints `Backend: local` with counts.

- [ ] **Step 5: Commit**

```bash
git add crates/mx-cli/src/commands/rag.rs crates/mx-cli/src/commands/mod.rs crates/mx-cli/src/main.rs
git commit -m "feat(mx-cli): add mx rag ingest/status commands"
```

---

### Task 10: Replace Weaviate with corpus in mx-mcp-server

**Acceptance Criteria (observable):**
- `cargo build -p mx-mcp-server` exits 0 with NO `rag/`, `weaviate/` modules, no `mx-ingest` binary, and no `docker-compose.yml` in the crate.
- Piping an MCP `tools/list` request into `mx-mcp` returns JSON whose tool names include `rag_context` and `rag_health` (verify: `echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | cargo run -p mx-mcp-server --bin mx-mcp 2>/dev/null | grep -o 'rag_context'` prints `rag_context`).
- With the local test DB ingested (Task 8 fixtures or real corpus) a `tools/call` of `rag_health` returns text containing `"backend": "local"`; with no DB reachable it returns an offline message containing `rag.toml` — the server does not crash either way.
- `grep -ri weaviate crates/` returns no matches.

**Verify via:** cli

**Files:**
- Modify: `crates/mx-mcp-server/src/error.rs` (replace `Weaviate(String)` variant with `Corpus(String)`)
- Modify: `crates/mx-mcp-server/src/main.rs` (drop weaviate flags/wiring)
- Modify: `crates/mx-mcp-server/src/mcp/server.rs` (hold `Option<CorpusStore>`)
- Modify: `crates/mx-mcp-server/src/tools/mod.rs` (re-point 7 handlers, add `rag_context`, rewrite descriptions)
- Delete: `crates/mx-mcp-server/src/rag/mod.rs`, `crates/mx-mcp-server/src/weaviate/mod.rs`, `crates/mx-mcp-server/src/bin/ingest.rs`, `crates/mx-mcp-server/docker-compose.yml`
- Modify: `crates/mx-mcp-server/Cargo.toml` (remove `mx-ingest` bin section)

**Interfaces:**
- Consumes: `mx_lib::corpus::{RagConfig, CorpusStore, TechQuery, TechHit, SearchMode}`.
- Produces: `ToolRegistry::execute(..., corpus: Option<&CorpusStore>)` signature; new tool name `rag_context` with args `working_on` (required), `language`, `category`, `limit`.

- [ ] **Step 1: Update error.rs**

Replace the `Weaviate` variant:

```rust
    #[error("Corpus error: {0}")]
    Corpus(String),
```

- [ ] **Step 2: Rewrite main.rs**

Replace the Weaviate block of `main.rs` so it becomes:

```rust
//! MechCrate MCP Server

mod error;
mod mcp;
mod mx;
mod project;
mod tools;
mod unyform;

use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

use crate::mcp::server::McpServer;

#[derive(Parser, Debug)]
#[command(name = "mx-mcp")]
#[command(about = "MechCrate MCP Server - LLM-powered project management")]
#[command(version)]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// MechCrate root directory (auto-detected if not specified)
    #[arg(long, env = "MECH_CRATE_ROOT")]
    mech_crate_root: Option<String>,

    /// Skip the techniques corpus (RAG tools will report offline)
    #[arg(long)]
    no_rag: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let level = if args.debug { Level::DEBUG } else { Level::INFO };
    let filter = EnvFilter::from_default_env().add_directive(level.into());
    fmt().with_env_filter(filter).with_writer(std::io::stderr).init();

    info!("Starting MechCrate MCP Server v{}", env!("CARGO_PKG_VERSION"));

    let server = McpServer::new(args.mech_crate_root, args.no_rag)?;
    server.run().await?;

    Ok(())
}
```

(The `detect_mcp_dir` function and all weaviate imports are deleted.)

- [ ] **Step 3: Rewire server.rs**

- Change imports: remove `use crate::rag::WeaviateClient;`, add `use mx_lib::corpus::{CorpusStore, RagConfig};` and `use tracing::warn;` if not present.
- Change struct + constructor:

```rust
pub struct McpServer {
    mech_crate_root: PathBuf,
    no_rag: bool,
    tools: ToolRegistry,
}

impl McpServer {
    pub fn new(mech_crate_root: Option<String>, no_rag: bool) -> McpResult<Self> {
        let root = match mech_crate_root {
            Some(path) => PathBuf::from(path),
            None => Self::detect_mech_crate_root()?,
        };
        info!("MechCrate root: {:?}", root);
        Ok(Self { mech_crate_root: root, no_rag, tools: ToolRegistry::new() })
    }
```

- In `run()`, replace `let weaviate = WeaviateClient::new(&self.weaviate_url);` with:

```rust
        let corpus: Option<CorpusStore> = if self.no_rag {
            info!("RAG disabled (--no-rag)");
            None
        } else {
            match CorpusStore::connect(&RagConfig::load()).await {
                Ok(s) => {
                    info!("Techniques corpus connected ({} backend)", s.backend().label());
                    Some(s)
                }
                Err(e) => {
                    warn!("Techniques corpus unavailable: {e}. RAG tools will report offline.");
                    None
                }
            }
        };
```

- Thread `corpus.as_ref()` through `handle_tool_call` in place of `&weaviate` (update its signature to `corpus: Option<&CorpusStore>` and the `execute(...)` call accordingly).

- [ ] **Step 4: Re-point tools/mod.rs**

- Imports: remove `use crate::rag::{format_search_results, WeaviateClient};`, add `use mx_lib::corpus::{CorpusStore, SearchMode, TechHit, TechQuery};`.
- Change `execute` signature: `weaviate: &WeaviateClient` → `corpus: Option<&CorpusStore>`.
- Add at module scope:

```rust
const CORPUS_OFFLINE: &str = "Techniques corpus is offline: could not reach Neon or local Postgres.\n\
Configure ~/.mech-crate/config/rag.toml (database_url / fallback_database_url) or start a local pgvector:\n\
  docker run -d --name mx-rag -p 5432:5432 -e POSTGRES_DB=mx_rag -e POSTGRES_HOST_AUTH_METHOD=trust pgvector/pgvector:pg17\n\
Then run: mx rag ingest";

/// Render hits grouped by source doc, with metadata and scores.
fn format_hits(header: &str, hits: &[TechHit], mode: SearchMode) -> String {
    if hits.is_empty() {
        return format!("{header}\n\nNo relevant techniques found. Try different phrasing, or check `rag_health` and `mx rag ingest`.");
    }
    let mut out = format!("{header}\n");
    if matches!(mode, SearchMode::TrigramOnly) {
        out.push_str("\n> Note: lexical-only search (no embedding key configured); results may be weaker.\n");
    }
    let mut by_doc: Vec<(&str, Vec<&TechHit>)> = Vec::new();
    for h in hits {
        match by_doc.iter_mut().find(|(p, _)| *p == h.path.as_str()) {
            Some((_, v)) => v.push(h),
            None => by_doc.push((h.path.as_str(), vec![h])),
        }
    }
    for (path, doc_hits) in by_doc {
        let first = doc_hits[0];
        out.push_str(&format!("\n## {} ({})\n", first.title, path));
        out.push_str(&format!(
            "category: {} | languages: {} | use cases: {}\n",
            first.category,
            first.languages.join(", "),
            first.use_cases.join("; ")
        ));
        if let Some(s) = &first.summary {
            out.push_str(&format!("> {}\n", s));
        }
        for h in doc_hits {
            out.push_str(&format!("\n### {} (score {:.2})\n\n{}\n", h.heading_path, h.score, h.content));
        }
        out.push_str("\n---\n");
    }
    out
}
```

- Add `RagContext` to the `ToolHandler` enum and this definition to `define_all_tools()` (place it FIRST in the RAG block):

```rust
            ToolDefinition {
                tool: Tool {
                    name: "rag_context".to_string(),
                    description: r#"Get development techniques relevant to what you are working on RIGHT NOW.

This is the primary entry point to the techniques corpus (theory, patterns,
architecture, concurrency, API design, databases, Docker, FSM/FRP, and more,
drawn from mech-crate docs/development). Describe the task in 1-2 sentences
and get back the most relevant technique chunks, grouped by source document.

Examples:
- working_on: "designing a retry/backoff strategy for an async Rust job queue", language: "rust"
- working_on: "structuring a Laravel service layer so business logic stays testable", language: "php"
- working_on: "choosing between embedding and referencing for MongoDB order documents""#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "working_on": {
                                "type": "string",
                                "description": "1-2 sentence description of the current task/problem"
                            },
                            "language": {
                                "type": "string",
                                "description": "Optional language filter (e.g. rust, typescript, php, python)"
                            },
                            "category": {
                                "type": "string",
                                "description": "Optional category filter (e.g. theory, patterns, architecture, concurrency, api-design, database, docker, shell, blockchain, ml, security, process, frontend, infra)"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum chunks to return (default: 5)"
                            }
                        })),
                        required: Some(vec!["working_on".to_string()]),
                    },
                },
                handler: ToolHandler::RagContext,
            },
```

- Rewrite the 7 existing rag tool descriptions to be technique-oriented (same names/args). Exact replacement descriptions:
  - `rag_search`: `"Semantic + lexical hybrid search over the development-techniques corpus (theory, patterns, architecture, concurrency, API design, databases, Docker, and more). Prefer rag_context when you can describe your current task."`
  - `rag_search_category`: `"Search the techniques corpus within one category. Categories: theory, patterns, architecture, concurrency, api-design, database, frontend, docker, infra, shell, blockchain, ml, security, process, other."` (also update the `category` property description to this category list)
  - `rag_find_implementation`: `"Find code-bearing technique content for a pattern in a given language (filters on the corpus languages metadata; e.g. 'lens/prism optics' + language 'typescript')."`
  - `rag_get_guidance`: `"Get architecture/design guidance from the techniques corpus for a specific problem, optionally with constraints (e.g. 'must be lock-free', 'PHP 8')."`
  - `rag_compare_approaches`: `"Compare two or more approaches/technologies using the techniques corpus (e.g. ['mutex', 'atomics'] or ['embedding documents', 'referencing documents'])."`
  - `rag_find_related`: `"Find techniques related to a topic or document, excluding the topic's own doc — useful to expand context around a chosen approach."`
  - `rag_health`: `"Check techniques corpus availability: backend (neon/local/offline), doc/chunk counts, embedding model, last ingest time."`
- Replace the 7 handler arms + add the new one. Every arm starts with the same guard; complete replacement code:

```rust
            ToolHandler::RagContext => {
                let Some(corpus) = corpus else { return Ok(ToolCallResult::text(CORPUS_OFFLINE.to_string())) };
                let working_on = args.get("working_on").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'working_on' is required".to_string())
                })?;
                let language = args.get("language").and_then(|v| v.as_str());
                let category = args.get("category").and_then(|v| v.as_str());
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as i64;
                match corpus.search(&TechQuery { text: working_on, category, language, limit }).await {
                    Ok((hits, mode)) => Ok(ToolCallResult::text(format_hits(
                        &format!("# Techniques for: {}", working_on), &hits, mode))),
                    Err(e) => Ok(ToolCallResult::text(format!("Corpus search failed: {e}"))),
                }
            }

            ToolHandler::RagSearch => {
                let Some(corpus) = corpus else { return Ok(ToolCallResult::text(CORPUS_OFFLINE.to_string())) };
                let query = args.get("query").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'query' is required".to_string())
                })?;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as i64;
                match corpus.search(&TechQuery { text: query, category: None, language: None, limit }).await {
                    Ok((hits, mode)) => Ok(ToolCallResult::text(format_hits(
                        &format!("# Results for: {}", query), &hits, mode))),
                    Err(e) => Ok(ToolCallResult::text(format!("Corpus search failed: {e}"))),
                }
            }

            ToolHandler::RagSearchCategory => {
                let Some(corpus) = corpus else { return Ok(ToolCallResult::text(CORPUS_OFFLINE.to_string())) };
                let query = args.get("query").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'query' is required".to_string())
                })?;
                let category = args.get("category").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'category' is required".to_string())
                })?;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as i64;
                match corpus.search(&TechQuery { text: query, category: Some(category), language: None, limit }).await {
                    Ok((hits, mode)) => Ok(ToolCallResult::text(format_hits(
                        &format!("# [{}] results for: {}", category, query), &hits, mode))),
                    Err(e) => Ok(ToolCallResult::text(format!("Corpus search failed: {e}"))),
                }
            }

            ToolHandler::RagFindImplementation => {
                let Some(corpus) = corpus else { return Ok(ToolCallResult::text(CORPUS_OFFLINE.to_string())) };
                let pattern = args.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'pattern' is required".to_string())
                })?;
                let language = args.get("language").and_then(|v| v.as_str());
                let query = format!("code implementation example {}", pattern);
                match corpus.search(&TechQuery { text: &query, category: None, language, limit: 8 }).await {
                    Ok((hits, mode)) => {
                        let code_hits: Vec<TechHit> = hits.iter().filter(|h| h.content.contains("```")).cloned().collect();
                        let chosen = if code_hits.is_empty() { hits } else { code_hits };
                        let chosen: Vec<TechHit> = chosen.into_iter().take(5).collect();
                        Ok(ToolCallResult::text(format_hits(
                            &format!("# Implementations: {}", pattern), &chosen, mode)))
                    }
                    Err(e) => Ok(ToolCallResult::text(format!("Corpus search failed: {e}"))),
                }
            }

            ToolHandler::RagGetGuidance => {
                let Some(corpus) = corpus else { return Ok(ToolCallResult::text(CORPUS_OFFLINE.to_string())) };
                let problem = args.get("problem").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'problem' is required".to_string())
                })?;
                let constraints: Vec<String> = args
                    .get("constraints")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                let query = if constraints.is_empty() {
                    format!("architecture design pattern best practice {}", problem)
                } else {
                    format!("architecture design pattern {} constraints: {}", problem, constraints.join(", "))
                };
                let mut header = format!("# Guidance for: {}", problem);
                if !constraints.is_empty() {
                    header.push_str(&format!("\n**Constraints:** {}", constraints.join(", ")));
                }
                match corpus.search(&TechQuery { text: &query, category: None, language: None, limit: 7 }).await {
                    Ok((hits, mode)) => Ok(ToolCallResult::text(format_hits(&header, &hits, mode))),
                    Err(e) => Ok(ToolCallResult::text(format!("Corpus search failed: {e}"))),
                }
            }

            ToolHandler::RagCompareApproaches => {
                let Some(corpus) = corpus else { return Ok(ToolCallResult::text(CORPUS_OFFLINE.to_string())) };
                let approaches: Vec<String> = args
                    .get("approaches")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .ok_or_else(|| McpError::InvalidArguments("'approaches' is required".to_string()))?;
                let criteria: Vec<String> = args
                    .get("criteria")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                let mut result = format!("# Comparison: {}\n", approaches.join(" vs "));
                if !criteria.is_empty() {
                    result.push_str(&format!("**Focus:** {}\n", criteria.join(", ")));
                }
                for approach in &approaches {
                    let query = if criteria.is_empty() {
                        format!("{} technique tradeoffs usage", approach)
                    } else {
                        format!("{} {}", approach, criteria.join(" "))
                    };
                    match corpus.search(&TechQuery { text: &query, category: None, language: None, limit: 3 }).await {
                        Ok((hits, mode)) => {
                            result.push_str(&format_hits(&format!("\n## {}", approach), &hits, mode));
                        }
                        Err(e) => result.push_str(&format!("\n## {}\n\nSearch error: {e}\n", approach)),
                    }
                }
                Ok(ToolCallResult::text(result))
            }

            ToolHandler::RagFindRelated => {
                let Some(corpus) = corpus else { return Ok(ToolCallResult::text(CORPUS_OFFLINE.to_string())) };
                let topic = args.get("topic").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'topic' is required".to_string())
                })?;
                let max_results = args.get("max_results").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
                let query = topic.replace(".md", "").replace(['-', '_'], " ");
                match corpus.search(&TechQuery { text: &query, category: None, language: None, limit: (max_results + 5) as i64 }).await {
                    Ok((hits, mode)) => {
                        let filtered: Vec<TechHit> = hits
                            .into_iter()
                            .filter(|h| !h.path.contains(topic) && !h.title.contains(topic))
                            .take(max_results)
                            .collect();
                        Ok(ToolCallResult::text(format_hits(
                            &format!("# Related to: {}", topic), &filtered, mode)))
                    }
                    Err(e) => Ok(ToolCallResult::text(format!("Corpus search failed: {e}"))),
                }
            }

            ToolHandler::RagHealth => {
                let Some(corpus) = corpus else { return Ok(ToolCallResult::text(CORPUS_OFFLINE.to_string())) };
                match corpus.status().await {
                    Ok(st) => Ok(ToolCallResult::text(serde_json::to_string_pretty(&st).unwrap_or_default())),
                    Err(e) => Ok(ToolCallResult::text(format!("Corpus status failed: {e}"))),
                }
            }
```

- [ ] **Step 5: Delete Weaviate artifacts**

```bash
git rm crates/mx-mcp-server/src/rag/mod.rs crates/mx-mcp-server/src/weaviate/mod.rs crates/mx-mcp-server/src/bin/ingest.rs crates/mx-mcp-server/docker-compose.yml
```
Remove `mod rag;` / `mod weaviate;` from `main.rs` (done in Step 2), remove the `[[bin]] mx-ingest` section from `crates/mx-mcp-server/Cargo.toml`, and add `mx-lib = { path = "../mx-lib" }` if not already a dependency (it is — verify).

- [ ] **Step 6: Build, smoke-test, verify no weaviate references**

```bash
cargo build -p mx-mcp-server
printf '%s\n%s\n' \
  '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"smoke","version":"0"}}}' \
  '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' \
  | ./target/debug/mx-mcp 2>/dev/null | grep -o '"rag_context"' | head -1
printf '%s\n%s\n' \
  '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"smoke","version":"0"}}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"rag_health","arguments":{}}}' \
  | MX_RAG_FALLBACK_DATABASE_URL=postgres://postgres@localhost:55433/mx_rag ./target/debug/mx-mcp 2>/dev/null | grep -o '\\"backend\\": \\"local\\"'
grep -ri weaviate crates/ || echo "CLEAN"
```
Expected: `"rag_context"`, the backend-local match, `CLEAN`. (The rag_health text is JSON embedded in the MCP result string, hence the escaped quotes in the final grep.)

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(mx-mcp): replace Weaviate with pgvector corpus; add rag_context tool"
```

---

### Task 11: Clean up `mx mcp` command and McpManager

**Acceptance Criteria (observable):**
- `mx mcp --help` (via `cargo run -p mx-cli -- mcp --help`) exits 0 and no longer lists `start`, `up`, `stop`, `down`, `logs`, or `ingest` subcommands; `build`, `status`, `config`, `run`, `info`, `test` remain.
- `mx mcp status` prints corpus status (backend/counts) or the offline hint, with no Weaviate/docker-compose references.
- `grep -ri weaviate crates/` returns no matches; `cargo build --workspace` exits 0.

**Verify via:** cli

**Files:**
- Modify: `crates/mx-cli/src/commands/mcp.rs`
- Modify: `crates/mx-lib/src/mcp/mod.rs`

**Interfaces:**
- Consumes: `mx_lib::corpus::{RagConfig, CorpusStore}` for the new `status`.

- [ ] **Step 1: Trim the subcommand enum**

In `crates/mx-cli/src/commands/mcp.rs` remove the `Start`, `Up`, `Stop`, `Down`, `Logs`, and `Ingest` variants (and their match arms + private methods `start`, `stop`, `logs`, `ingest`). Resulting enum:

```rust
#[derive(Subcommand, Debug)]
enum McpSubcommand {
    /// Build MCP server binaries
    Build,
    /// Show status (corpus backend + counts)
    Status,
    /// Alias for status
    Ps,
    /// Show MCP client configuration
    Config,
    /// Run MCP server interactively
    Run,
    /// Show MCP info
    Info,
    /// Test MCP server
    Test,
}
```

- [ ] **Step 2: Rewrite status to use the corpus**

Replace the `status` method body with:

```rust
    async fn status(&self, _mcp: &McpManager) -> Result<()> {
        let cfg = mx_lib::corpus::RagConfig::load();
        match mx_lib::corpus::CorpusStore::connect(&cfg).await {
            Ok(store) => {
                let st = store.status().await?;
                println!("{}", style("Techniques Corpus").bold());
                println!("  {} Backend: {}", style("•").dim(), st["backend"].as_str().unwrap_or("?"));
                println!("  {} Docs: {} / Chunks: {}", style("•").dim(), st["docs"], st["chunks"]);
                println!("  {} Model: {}", style("•").dim(), st["embedding_model"].as_str().unwrap_or("?"));
            }
            Err(e) => {
                println!("{} Corpus offline: {}", style("✗").red().bold(), e);
                println!("  Configure ~/.mech-crate/config/rag.toml or start local pgvector (see mx rag status).");
            }
        }
        Ok(())
    }
```

Update `info`/`config`/`test` bodies only as needed to drop Weaviate fields (see Step 3) — keep their MCP-server behavior.

- [ ] **Step 3: Purge McpManager weaviate helpers**

In `crates/mx-lib/src/mcp/mod.rs` delete: `ingest_binary`, `allocate_ports`, `http_port`, `weaviate_url`, `is_weaviate_running`, `start_weaviate`, `stop_weaviate`, `weaviate_logs`, `weaviate_status`, `ingest`, and any port-file constants/helpers they used. Update `McpInfo` (and its construction in `info()` / use in `generate_config()`) to drop `weaviate_url`/`weaviate_running` fields. Keep `state_dir`, `source_dir`, `mcp_binary`, `needs_build`, `build`, `ensure_binary`, `info`, `generate_config`.

- [ ] **Step 4: Verify**

```bash
cargo build --workspace
cargo run -p mx-cli -- mcp --help
MX_RAG_FALLBACK_DATABASE_URL=postgres://postgres@localhost:55433/mx_rag cargo run -p mx-cli -- mcp status
grep -ri weaviate crates/ || echo "CLEAN"
```
Expected: help shows trimmed list; status prints corpus info; `CLEAN`.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "refactor(mx-cli): drop weaviate lifecycle from mx mcp; corpus-backed status"
```

---

### Task 12: Frontmatter authoring pass over docs/development

**Acceptance Criteria (observable):**
- `cargo run -p mx-cli -- rag ingest --dry-run` exits 0 and prints `0 warnings` (every non-INDEX doc has valid frontmatter).
- `git diff --stat` shows only `docs/development/*.md` modified, each gaining a frontmatter block at byte 0.
- `docs/development/INDEX.md` gains a short "Frontmatter authoring" section documenting the schema and the category taxonomy.

**Verify via:** cli

**Files:**
- Modify: every `docs/development/*.md` except `INDEX.md`
- Modify: `docs/development/INDEX.md` (authoring guide section)

**Interfaces:**
- Consumes: frontmatter schema from Task 2 (`title`, `category`, `languages`, `complexity`, `use_cases`, `summary`).

- [ ] **Step 1: Add frontmatter to each doc**

For each file, insert at byte 0 a block following this template, then a blank line, then the original content:

```yaml
---
title: <from the doc's own H1, or INDEX.md entry>
category: <from table below>
languages: [<from table below>]
complexity: <from table below>
use_cases:
  - <2-4 concrete "when to reach for this" phrases, from the doc's intro or INDEX.md "Best Use Cases">
summary: <one sentence, from the doc's stated purpose>
---
```

Assignments (title/use_cases/summary come from each doc's own intro + INDEX.md; these three columns are fixed):

| File | category | languages | complexity |
|---|---|---|---|
| appendix-actor-model.md | concurrency | rust, typescript | advanced |
| appendix-algebraic-effects-optics.md | theory | typescript, rust | expert |
| appendix-api-design.md | api-design | | advanced |
| appendix-astro-scaffold.md | frontend | typescript | intermediate |
| appendix-build-deploy-recipe.md | infra | | intermediate |
| appendix-business-logic-placement.md | architecture | | advanced |
| appendix-category-theory-php.md | theory | php | advanced |
| appendix-category-theory-rust.md | theory | rust | advanced |
| appendix-category-theory-typescript.md | theory | typescript | advanced |
| appendix-category-theory.md | theory | haskell | expert |
| appendix-concurrency-time.md | concurrency | | advanced |
| appendix-consensus.md | architecture | | expert |
| appendix-consistency-models.md | architecture | | expert |
| appendix-coordination-models.md | architecture | | expert |
| appendix-dataflow.md | patterns | | advanced |
| appendix-emergence.md | theory | | advanced |
| appendix-file-structure-theory.md | architecture | | intermediate |
| appendix-formal-verification.md | theory | | research |
| appendix-frp-general.md | patterns | | advanced |
| appendix-frp-js-ts.md | patterns | javascript, typescript | advanced |
| appendix-frp-php.md | patterns | php | advanced |
| appendix-frp-rust.md | patterns | rust | advanced |
| appendix-fsm.md | patterns | | advanced |
| appendix-groundbreaking-patterns.md | patterns | rust, typescript | expert |
| appendix-laravel.md | patterns | php | intermediate |
| appendix-memory-models.md | concurrency | | expert |
| appendix-novel-theories.md | theory | rust, typescript | research |
| appendix-pattern-playbook.md | patterns | | intermediate |
| appendix-process-calculi.md | theory | | research |
| appendix-rag.md | ml | python | advanced |
| appendix-rust-concurrency.md | concurrency | rust | expert |
| appendix-rust.md | patterns | rust | advanced |
| appendix-shell-scripting.md | shell | bash | intermediate |
| appendix-software-general-biology.md | theory | | intermediate |
| appendix-solana-rpc.md | blockchain | rust, typescript | advanced |
| appendix-streams.md | patterns | | advanced |
| appendix-theory-map.md | theory | | intermediate |
| appendix-type-theory.md | theory | | expert |
| APPLE_DESIGN_GUIDELINES.md | frontend | | intermediate |
| APPLE_DESIGN_QUICK_GUIDE.md | frontend | | intermediate |
| BUILD_DEPLOY_RECIPE.md | infra | | intermediate |
| database-design-guide.md | database | sql, redis, javascript, python | advanced |
| docker-assembly-guide.md | docker | | intermediate |
| ghostnet-efficient-cnn-guide.md | ml | python | advanced |
| INFRA_CONFIG.md | infra | | intermediate |
| instructions.md | process | | intermediate |
| MX_QUICK_REFERENCE.md | process | | intermediate |
| MX_RUST_CLI_AND_MCP_SERVER.md | architecture | rust | advanced |
| mx-mcp~usage.md | process | | intermediate |
| QUICK_REFERENCE.md | process | | intermediate |
| RECIPE_AUTHORING_GUIDE.md | process | | intermediate |
| RUST_CLI_DEVELOPMENT.md | patterns | rust | advanced |
| rwa-blockchain-guide.md | blockchain | solidity, typescript, python | advanced |
| sec-compliance-framework.md | security | | advanced |
| SHELL_SCRIPTING_GUIDE.md | shell | bash | intermediate |
| sources.md | process | | intermediate |

(If a file listed here no longer exists or new files appeared, apply best-fit category from the taxonomy and note it in the commit message.)

- [ ] **Step 2: Add the authoring guide to INDEX.md**

Append to `docs/development/INDEX.md`:

```markdown
## Frontmatter Authoring (for the techniques corpus)

Every doc in this folder (except INDEX.md) carries YAML frontmatter consumed by `mx rag ingest`:

​```yaml
---
title: Human-readable title
category: one of: theory | patterns | architecture | concurrency | api-design | database | frontend | docker | infra | shell | blockchain | ml | security | process | other
languages: [rust, typescript]        # omit or [] when language-agnostic
complexity: intermediate | advanced | expert | research
use_cases:
  - short "reach for this when..." phrases (2-4)
summary: One sentence describing the doc.
---
​```

Docs without frontmatter still ingest (heuristics + a warning), but filtered retrieval quality drops. Re-ingest after edits: `mx rag ingest`.
```

(Remove the zero-width escapes around the fences when writing the actual file.)

- [ ] **Step 3: Verify zero warnings**

Run: `cargo run -p mx-cli -- rag ingest --dry-run`
Expected: `Dry run: 56 docs, <N> chunks, 0 warnings` (doc count = current file count minus INDEX.md).

- [ ] **Step 4: Commit**

```bash
git add docs/development
git commit -m "docs(development): add technique frontmatter to all docs + authoring guide"
```

---

### Task 13: Neon provisioning, config, live ingest

**Acceptance Criteria (observable):**
- `~/.mech-crate/config/rag.toml` exists with the Neon `database_url` (project `mech-crate`, org PriceLove LLC) and local fallback.
- `mx rag ingest` (release or `cargo run -p mx-cli -- rag ingest`) against Neon exits 0 reporting ~56 docs ingested with embeddings (requires `OPENAI_API_KEY`).
- `mx rag status` prints `Backend: neon`, docs ≥ 50, chunks > 500, model `text-embedding-3-small`.
- An MCP `tools/call` of `rag_context` with `working_on: "choosing between mutex and atomics for a hot counter in rust"` returns text mentioning `appendix-rust-concurrency` (canonical relevance probe).

**Verify via:** cli

**Files:**
- Create: `~/.mech-crate/config/rag.toml` (user machine, not committed)

**Interfaces:**
- Consumes: everything prior.

- [ ] **Step 1: Provision Neon**

Create the Neon project via the Neon MCP (org `org-snowy-credit-95327987`, name `mech-crate`, region `aws-us-west-2`, PG 17) or `neonctl projects create --name mech-crate --org-id org-snowy-credit-95327987`. Capture the pooled connection string for the default database.

- [ ] **Step 2: Write rag.toml**

```bash
mkdir -p ~/.mech-crate/config
cat > ~/.mech-crate/config/rag.toml <<'EOF'
database_url = "<NEON_CONNECTION_STRING>"
fallback_database_url = "postgres://postgres@localhost:5432/mx_rag"
# embedding_base_url / embedding_model use defaults (OpenAI text-embedding-3-small)
# API key comes from OPENAI_API_KEY env
EOF
```

- [ ] **Step 3: Live ingest and status**

```bash
cargo run -p mx-cli -- rag ingest
cargo run -p mx-cli -- rag status
```
Expected: ingest reports ~56 docs; status shows `Backend: neon` and the counts.

- [ ] **Step 4: Relevance probe through the MCP server**

```bash
cargo build -p mx-mcp-server
printf '%s\n%s\n' \
  '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"smoke","version":"0"}}}' \
  '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"rag_context","arguments":{"working_on":"choosing between mutex and atomics for a hot counter in rust","language":"rust"}}}' \
  | ./target/debug/mx-mcp 2>/dev/null | grep -o 'appendix-rust-concurrency' | head -1
```
Expected: `appendix-rust-concurrency`.

- [ ] **Step 5: Commit (docs-only if anything changed in-repo)**

No repo files change in this task besides possibly notes; if none: `git commit --allow-empty -m "chore(corpus): neon provisioned and live corpus ingested"`.

---

### Task 14: `techniques` skill

**Acceptance Criteria (observable):**
- `~/.claude/skills/techniques/SKILL.md` exists with valid frontmatter (`name: techniques`, a trigger-rich `description`) and body teaching the rag_context-first loop, drill-down tools, advisory application, and offline fallback.
- `cat ~/.claude/skills/techniques/SKILL.md | head -5` shows the frontmatter.

**Verify via:** cli

**Files:**
- Create: `~/.claude/skills/techniques/SKILL.md`

- [ ] **Step 1: Write the skill**

```markdown
---
name: techniques
description: 'Consult the mx techniques corpus (RAG over mech-crate docs/development: theory, patterns, architecture, concurrency, API design, databases, Docker, FSM/FRP, security) when deciding HOW to implement something. Use when choosing between approaches/architectures/patterns, starting feature work in a covered domain, writing an implementation plan, or the user asks "what is the best way to build X". Triggers: /techniques, "check the techniques library", "what patterns apply".'
---

# Techniques — Corpus-Backed Building Patterns

Query the mx MCP techniques corpus before committing to an implementation approach. The corpus holds curated engineering technique docs (mech-crate `docs/development`), chunked and searchable by meaning + metadata.

**Announce at start:** "Consulting the techniques corpus."

## Core loop

1. **Describe the work.** Call `mcp__mx__rag_context` with `working_on` = 1–2 sentences about the current task. Add `language` (rust/typescript/php/python/...) and `category` (theory | patterns | architecture | concurrency | api-design | database | frontend | docker | infra | shell | blockchain | ml | security | process) when obvious. Keep `limit` at 5 or less.
2. **Drill down only if deciding.** Choosing between two approaches → `mcp__mx__rag_compare_approaches`. Need code shape for a chosen pattern in a language → `mcp__mx__rag_find_implementation`. Expanding around a chosen doc → `mcp__mx__rag_find_related`.
3. **Apply as advisory, not gospel.** Returned techniques are patterns, not requirements. Adopt what fits the codebase's existing conventions; skip what doesn't. When a technique shapes a plan or PR, cite its source doc path (e.g. `docs/development/appendix-rust-concurrency.md`).
4. **Never block on the corpus.** If tools return the offline message or nothing relevant: note it in one line and proceed with your own judgment. `mcp__mx__rag_health` diagnoses; `mx rag ingest` re-ingests; do NOT stop work to repair the corpus unless the user asks.

## When to consult

- Writing or reviewing an implementation plan (writing-devloop-plans does this automatically)
- Starting a task in a covered domain (concurrency, DB schema, API design, Docker builds, FSM/FRP state, functional patterns)
- Torn between two designs — get the corpus's tradeoff framing before deciding
- The user explicitly asks for "the right way" / "best practice" / "what pattern"

## When NOT to consult

- Trivial mechanical edits (rename, typo, version bump)
- Domains the corpus doesn't cover — check `rag_health` by_category if unsure
- Re-querying the same question in the same session — reuse what you got
```

- [ ] **Step 2: Verify and commit**

```bash
head -5 ~/.claude/skills/techniques/SKILL.md
```
Expected: frontmatter with `name: techniques`. (Skill lives outside the repo; also snapshot a copy into the repo for versioning:)

```bash
mkdir -p skills/techniques && cp ~/.claude/skills/techniques/SKILL.md skills/techniques/SKILL.md
git add skills/techniques/SKILL.md
git commit -m "feat(skills): add techniques skill (corpus consultation loop)"
```

---

### Task 15: Augment writing-devloop-plans skill

**Acceptance Criteria (observable):**
- `~/.claude/skills/writing-devloop-plans/SKILL.md` contains a "Consult the techniques corpus" instruction inside Process step 1, including the `**Apply:** <doc> — <technique>` line convention and the never-block rule.
- `grep -c "rag_context" ~/.claude/skills/writing-devloop-plans/SKILL.md` prints ≥ 1.

**Verify via:** cli

**Files:**
- Modify: `~/.claude/skills/writing-devloop-plans/SKILL.md`

- [ ] **Step 1: Insert the consultation step**

In the `## Process` section, directly after the paragraph `1. **Invoke `superpowers:writing-plans` to produce the base plan.** ...`, insert:

```markdown
   **Consult the techniques corpus while writing the base plan.** Before drafting tasks, call `mcp__mx__rag_context` with a 1–2 sentence description of the feature and its stack (plus `language` when obvious). Weave returned techniques into task design. When a task directly applies one, add an `**Apply:** <source doc path> — <technique>` line directly below that task's Acceptance Criteria so the executing subagent inherits the reference. If the corpus is offline (`mcp__mx__rag_health`) or returns nothing relevant, note it and continue — never block planning on it.
```

- [ ] **Step 2: Verify and snapshot**

```bash
grep -c "rag_context" ~/.claude/skills/writing-devloop-plans/SKILL.md
mkdir -p skills/writing-devloop-plans && cp ~/.claude/skills/writing-devloop-plans/SKILL.md skills/writing-devloop-plans/SKILL.md
git add skills/writing-devloop-plans/SKILL.md
git commit -m "feat(skills): writing-devloop-plans consults techniques corpus"
```

---

### Task 16: Augment devloop subagent prompt

**Acceptance Criteria (observable):**
- `~/.claude/skills/devloop/subagent-prompt.md` has a numbered "CONSULT TECHNIQUES" step between criteria derivation and code work, capped at ONE `rag_context` call with `limit: 3`, honoring `**Apply:**` lines, with an explicit never-block rule; subsequent step numbers are consistent (no duplicates/gaps).
- `grep -c "rag_context" ~/.claude/skills/devloop/subagent-prompt.md` prints ≥ 1.

**Verify via:** cli

**Files:**
- Modify: `~/.claude/skills/devloop/subagent-prompt.md`

- [ ] **Step 1: Insert the step**

In the `YOUR JOB:` list, after step `1. DERIVE ACCEPTANCE CRITERIA ...` and before the current step 2 (`Use the superpowers:executing-plans skill...`), insert:

```
2. CONSULT TECHNIQUES (one call, before any code work).
   - If the task block contains an "**Apply:**" line, call mcp__mx__rag_context with that technique as working_on (limit: 3).
   - Otherwise call mcp__mx__rag_context ONCE with working_on = the task title plus a one-line summary of the acceptance criteria (limit: 3).
   - Apply returned techniques only where they clearly fit THIS task; cite the source doc path in a code comment only when the pattern would otherwise be non-obvious.
   - If the tool is unavailable, offline, or returns nothing relevant: proceed without it. Never block on the corpus, never try to repair it.
```

Renumber the existing steps 2..N to 3..N+1 (update any in-text references to those step numbers in the same file).

- [ ] **Step 2: Verify and snapshot**

```bash
grep -c "rag_context" ~/.claude/skills/devloop/subagent-prompt.md
grep -n "^[0-9]\+\." ~/.claude/skills/devloop/subagent-prompt.md | head -12   # eyeball: sequential numbering
mkdir -p skills/devloop && cp ~/.claude/skills/devloop/subagent-prompt.md skills/devloop/subagent-prompt.md
git add skills/devloop/subagent-prompt.md
git commit -m "feat(skills): devloop subagents consult techniques corpus per task"
```

---

### Task 17: Docs sweep and final verification

**Acceptance Criteria (observable):**
- `grep -rli weaviate docs/development README.md Makefile make templates scripts 2>/dev/null` returns nothing (historical docs `docs/architecture-review-2026-03-07.md`, `docs/unyform/*`, and `docs/superpowers/*` exempt).
- `cargo build --workspace && cargo test --workspace` exits 0 (DB tests run when `MX_RAG_TEST_DATABASE_URL` is exported).
- `cargo run -p mx-cli -- rag ingest --dry-run` still reports 0 warnings after doc edits.

**Verify via:** cli

**Files:**
- Modify: `docs/development/MX_QUICK_REFERENCE.md`, `docs/development/mx-mcp~usage.md`, `docs/development/MX_RUST_CLI_AND_MCP_SERVER.md`, `docs/development/RUST_CLI_DEVELOPMENT.md`, `docs/development/QUICK_REFERENCE.md` (whichever still reference Weaviate), plus `README.md` if it mentions Weaviate.

- [ ] **Step 1: Update stale RAG sections**

In each file found by `grep -rli weaviate docs/development README.md`, rewrite the RAG/Weaviate sections to describe the pgvector corpus: backend (Neon primary / local Postgres fallback via `~/.mech-crate/config/rag.toml`), `mx rag ingest` / `mx rag status`, the 8 `rag_*` MCP tools (including `rag_context`), and the embedding adapter (`text-embedding-3-small` default, `OPENAI_API_KEY`). Delete instructions about docker-compose Weaviate, `mx mcp start/stop/ingest`, and the transformers container.

- [ ] **Step 2: Final verification**

```bash
grep -rli weaviate docs/development README.md Makefile make templates scripts 2>/dev/null || echo "DOCS CLEAN"
grep -ri weaviate crates/ || echo "CODE CLEAN"
cargo build --workspace
MX_RAG_TEST_DATABASE_URL=postgres://postgres@localhost:55433/mx_rag cargo test --workspace
cargo run -p mx-cli -- rag ingest --dry-run
```
Expected: `DOCS CLEAN`, `CODE CLEAN`, builds/tests pass, dry run 0 warnings.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "docs: replace weaviate references with pgvector corpus workflow"
```
