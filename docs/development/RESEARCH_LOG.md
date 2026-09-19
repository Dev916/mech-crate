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
| 2026-09-18 | System One decision models and Jev (seed: Syntax video via vidwatch) | NEW | 42 | Authored `system-one-decision-models-jev.md` (what a System One model returns, the three primitives and confidence contract, the prescribed patterns, the in-front/beside/behind/instead-of placement rule, and an independent-evidence table that lands far below the vendor's headline multipliers) |
| 2026-09-18 | Enterprise Django architecture (layout, service layer, ORM, migrations, APIs, async, tenancy, testing) | NEW | 46 | Authored `django-enterprise-architecture.md` (Django 6.1 line verified against djangoproject.com release notes; reconciles HackSoft services/selectors against Bennett's model-first rebuttal; documents 6.1 fetch modes, the zero-downtime migration sequence, and that Django 6.0/6.1 ship no task worker despite DEP 14's DatabaseBackend) |
| 2026-09-18 | Django at scale (security hardening, performance, observability, operations) | NEW | 43 | Authored `django-production-hardening.md` (Django 6.1 line: every `check --deploy` item with its documented default, built-in CSP from 6.0 vs django-csp, the mutually exclusive CONN_MAX_AGE / native psycopg pool / PgBouncer strategies, server and worker sizing, OpenTelemetry under pre-fork workers, LTS calendar and advisory tracking) |
| 2026-09-18 | FRP in Python (family doc: reactivex, asyncio, anyio/trio, Textual) | NEW | 29 | Authored `appendix-frp-python.md` (corrects the family's RxPY-v4 assumption to reactivex 5.1.0 with method chaining; bounded queues plus TaskGroup/timeout/Queue.shutdown as the backpressure and cancellation story; runnable asyncio + reactivex intent loop) |
| 2026-09-18 | Python performance engineering (free-threading, JIT, profiling, async, native) | NEW | 45 | Authored `python-performance-engineering.md` (measure-first playbook; every interpreter figure re-verified verbatim against CPython What's New 3.13/3.14/3.15 and PEPs 703/779/744/836/734/669/799; corrects the widely repeated "ThreadPoolExecutor defaults to cores x 5" to the current `min(32, (os.process_cpu_count() or 1) + 4)`) |
| 2026-09-18 | Secure Python (supply chain, input handling, secrets, runtime hardening) | NEW | 48 | Authored `python-secure-coding-practices.md` (primary-source audit: Trusted Publishing and PEP 740 assert publisher identity, not artifact safety; tarfile's `data` default landed in 3.14; `re` has no timeout parameter; argon2id parameters taken verbatim from OWASP; CPython cannot sandbox itself) |
| 2026-09-13 | MCP server tool design: citation correction pass (review of PR #38) | IMPROVE | 13 | Amended `mcp-server-tool-design.md`: Claude Code tool-search gating restated per [6] (model generation, not v2.1.232; discovery cache needs v2.1.221 and is off by default since v2.1.238), 150k->2k saving reattributed to definition loading per [9], dropped the unsourced 15-20% description-tuning figure, prefix/suffix claim scoped to [8], no-parameter schema shape marked Recommended not MUST, `x-mcp-header` marked SHOULD NOT, annotations MUST qualified with trusted-servers, `defer_loading` set per `mcp_toolset` for connector tools; no sources added |
| 2026-09-12 | MCP server tool design: workflow tools, progressive discovery, code mode (seed: Neon video via vidwatch) | NEW | 13 | Authored `mcp-server-tool-design.md` (video watched frame-by-frame with vidwatch; every client-side claim checked against MCP 2026-07-28 client best practices + tools spec, Claude/OpenAI tool-search docs, Neon package READMEs; flags the video's Claude Code/Codex code-mode claim as overstated) |
| 2026-08-14 | Multi-agent LLM systems: comms, lifecycle, delegation, verification (sweep leg A) | NEW | 24 | Authored `multi-agent-systems-in-practice.md` (meeting-notes-steered sweep; incl. Managed Agents hub-and-spoke/interrupt/advisor primitives, measured A2A-vs-MCP adoption gap, cheating-agents verification evidence) |
| 2026-08-14 | LLM token & cache efficiency engineering (sweep leg B) | NEW | 24 | Authored `llm-token-cache-efficiency.md` (falsified the ~90-min cache TTL claim; corrected stale 15x delegation math to 5x; credit-cliff 1h->5m TTL drop; worktree cache-split finding) |
| 2026-08-14 | Vector DB updates: pgvector + retrieval fusion (sweep leg C) | IMPROVE+FRESH | 9 | Amended `pgvector-rust-batch-embedding.md` (0.8.6 release, CVE-2026-18022 with 32-bit-only calibration, TurboQuant PR closed unmerged); `rag-retrieval-fusion-and-chunking.md` verified FRESH, no edit |
| 2026-07-28 | Tries/radix trees + trie-path dispatch (forst pattern) | NEW | 19 | Authored `tries-and-radix-dispatch.md` (web research + forst repo as primary source; ART paper read in full, 68-claim report) |
| 2026-07-26 | pgvector in Rust + concurrent batch embedding pipelines | NEW | 20 | Authored `pgvector-rust-batch-embedding.md` (web research; incl. Neon-pooler statement-cache trap and HNSW filtered-recall trap both live in our corpus) |
| 2026-07-26 | RAG retrieval quality: hybrid fusion + chunking for code corpora | NEW | 16 | Authored `rag-retrieval-fusion-and-chunking.md` (web research; incl. empirically verified finding that our pg_trgm lexical arm is inert) |
| 2026-07-19 | mx cloudflare infra: scaffolding, credentials, worker+container deploy | NEW | 9 repo sources | Authored `mx-cloudflare-deploy.md` (8-item drift inventory incl. phantom `mx cf` command and credential env-var mismatch) |
| 2026-07-19 | mx recipes/blueprints lifecycle + image build pipeline | NEW | 12 repo sources | Authored `mx-recipes-and-build.md` (internal codebase research; 10-item gaps inventory incl. missing consumer-update provenance) |
| 2026-07-19 | mx framework app playbook: anatomy, scaffolding, migration, always-use-router | NEW | 9 repo sources | Authored `mx-app-playbook.md` (internal codebase research; web/x/hn providers skipped — proprietary topic) |
| 2026-07-18 | Rust async cancellation and graceful shutdown patterns | NEW | 9 | Authored `rust-async-cancellation-graceful-shutdown.md` — [PR #14](https://github.com/Dev916/mech-crate/pull/14) |
| 2026-07-18 | Rust atomics memory ordering (Acquire/Release/SeqCst) selection | FRESH | 0 | No-op — covered & current in appendix-rust-concurrency.md (top score 0.59, complexity expert); no PR |
