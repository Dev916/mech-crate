# Techniques RAG Library — Design Spec

**Date:** 2026-07-15
**Status:** Approved
**Repo:** mech-crate

## Overview

Build a development-techniques knowledge library, seeded from `docs/development/` (~50 documents: theory appendices, pattern playbooks, infra guides), exposed to Claude agents through the mx MCP server. Agents retrieve techniques relevant to whatever they are currently working on ("context-based retrieval"), at three hook points:

1. **Planning** — the `writing-devloop-plans` skill consults the library while authoring tasks.
2. **Execution** — each `devloop` task subagent consults the library before implementing.
3. **Ad-hoc** — a new standalone `techniques` skill teaches any agent when/how to use the tools.

The existing Weaviate-based RAG stack in `mx-mcp-server` is **replaced entirely** with Postgres + pgvector, following the proven `hq` corpus pattern (`~/dev/hq`, `crates/hq-corpus` + `migrations/002_corpus.sql`).

## Decisions (settled during brainstorming)

| Decision | Choice |
|---|---|
| Vector backend | pgvector on Neon (new dedicated project named `mech-crate`), local Postgres fallback. Weaviate module, GraphQL client, docker-compose, and transformers container are deleted. |
| Embeddings | `EmbeddingProvider` trait (adapter pattern). First impl: OpenAI-compatible `/embeddings` endpoint (configurable `base_url`/`api_key`/`model`), default `text-embedding-3-small`, 1536 dims — same as hq. Ollama later via the same trait (its `/v1/embeddings` is OpenAI-compatible). Per-agent providers are a future goal, not in scope. |
| Metadata | YAML frontmatter per doc (one-time authoring pass over all ~50 docs, seeded from INDEX.md). Heuristic fallback for docs without frontmatter. |
| Corpus layer | Port the hq pattern into a new `corpus` module inside `mx-mcp-server` (no shared crate, no cross-repo coupling; ~300 lines knowingly duplicated). |
| Hook points | writing-devloop-plans (plan authoring) + devloop subagent-prompt (per-task) + standalone `techniques` skill. |

## Architecture

```
crates/mx-lib/src/corpus/      — shared by mx-cli AND mx-mcp-server
    mod.rs          — public API: CorpusStore
    store.rs        — sqlx pool, Neon→local fallback, migrations, upserts, hybrid search
    embed.rs        — EmbeddingProvider trait + OpenAiCompatEmbedder (hq llm/openai.rs port)
    chunk.rs        — heading-aware chunker
    frontmatter.rs  — YAML frontmatter parser
    config.rs       — RagConfig (file + env)
    ingest.rs       — scan/ingest pipeline
crates/mx-lib/migrations/      — sqlx migrations

mx-mcp-server/src/
  rag/       — DELETED (Weaviate GraphQL client)
  weaviate/  — DELETED (auto-start manager)
  docker-compose.yml — DELETED

crates/mx-cli — new `mx rag ingest` / `mx rag status` subcommands
                (replaces the mx-ingest binary; `mx mcp` loses its Weaviate
                 start/stop/logs/ingest subcommands and its status becomes
                 corpus-backed)

docs/development/*.md — frontmatter added to every doc

~/.claude/skills/techniques/            — NEW skill
~/.claude/skills/devloop/subagent-prompt.md      — augmented
~/.claude/skills/writing-devloop-plans/SKILL.md  — augmented
```

**Data flow — ingest:** walk markdown files → parse frontmatter → doc sha256 skip → chunk → batch embed (256/request) → upsert chunks (content sha256 skip).

**Data flow — query:** MCP tool call → embed query text → hybrid SQL (cosine + trigram) with optional filters → markdown-formatted results grouped by source doc.

## Schema

