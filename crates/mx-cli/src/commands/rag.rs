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
    /// Mine research-gap themes from weak-scoring rag queries
    Gaps {
        /// Look-back window in days
        #[arg(long, default_value_t = 30)]
        days: i64,
        /// Minimum occurrences for a theme to report
        #[arg(long, default_value_t = 2)]
        min_count: i64,
    },
}

impl RagCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            RagSubcommand::Ingest {
                path,
                clear,
                force,
                reembed,
                dry_run,
            } => {
                self.ingest(path.clone(), *clear, *force, *reembed, *dry_run)
                    .await
            }
            RagSubcommand::Status => self.status().await,
            RagSubcommand::Gaps { days, min_count } => self.gaps(*days, *min_count).await,
        }
    }

    fn default_docs_dir() -> Result<PathBuf> {
        Ok(mx_lib::paths::mech_crate_root()?.join("docs/development"))
    }

    async fn ingest(
        &self,
        path: Option<PathBuf>,
        clear: bool,
        force: bool,
        reembed: bool,
        dry_run: bool,
    ) -> Result<()> {
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
        println!(
            "{} Connected to {} backend",
            style("→").cyan().bold(),
            store.backend().label()
        );
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
        println!(
            "  {} Backend: {}",
            style("•").dim(),
            st["backend"].as_str().unwrap_or("?")
        );
        println!("  {} Docs: {}", style("•").dim(), st["docs"]);
        println!("  {} Chunks: {}", style("•").dim(), st["chunks"]);
        println!(
            "  {} Embedding model: {}",
            style("•").dim(),
            st["embedding_model"].as_str().unwrap_or("?")
        );
        if cfg.embedding_api_key.is_some() {
            println!(
                "  {} Embedding key: {}",
                style("•").dim(),
                style("configured").green()
            );
        } else {
            println!(
                "  {} Embedding key: {} — lexical-only retrieval; set embedding_api_key in ~/.mech-crate/config/rag.toml or export OPENAI_API_KEY",
                style("•").dim(),
                style("MISSING").yellow().bold()
            );
        }
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
        println!(
            "  {} Logged queries: {}",
            style("•").dim(),
            st["logged_queries"]
        );
        Ok(())
    }

    async fn gaps(&self, days: i64, min_count: i64) -> Result<()> {
        let cfg = RagConfig::load();
        let store = CorpusStore::connect(&cfg).await?;
        let gaps = store.gaps(days, min_count).await?;
        if gaps.is_empty() {
            println!(
                "{} No gap themes in the last {} days (min count {}).",
                style("✓").green().bold(),
                days,
                min_count
            );
            return Ok(());
        }
        println!(
            "{}",
            style(format!("Research gaps — last {} days", days)).bold()
        );
        for g in gaps {
            let avg = g
                .avg_score
                .map(|s| format!("{:.2}", s))
                .unwrap_or_else(|| "n/a".into());
            println!(
                "  {} {} — {} hits, avg score {}, last {}",
                style("•").dim(),
                g.theme,
                g.count,
                avg,
                g.last_seen.format("%Y-%m-%d")
            );
        }
        Ok(())
    }
}
