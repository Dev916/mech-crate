---
title: Research Log
category: process
complexity: intermediate
use_cases:
  - auditing technique research runs
  - tracing corpus growth over time
summary: Append-only audit log of technique-research runs — topic, verdict, sources count, PR link.
---

# Research Log

Append-only. One row per research run, newest first. Written by the `technique-research` skill in Phase 5 (or on no-op).

| Date | Topic | Verdict | Sources | Outcome |
|---|---|---|---|---|
| 2026-07-28 | Tries/radix trees + trie-path dispatch (forst pattern) | NEW | 19 | Authored `tries-and-radix-dispatch.md` (web research + forst repo as primary source; ART paper read in full, 68-claim report) |
| 2026-07-26 | pgvector in Rust + concurrent batch embedding pipelines | NEW | 20 | Authored `pgvector-rust-batch-embedding.md` (web research; incl. Neon-pooler statement-cache trap and HNSW filtered-recall trap both live in our corpus) |
| 2026-07-26 | RAG retrieval quality: hybrid fusion + chunking for code corpora | NEW | 16 | Authored `rag-retrieval-fusion-and-chunking.md` (web research; incl. empirically verified finding that our pg_trgm lexical arm is inert) |
| 2026-07-19 | mx cloudflare infra: scaffolding, credentials, worker+container deploy | NEW | 9 repo sources | Authored `mx-cloudflare-deploy.md` (8-item drift inventory incl. phantom `mx cf` command and credential env-var mismatch) |
| 2026-07-19 | mx recipes/blueprints lifecycle + image build pipeline | NEW | 12 repo sources | Authored `mx-recipes-and-build.md` (internal codebase research; 10-item gaps inventory incl. missing consumer-update provenance) |
| 2026-07-19 | mx framework app playbook: anatomy, scaffolding, migration, always-use-router | NEW | 9 repo sources | Authored `mx-app-playbook.md` (internal codebase research; web/x/hn providers skipped — proprietary topic) |
| 2026-07-18 | Rust async cancellation and graceful shutdown patterns | NEW | 9 | Authored `rust-async-cancellation-graceful-shutdown.md` — [PR #14](https://github.com/Dev916/mech-crate/pull/14) |
| 2026-07-18 | Rust atomics memory ordering (Acquire/Release/SeqCst) selection | FRESH | 0 | No-op — covered & current in appendix-rust-concurrency.md (top score 0.59, complexity expert); no PR |