```sql
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE technique_docs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    path TEXT UNIQUE NOT NULL,   -- upsert keyed by path
    title TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'other',
    languages TEXT[] NOT NULL DEFAULT '{}',
    complexity TEXT NOT NULL DEFAULT 'intermediate',
    use_cases TEXT[] NOT NULL DEFAULT '{}',
    summary TEXT,
    sha256 TEXT NOT NULL,        -- compared to skip unchanged files
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE technique_chunks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    doc_id UUID NOT NULL REFERENCES technique_docs(id) ON DELETE CASCADE,
    heading_path TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL,
    embedding vector(1536),
    embedding_model TEXT NOT NULL,
    content_sha256 TEXT UNIQUE NOT NULL,
    category TEXT NOT NULL DEFAULT 'other',      -- denormalized for filter speed
    languages TEXT[] NOT NULL DEFAULT '{}',       -- denormalized
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX technique_chunks_embedding_hnsw
    ON technique_chunks USING hnsw (embedding vector_cosine_ops);
CREATE INDEX technique_chunks_content_trgm
    ON technique_chunks USING gin (content gin_trgm_ops);
CREATE INDEX technique_chunks_category_idx ON technique_chunks (category);
```

Migrations run via `sqlx::migrate!` on connect (both Neon and local get identical schema).

## Search

hq's hybrid formula verbatim, plus filters:

```sql
SELECT ..., (0.85 * (1 - (embedding <=> $1)) + 0.15 * similarity(content, $2)) AS score
FROM technique_chunks
WHERE ($3::text IS NULL OR category = $3)
  AND ($4::text IS NULL OR $4 = ANY(languages))
ORDER BY score DESC LIMIT $5;
```

**Degradation:** if no embedding provider is configured/reachable, search runs trigram-only and says so in the tool output.

**Provider switching:** `embedding_model` is stored per chunk. If configured model ≠ stored model, tools warn to run `mx rag ingest --reembed`. A dimension change (e.g. 1536 → 768 for Ollama) additionally requires a documented migration; not automated.

## Ingestion (`mx rag ingest`)

Flags: `--path <dir>` (default `<repo>/docs/development`), `--clear`, `--reembed`, `--force`.

`mx rag status` is a thin CLI twin of the `rag_health` tool: prints active backend, doc/chunk counts, embedding model, last ingest time.

1. Walk `*.md`, skipping `INDEX.md` (frontmatter supersedes it as machine-readable metadata).
2. Parse YAML frontmatter:
   ```yaml
   ---
   title: Rust Concurrency Deep Dive
   category: concurrency
   languages: [rust]
   complexity: expert        # intermediate | advanced | expert | research
   use_cases: [lock-free data structures, memory ordering decisions]
   summary: Concurrency primitives with performance analysis and memory ordering.
   ---
   ```
   Missing/malformed frontmatter → warn + heuristic fallback (title from first `#`, category from path keywords); never aborts the batch.
3. Doc-level `sha256` skip; chunk-level `content_sha256` skip. Re-runs are cheap and idempotent.
4. Chunking: split on `##` headings; sections > ~1200 chars sub-split at paragraph boundaries; every chunk's content is prefixed with its `Doc Title > Heading` path so chunks are self-contextualizing.
5. Batch embed (256 inputs per `/embeddings` request) → upsert.

**Category taxonomy** (starter, open vocabulary): `theory`, `patterns`, `architecture`, `concurrency`, `api-design`, `database`, `frontend`, `docker`, `infra`, `shell`, `blockchain`, `ml`, `security`, `process`. Documented in a short authoring guide section added to `docs/development/INDEX.md`.

**Frontmatter authoring pass:** implementation includes adding frontmatter to all ~50 docs, seeded from INDEX.md's existing per-doc metadata (languages, complexity, use cases, keywords).

## MCP tool surface

7 existing names re-pointed to pgvector + 1 new tool. All descriptions rewritten to be technique-oriented.

