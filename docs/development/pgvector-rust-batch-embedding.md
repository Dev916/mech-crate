---
title: "pgvector in Rust: Operational Practice and Concurrent Batch Embedding"
category: database
languages: [rust, sql]
complexity: advanced
use_cases:
  - operating pgvector (HNSW tuning, quantization, filtered search) from Rust/sqlx
  - avoiding pooled-connection traps (statement cache, session GUCs) with Neon/PgBouncer
  - building a concurrent batch-embedding pipeline with rate limits, retries, and backpressure
  - deciding when an ANN index is worth having at all
summary: Evidence-based operational guidance for pgvector from Rust — HNSW/halfvec/filtered-search mechanics with vendor benchmarks, the pooler gotchas, and the correct tokio pipeline shape for batch embedding (buffer_unordered + two-axis governor + bounded-channel writer), with the OpenAI API's real limits.
provenance: researched
researched: 2026-08-14
sources:
  - https://github.com/pgvector/pgvector/blob/master/README.md
  - https://github.com/pgvector/pgvector-rust/blob/master/README.md
  - https://docs.rs/sqlx/latest/sqlx/postgres/struct.PgConnectOptions.html
  - https://neon.com/docs/ai/ai-vector-search-optimization
  - https://neon.com/blog/dont-use-vector-use-halvec-instead-and-save-50-of-your-storage-cost
  - https://jkatz05.com/post/postgres/pgvector-scalar-binary-quantization/
  - https://supabase.com/blog/pgvector-fast-builds
  - https://aws.amazon.com/blogs/database/supercharging-vector-search-performance-and-relevance-with-pgvector-0-8-0-on-amazon-aurora-postgresql/
  - https://docs.pgedge.com/pgvector/v0-8-0/iterative-index-scans/
  - https://www.postgresql.org/about/news/pgvector-082-released-3245
  - https://docs.rs/futures/latest/futures/stream/struct.BufferUnordered.html
  - https://github.com/rust-lang/futures-rs/issues/2387
  - https://docs.rs/governor
  - https://docs.rs/backon/latest/backon/
  - https://developers.openai.com/api/reference/python/resources/embeddings/methods/create
  - https://github.com/openai/openai-cookbook/blob/main/examples/api_request_parallel_processor.py
  - https://developers.openai.com/api/docs/guides/batch
  - https://www.baseten.co/blog/your-client-code-matters-10x-higher-embedding-throughput-with-python-and-rust/
  - https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html
  - https://supabase.com/blog/matryoshka-embeddings
  - https://github.com/pgvector/pgvector/blob/master/CHANGELOG.md
  - https://cve.threatint.com/CVE/CVE-2026-18022
  - https://github.com/pgvector/pgvector/pull/989
---

# pgvector in Rust: Operations + Batch Embedding Pipelines

Operational truths for running pgvector from Rust and embedding at scale, with vendor-benchmark evidence. Inline `[n]` cites key to `sources`.

## 1. Rust access + the pooler traps

The `pgvector` crate (0.4.x for sqlx 0.8/0.9) exposes `Vector`/`HalfVector`/`SparseVector`/`Bit` behind cargo features (`halfvec` is NOT default) — binding is plain `bind()/try_get()` [2].

**Trap 1 — statement cache vs transaction-mode poolers.** sqlx caches prepared statements per connection (default capacity 100). Through PgBouncer transaction mode — including **Neon's `-pooler` endpoints** — cached statement names collide across physical backends: intermittent `prepared statement "sqlx_s_N" already exists` under load. Fix: `statement_cache_capacity(0)` (or `?statement-cache-capacity=0`) when connecting through a pooler; keep the default on direct connections [3]. PgBouncer ≥1.22 supports protocol-level prepared statements, but verify your pooler's version rather than assume.

**Trap 2 — session GUCs leak through pools.** `hnsw.ef_search` (default 40) is session-level: a bare `SET` on a pooled connection contaminates unrelated queries that later land on that backend. Always scope it: `BEGIN; SET LOCAL hnsw.ef_search = N; SELECT …; COMMIT;` — in sqlx, a transaction with the `SET LOCAL` issued on `&mut *tx` [1].

**Bulk writes**: `INSERT … SELECT FROM UNNEST($1::uuid[], $2::vector[]) ON CONFLICT … DO UPDATE` beats multi-VALUES (~4× at 100k rows); binary COPY only wins above ~10k rows/batch.

