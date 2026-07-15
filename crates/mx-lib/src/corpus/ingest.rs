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
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
    {
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
            warnings.push(format!(
                "{}: no valid frontmatter, using heuristics",
                rel_path
            ));
        }
        let fm = fm.unwrap_or_default();
        let title = fm
            .title
            .or_else(|| {
                body.lines()
                    .find_map(|l| l.strip_prefix("# ").map(|t| t.trim().to_string()))
            })
            .unwrap_or_else(|| {
                path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default()
            });
        let meta = DocMeta {
            path: rel_path.clone(),
            title: title.clone(),
            category: fm
                .category
                .unwrap_or_else(|| categorize_path(path).to_string()),
            languages: fm.languages,
            complexity: fm.complexity.unwrap_or_else(|| "intermediate".to_string()),
            use_cases: fm.use_cases,
            summary: fm.summary,
        };
        let chunks = chunk_markdown(&title, body, DEFAULT_CHUNK_CHARS);
        docs.push(ParsedDoc {
            path: path.to_path_buf(),
            rel_path,
            sha256,
            meta,
            chunks,
        });
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
                    summary.warnings.push(
                        "no embedding API key: chunks stored without embeddings (trigram-only search)"
                            .into(),
                    );
                }
                vec![None; texts.len()]
            }
        };
        for (chunk, embedding) in doc.chunks.iter().zip(embeddings) {
            summary.chunks_seen += 1;
            if store
                .insert_chunk(doc_id, chunk, &doc.meta, embedding)
                .await?
            {
                summary.chunks_new += 1;
            }
        }
        summary.docs_ingested += 1;
        tracing::info!("ingested {} ({} chunks)", doc.meta.path, doc.chunks.len());
    }
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    // Shared DB test lock: the store and ingest DB tests hit the same Postgres
    // database, so they serialize on one process-wide mutex (per plan note).
    use crate::corpus::store::db_lock;
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
        assert_eq!(plain.meta.category, "other"); // heuristic default
        assert!(warnings.iter().any(|w| w.contains("no-fm.md")));
    }

    #[tokio::test]
    async fn ingest_idempotent_and_replaces_changed() {
        let Some(url) = std::env::var("MX_RAG_TEST_DATABASE_URL").ok() else {
            return;
        };
        let _guard = db_lock().lock().await;
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
        let s1 = ingest(
            &store,
            &docs,
            &IngestOptions {
                clear: false,
                force: false,
            },
        )
        .await
        .unwrap();
        assert_eq!(s1.docs_ingested, 2);
        assert!(s1.chunks_new > 0);

        // unchanged re-run: all skipped
        let (docs, _) = scan_dir(dir.path()).unwrap();
        let s2 = ingest(
            &store,
            &docs,
            &IngestOptions {
                clear: false,
                force: false,
            },
        )
        .await
        .unwrap();
        assert_eq!(s2.docs_skipped, 2);
        assert_eq!(s2.chunks_new, 0);

        // change one file: it re-ingests
        fs::write(
            dir.path().join("no-fm.md"),
            "# Plain Doc\n\n## Beta\n\nCHANGED body",
        )
        .unwrap();
        let (docs, _) = scan_dir(dir.path()).unwrap();
        let s3 = ingest(
            &store,
            &docs,
            &IngestOptions {
                clear: false,
                force: false,
            },
        )
        .await
        .unwrap();
        assert_eq!(s3.docs_ingested, 1);
        assert_eq!(s3.docs_skipped, 1);
    }
}
