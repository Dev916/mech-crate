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
pub use store::{BackendKind, CorpusStore, DocMeta, SearchMode, TechHit, TechQuery};