| Tool | Behavior |
|---|---|
| `rag_context` **(NEW — primary)** | `working_on` (free text) + optional `language`, `category`, `limit` → hybrid search, results grouped by doc with source paths and `use_cases`. The entry point used by skills/devloop. |
| `rag_search` | General hybrid search |
| `rag_search_category` | Hybrid search + category filter |
| `rag_get_guidance` | `problem` + `constraints[]` → guidance-formatted results |
| `rag_find_implementation` | `concept` + `language` → filters `languages[]` |
| `rag_compare_approaches` | Two approaches → side-by-side relevant chunks |
| `rag_find_related` | Technique/doc title → related chunks from *other* docs |
| `rag_health` | Backend in use (`neon` / `local` / `offline`), doc + chunk counts, embedding model, last ingest time |

## Configuration & connection fallback

`~/.mech-crate/config/rag.toml` (matching the existing `~/.mech-crate/config/` layout), env-overridable:

```toml
database_url = "postgres://...neon.tech/mx_rag"        # primary (Neon)
fallback_database_url = "postgres://localhost:5432/mx_rag"  # local
embedding_base_url = "https://api.openai.com/v1"
embedding_model = "text-embedding-3-small"
# api key via OPENAI_API_KEY env (or [rag].embedding_api_key)
```

On startup: try primary with a short connect timeout → on failure use fallback → log which backend is active. Both down → tools return an actionable message; the MCP server never crashes over corpus unavailability. No local-Postgres auto-start (deliberate departure from the Weaviate auto-start pattern); docs include a `docker run pgvector/pgvector:pg17` one-liner.

**Provisioning:** the Neon project is created once via the Neon MCP during implementation (org PriceLove LLC); the connection string goes into config.

## Error handling summary

| Condition | Behavior |
|---|---|
| Neon unreachable | Fall back to local Postgres, log it |
| Both DBs down | Tools return actionable offline message; `rag_health` says `offline` |
| No embedding API key | Ingest fails loudly; search degrades to trigram-only with a note |
| Frontmatter malformed | Warn + heuristic fallback, continue batch |
| Configured model ≠ stored model | Tools warn to `mx rag ingest --reembed` |
| Empty results | Suggest `rag_health` + rephrasing |

## Skill & devloop integration

**New skill `~/.claude/skills/techniques/`** — triggers: deciding how to implement something, choosing between architectures/patterns, starting feature work in a covered domain, `/techniques`. Teaches:
1. Describe the current task in 1–2 sentences → `rag_context` (+ language/stack).
2. Drill down with `rag_find_implementation` / `rag_compare_approaches` when weighing options.
3. Treat results as advisory patterns; cite the source doc in plan/PR; don't cargo-cult.
4. If offline (`rag_health`), note it and continue — never block work.

**writing-devloop-plans** — one new step: before writing tasks, call `rag_context` with the feature description + stack; weave techniques into task design; tasks may carry an `Apply: <doc> — <technique>` line the executing subagent inherits.

**devloop `subagent-prompt.md`** — short "consult techniques" section: one `rag_context` call (task title + criteria, `limit: 3`) before implementing; apply if relevant; proceed regardless if unavailable. Kept light to protect subagent context budget.

## Testing

- **Unit:** chunker (heading splits, sub-splits, prefixing), frontmatter parser (valid/missing/malformed), embedding response parsing (fixtures), connection fallback (bad primary → fallback).
- **DB integration** (hq-style; requires local pgvector, skipped when absent): upsert idempotency, hybrid scoring, category/language filters.
- **End-to-end:** ingest fixture docs → `rag_context` returns expected doc; `rag_health` correct across neon/local/offline states.
- **Live verification** (devloop `cli`/`api` toolkits): real ingest of the full corpus, sample queries via MCP tools, Neon path verified post-provisioning.
- **Removal check:** `grep -ri weaviate` across the repo returns nothing (code, compose, docs).

## Out of scope / future

- Per-agent embedding providers (trait makes this possible later).
- Ingesting folders beyond `docs/development` (pipeline takes `--path`, so trivial later).
- Automated re-embed/migration on dimension change (documented manual step).
- Remote/shared corpus API (context-lake style); this is single-user, config-driven.