## 2. HNSW: tuning, size, maintenance

- Defaults: `m=16`, `ef_construction=64`, `ef_search=40` [1]. Production band for 1536-dim embeddings across real benchmarks: `m=16–32`, `ef_construction=128–256`; set `ef_search ≥ LIMIT` [4][7].
- **`maintenance_work_mem` is the build lever**: builds are dramatically faster when the graph fits (pgvector NOTICEs when it doesn't); raise `max_parallel_maintenance_workers` (default 2). Supabase: 1M×1536 builds in ~5–9.5 min with 15 workers + 30GB, vs 39–87 min prior [7]. Neon: cap `maintenance_work_mem` at 50–60% of compute RAM; scale compute up to build, down to serve [4][5].
- Memory: index ≈ 1.5–2× raw vectors (1536-dim ≈ 6KB/row). ANN is only fast while the graph is cache-resident — autoscaling down can push it out and cliff your first queries after scale-up.
- **Index eligibility**: the query must have `ORDER BY <distance-op> ASC` + `LIMIT`, with the ORDER BY directly on the operator — `ORDER BY 1 - (embedding <=> $1) DESC` silently skips the index [1].
- **When NOT to index**: exact search is fine below ~10k–100k rows (parallel seq scan, 100% recall — `SET max_parallel_workers_per_gather = 4`). Start without an index; add HNSW when queries actually slow [4].
- **Churn degrades recall, not just space**: MVCC re-embeds leave dead tuples in the graph; documented failure path to near-0% recall until vacuum. pgvector's own advice: `REINDEX INDEX CONCURRENTLY` first, then `VACUUM` (HNSW vacuum is slow) [1]. Always `CREATE INDEX CONCURRENTLY` on live tables.
- **Versions**: **0.8.6 is current** (2026-07-29): fixes an IVFFlat build buffer overflow on 32-bit systems, an array→`sparsevec` cast not limiting non-zero elements, and IVFFlat scan memory usage under nested-loop joins [21]. **CVE-2026-3172** (buffer overflow in parallel HNSW builds, can leak other relations' data) affects 0.8.0/0.8.1 — check `SELECT extversion FROM pg_extension WHERE extname='vector'` and be ≥0.8.2 [10]. **CVE-2026-18022** (integer wraparound → out-of-bounds write during IVFFlat index build; ≤0.8.5, fixed 0.8.6) is rated CVSS High but **only 32-bit builds are affected** — 64-bit deployments (Neon, RDS, standard x86-64/arm64) are not practically vulnerable; upgrade as hygiene, not urgency [22]. Watch item: PostgreSQL 19 GA lands Sept 2026 — verify pgvector compatibility before moving.

## 3. Quantization: what's proven

- **halfvec (fp16) is the near-free win at 1536 dims**: ~50% storage, ~23% faster builds, 50% faster prewarm, equivalent recall/latency — confirmed independently by Neon and a pgvector maintainer [5][6]. Try it via expression index first (`USING hnsw ((embedding::halfvec(1536)) halfvec_cosine_ops)`) — queries must use the identical cast to hit the index [1].
- **Binary quantization is dataset-dependent and contested**: Katz measured 91.6% recall at 1536-dim with 4× over-fetch rescoring; Neon measured ~50% on the same dimensionality and recommends against. Measure on your data or skip [5][6]. Reinforcing signal: the TurboQuant proposal ("10× smaller vector indexes", PR #989) was **closed unmerged** in 2026-07 — upstream still ships nothing beyond `binary_quantize` [23].
- **Matryoshka dimensions**: `text-embedding-3-small` was trained at {512, 1024, 1536} — the API `dimensions` param gives a 3× storage cut *before* halfvec and cheaper distance math; requires re-embedding, so decide while the corpus is small [20].

## 4. Filtered vector search: the recall trap

With HNSW, a plain `WHERE` filter is applied **after** the index scan: at `ef_search=40`, a filter matching 10% of rows returns ~4 results *regardless of LIMIT* [8][9]. pgvector 0.8.0's iterative scans fix this (`hnsw.iterative_scan = strict_order | relaxed_order`; knobs `max_scan_tuples`=20k, `scan_mem_multiplier`); AWS measured filtered recall going **10% → 100%** with 1.5–1.8× latency improvement [8].

Decision ladder: highly selective filter → B-tree on the filter column and let the planner skip HNSW; moderately selective → `SET LOCAL hnsw.iterative_scan = strict_order`; latency-critical → `relaxed_order`, **but then you MUST re-sort via a `MATERIALIZED` CTE** or ordering is silently wrong [9]; few fixed filter values → partial HNSW indexes; many values at scale → partitioning [1].

## 5. The batch-embedding pipeline (Rust/tokio)

**API hard limits** (OpenAI-compatible `/embeddings`) [15]: 8,192 tokens per input · **2048 inputs per request** · **300,000 tokens summed per request** · empty strings rejected. The token cap binds first in real corpora (~600 inputs at 500-token chunks) — **batch by tokens, not count** (tiktoken `cl100k_base`, count-only path), with headroom (~250k) for tokenizer skew.

**The shape** (each element evidence-backed):

```
chunks → token-count + oversize guard (>8192 → split/dead-letter)
      → token-aware batcher (n==MAX || tokens>250k)
      → .map(|b| async { rpm.until_ready(); tpm.until_n_ready(b.tokens); retry(embed(b)) })
      → .buffer_unordered(N)                      // N ≈ target_RPS × mean_latency; start 16
      → bounded mpsc::channel(64) → writer task   // UNNEST upsert, ON CONFLICT DO UPDATE
```

- `buffer_unordered` starts futures in order, yields in completion order, and IS the backpressure mechanism — the source stops being polled when N futures are in flight [11]. IO-bound fan-out doesn't need JoinSet/multi-core.
- **The footgun**: `FuturesUnordered` futures only progress while polled — a slow `.await` (like a DB insert) in the consuming loop **stalls every in-flight HTTP request** [12]. Hence the bounded-channel hand-off to a dedicated writer; bounded (never `unbounded_channel`) so a lagging DB propagates backpressure instead of buffering the corpus in RAM [19].
- **Rate limiting**: OpenAI enforces RPM and TPM *independently* (org-wide — a backfill starves production traffic). Two `governor` limiters: requests (1 cell/call) and tokens (`until_n_ready(batch_tokens)`); set to ~80% of your dashboard's actual limits, not blog tables [13][16]. Honor `retry-after-ms` on 429s — it's calibrated to your request.
- **Retries**: `backon` (the `backoff` crate is unmaintained) — exponential + jitter around the HTTP call only; retry 429/5xx/timeout, never 400/401; 400s bisect-and-retry to isolate poison items. Embedding calls are effect-idempotent — make the *write* idempotent and aggressive retry is safe [14].
- **Cheap wins**: share one `reqwest::Client` with `pool_max_idle_per_host ≥ N`; `encoding_format: "base64"` (a 256×1536 float JSON response is ~7.5MB to parse).
- **Backfills**: consider the **Batch API** — 50% cheaper, 50k embedding inputs per batch, typically 1–6h, separate quota from your synchronous RPM/TPM [17]. Right architecture: Batch API for bulk reindex, the streaming pipeline for incremental.
- Track per-chunk state (`pending/done/failed` + dead-letter) so runs are resumable and retries never re-bill [16].

## Synthesis (inferred)

Applied to the mx techniques corpus (Rust/tokio/sqlx/pgvector on Neon pooled endpoint, sequential ≤256-input batches, no retries/rate-limiting, HNSW defaults, ~2k chunks):

1. **Pooler safety now**: `statement-cache-capacity=0` on the Neon `-pooler` URL (§1 trap 1) — the failure is intermittent-under-load, the worst kind. Verify pgvector ≥0.8.2 (CVE).
2. **Filtered-search audit**: our `category`/`language` filters ride on HNSW with default `ef_search` — the §4 trap class. At 2k chunks the honest fix is simpler: exact search (drop/skip the index) gives 100% recall in milliseconds; revisit HNSW + iterative scans past ~50k chunks.
3. **Ingest hardening, in order**: token-aware batching (latent 400 as chunks grow), `backon` retries + `ON CONFLICT` idempotent writes, then `buffer_unordered(16)` + bounded-channel writer + two-axis governor when corpus growth makes sequential ingest slow.
4. **At next re-embed**: evaluate `dimensions: 512` (Matryoshka) and halfvec via expression index — both near-free at this corpus size; skip binary quantization.
5. Re-embedding backfills should end with `REINDEX INDEX CONCURRENTLY` (§2 churn/recall) — or nothing, if we've dropped the index at this scale.
