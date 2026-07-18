# Technique Research Mode (Self-Growing Library) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the techniques corpus self-growing: a `technique-research` skill that researches topics (directed or autonomous), improves/authors technique docs PR-gated, plus rag query logging + `mx rag gaps` for gap mining, a weekly local cron, and filed follow-up issues.

**Architecture:** The research engine is a skill (agent-native judgment + the existing deep-research skill); Rust adds only persistent query logging (`rag_queries` table, fire-and-forget insert in `CorpusStore::search`, `gaps()` aggregation, `mx rag gaps`). Growth artifacts live in `docs/development` (RESEARCH_BACKLOG.md, RESEARCH_LOG.md, provenance frontmatter). Corpus learns only on merge.

**Tech Stack:** Rust (sqlx/pgvector as established), markdown skills, gh CLI, Claude Code cron (CronCreate).

**Spec:** `docs/superpowers/specs/2026-07-18-self-growing-techniques-design.md`

**Compatible with:** devloop skill v0.1+

## Global Constraints

- Query logging must NEVER fail or slow a search: spawned task, errors traced and dropped (no panics in domain paths; fail fast only at process boundaries).
- Gap threshold exactly `top_score < 0.45 OR top_score IS NULL`; normalization = lowercase + collapse whitespace runs to single spaces (trimmed).
- Test DB pattern unchanged: `MX_RAG_TEST_DATABASE_URL=postgres://postgres@localhost:55433/mx_rag` (container `mx-rag-test`; `docker start mx-rag-test` if exited); DB tests skip when unset and serialize on `db_lock()`.
- Skill files: author in `~/.claude/skills/technique-research/`, snapshot identically into repo `skills/technique-research/`.
- All Rust work follows the existing corpus module conventions (anyhow, runtime sqlx queries, no compile-time macros). Run `cargo fmt` on touched files before each commit; git-restore unrelated fmt fallout.
- Conventional commit per task, trailer:
  `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>` + `Claude-Session: https://claude.ai/code/session_01QQozAZdFXi7WfRBxLpx3Ru`
- Branch: all tasks commit on `feat/technique-research-mode`. Research-run PRs created during verification use `research/<slug>` branches cut from this branch.

---

### Task 1: Query logging — migration, TechQuery.tool, fire-and-forget insert

**Acceptance Criteria (observable):**
- With the test DB, `cargo test -p mx-lib corpus::store` exits 0 including new tests proving: a `search()` call inserts one `rag_queries` row carrying the query text, tool name, mode, and the top hit's score; an empty-result search logs `top_score = NULL`; and `search()` still returns Ok results after `DROP TABLE rag_queries` (never-fail semantics).
- `rag_health` / `mx rag status` output now includes a `logged_queries` count (verified in Task 3's live check; here via the status unit assertion).
- `cargo build --workspace` exits 0 (all 7 MCP handler call sites updated with their tool names).

**Verify via:** cli

**Apply:** docs/development/appendix-rust.md — "No panics in domain paths; fail fast only at process boundaries" + boundary error mapping: logging is an edge effect, isolated in a spawned task that maps every failure to a trace.

**Files:**
- Create: `crates/mx-lib/migrations/0002_rag_queries.sql`
- Modify: `crates/mx-lib/src/corpus/store.rs`
- Modify: `crates/mx-mcp-server/src/tools/mod.rs` (add `tool:` to every `TechQuery { ... }` literal)

**Interfaces:**
- Consumes: existing `CorpusStore`, `TechQuery`, `SearchMode`, `db_lock()`.
- Produces: `TechQuery.tool: &'a str` (new field); `status()` JSON gains `"logged_queries": <i64>`.

- [ ] **Step 1: Write the migration**

Create `crates/mx-lib/migrations/0002_rag_queries.sql`:

```sql
CREATE TABLE IF NOT EXISTS rag_queries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    query TEXT NOT NULL,
    tool TEXT NOT NULL,
    category TEXT,
    language TEXT,
    top_score DOUBLE PRECISION,
    mode TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS rag_queries_created_idx ON rag_queries (created_at);
```

- [ ] **Step 2: Write the failing tests**

Add to the `tests` module in `crates/mx-lib/src/corpus/store.rs` (inside the existing patterns — reuse `test_cfg()`, `meta()`, `sparse()`, `db_lock()`):

```rust
    #[tokio::test]
    async fn search_logs_query_with_top_score() {
        let Some(cfg) = test_cfg() else { return };
        let _guard = db_lock().lock().await;
        let store = CorpusStore::connect(&cfg).await.unwrap();
        store.clear().await.unwrap();
        sqlx::query("DELETE FROM rag_queries").execute(store.pool()).await.unwrap();

        let m = meta("docs/log.md");
        let id = store.upsert_doc(&m, &sha256_hex("l")).await.unwrap();
        let c = Chunk { heading_path: "T > L".into(), content: "T > L\n\nlogging telemetry".into() };
        store.insert_chunk(id, &c, &m, None).await.unwrap();

        let (hits, _) = store
            .search(&TechQuery { text: "logging telemetry", category: None, language: None, limit: 5, tool: "rag_search" })
            .await
            .unwrap();
        assert_eq!(hits.len(), 1);
        store.flush_query_log().await; // deterministic: await the spawned insert

        let (query, tool, mode, score): (String, String, String, Option<f64>) = sqlx::query_as(
            "SELECT query, tool, mode, top_score FROM rag_queries ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_one(store.pool())
        .await
        .unwrap();
        assert_eq!(query, "logging telemetry");
        assert_eq!(tool, "rag_search");
        assert_eq!(mode, "trigram_only");
        assert!(score.is_some());
    }

    #[tokio::test]
    async fn empty_search_logs_null_score_and_drop_table_never_fails_search() {
        let Some(cfg) = test_cfg() else { return };
        let _guard = db_lock().lock().await;
        let store = CorpusStore::connect(&cfg).await.unwrap();
        store.clear().await.unwrap();
        sqlx::query("DELETE FROM rag_queries").execute(store.pool()).await.unwrap();

        let (hits, _) = store
            .search(&TechQuery { text: "zz nonexistent zz", category: None, language: None, limit: 5, tool: "rag_context" })
            .await
            .unwrap();
        assert!(hits.is_empty());
        store.flush_query_log().await;
        let score: Option<f64> =
            sqlx::query_scalar("SELECT top_score FROM rag_queries ORDER BY created_at DESC LIMIT 1")
                .fetch_one(store.pool())
                .await
                .unwrap();
        assert!(score.is_none());

        // status carries the count
        let st = store.status().await.unwrap();
        assert!(st["logged_queries"].as_i64().unwrap() >= 2);

        // never-fail: drop the table, search still succeeds
        sqlx::query("DROP TABLE rag_queries").execute(store.pool()).await.unwrap();
        let res = store
            .search(&TechQuery { text: "still works", category: None, language: None, limit: 5, tool: "rag_search" })
            .await;
        assert!(res.is_ok());
        store.flush_query_log().await; // insert fails silently
        // restore for subsequent tests
        sqlx::query(include_str!("../../migrations/0002_rag_queries.sql"))
            .execute(store.pool())
            .await
            .ok();
    }
```

Note: `include_str!` executes the whole migration file as one statement batch; if sqlx rejects multi-statement execution, split on `;` and execute each — implement whichever compiles, the intent is "recreate the table".

- [ ] **Step 3: Run tests to verify they fail**

Run: `MX_RAG_TEST_DATABASE_URL=postgres://postgres@localhost:55433/mx_rag cargo test -p mx-lib corpus::store`
Expected: FAIL — `tool` field and `flush_query_log` don't exist.

- [ ] **Step 4: Implement**

In `store.rs`:

1. Add `pub tool: &'a str,` to `TechQuery`.
2. Add a join-handle holder so tests can await the spawned insert:

```rust
pub struct CorpusStore {
    pool: PgPool,
    backend: BackendKind,
    embedder: Option<Arc<dyn EmbeddingProvider>>,
    model: String,
    last_log_task: tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
}
```

(initialize `last_log_task: tokio::sync::Mutex::new(None)` in `connect`.)

3. At the end of `search()` (NOT `search_with_embedding` — direct callers of that stay unlogged), after obtaining `(hits, mode)`:

```rust
        let log_pool = self.pool.clone();
        let entry = QueryLogEntry {
            query: q.text.to_string(),
            tool: q.tool.to_string(),
            category: q.category.map(String::from),
            language: q.language.map(String::from),
            top_score: hits.first().map(|h| h.score),
            mode: match mode {
                SearchMode::Hybrid => "hybrid",
                SearchMode::TrigramOnly => "trigram_only",
            }
            .to_string(),
        };
        let handle = tokio::spawn(async move {
            if let Err(e) = sqlx::query(
                "INSERT INTO rag_queries (query, tool, category, language, top_score, mode)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(&entry.query)
            .bind(&entry.tool)
            .bind(&entry.category)
            .bind(&entry.language)
            .bind(entry.top_score)
            .bind(&entry.mode)
            .execute(&log_pool)
            .await
            {
                tracing::debug!("rag query log insert failed (ignored): {e}");
            }
        });
        *self.last_log_task.lock().await = Some(handle);
        Ok((hits, mode))
```

with a small private struct:

```rust
struct QueryLogEntry {
    query: String,
    tool: String,
    category: Option<String>,
    language: Option<String>,
    top_score: Option<f64>,
    mode: String,
}
```

4. Test/support helper (compiled always; used by tests and harmless elsewhere):

```rust
    /// Await the most recent spawned query-log insert (deterministic tests).
    pub async fn flush_query_log(&self) {
        if let Some(h) = self.last_log_task.lock().await.take() {
            let _ = h.await;
        }
    }
```

5. In `status()`, add before the final `json!`:

```rust
        let logged_queries: i64 = sqlx::query_scalar("SELECT count(*) FROM rag_queries")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
```

(note: `query_scalar(...).fetch_one(...).await` returns `Result` — use `.unwrap_or(0)` via `match`/`unwrap_or_else` so a missing table can't fail status; concretely: `let logged_queries: i64 = sqlx::query_scalar("SELECT count(*) FROM rag_queries").fetch_one(&self.pool).await.unwrap_or(0);` requires the Result's Ok type to be i64 — write `.await.unwrap_or(0)` on the `Result<i64, _>`.) Add `"logged_queries": logged_queries,` to the JSON.

6. In `crates/mx-mcp-server/src/tools/mod.rs`, add the `tool:` field to every `TechQuery { ... }` literal with its handler's tool name: `"rag_context"`, `"rag_search"`, `"rag_search_category"`, `"rag_find_implementation"`, `"rag_get_guidance"`, `"rag_compare_approaches"` (each per-approach search), `"rag_find_related"`.

7. Update the two existing store tests that construct `TechQuery` (`hybrid_search_ranks_and_filters`, `trigram_only_when_no_embeddings`) to include `tool: "test"`.

- [ ] **Step 5: Run tests to verify they pass**

```bash
docker start mx-rag-test 2>/dev/null; sleep 2
MX_RAG_TEST_DATABASE_URL=postgres://postgres@localhost:55433/mx_rag cargo test -p mx-lib corpus::store
cargo build --workspace
```
Expected: all store tests pass (6 now); workspace builds.

- [ ] **Step 6: Commit**

```bash
git add crates/mx-lib/migrations/0002_rag_queries.sql crates/mx-lib/src/corpus/store.rs crates/mx-mcp-server/src/tools/mod.rs
git commit -m "feat(corpus): log rag queries fire-and-forget for gap mining"
```

---

### Task 2: Gaps aggregation in the store

**Acceptance Criteria (observable):**
- With the test DB, `cargo test -p mx-lib corpus::store` exits 0 including a new test proving: seeded queries below/at/above the 0.45 threshold aggregate correctly (weak + NULL counted, strong excluded), case/whitespace variants group into one theme, ordering is by count descending, and `min_count` filters singletons.

**Verify via:** cli

**Files:**
- Modify: `crates/mx-lib/src/corpus/store.rs`

**Interfaces:**
- Produces: `pub struct GapTheme { pub theme: String, pub count: i64, pub avg_score: Option<f64>, pub last_seen: chrono::DateTime<chrono::Utc> }` and `CorpusStore::gaps(&self, days: i64, min_count: i64) -> anyhow::Result<Vec<GapTheme>>`.

- [ ] **Step 1: Write the failing test**

```rust
    #[tokio::test]
    async fn gaps_aggregates_weak_queries() {
        let Some(cfg) = test_cfg() else { return };
        let _guard = db_lock().lock().await;
        let store = CorpusStore::connect(&cfg).await.unwrap();
        sqlx::query("DELETE FROM rag_queries").execute(store.pool()).await.unwrap();

        let seed = |q: &str, score: Option<f64>| {
            let pool = store.pool().clone();
            let q = q.to_string();
            async move {
                sqlx::query(
                    "INSERT INTO rag_queries (query, tool, top_score, mode) VALUES ($1, 'test', $2, 'hybrid')",
                )
                .bind(q)
                .bind(score)
                .execute(&pool)
                .await
                .unwrap();
            }
        };
        seed("GraphQL  Federation", Some(0.30)).await; // weak
        seed("graphql federation", Some(0.20)).await;  // same theme, different case/spacing
        seed("graphql federation", None).await;        // NULL counts as weak
        seed("solid rust patterns", Some(0.90)).await; // strong: excluded
        seed("lonely topic", Some(0.10)).await;        // weak but singleton

        let gaps = store.gaps(30, 2).await.unwrap();
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].theme, "graphql federation");
        assert_eq!(gaps[0].count, 3);
        let with_singletons = store.gaps(30, 1).await.unwrap();
        assert_eq!(with_singletons.len(), 2);
        assert_eq!(with_singletons[0].theme, "graphql federation"); // count desc
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `MX_RAG_TEST_DATABASE_URL=... cargo test -p mx-lib corpus::store gaps_aggregates`
Expected: FAIL — `gaps` not defined.

- [ ] **Step 3: Implement**

```rust
/// One mined gap theme from weak-scoring queries.
#[derive(Debug, Clone)]
pub struct GapTheme {
    pub theme: String,
    pub count: i64,
    pub avg_score: Option<f64>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

impl CorpusStore {
    /// Themes from queries in the last `days` whose top_score < 0.45 (or NULL),
    /// grouped by normalized text, needing at least `min_count` occurrences.
    pub async fn gaps(&self, days: i64, min_count: i64) -> anyhow::Result<Vec<GapTheme>> {
        let rows: Vec<(String, i64, Option<f64>, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
            "SELECT lower(regexp_replace(trim(query), '\\s+', ' ', 'g')) AS theme,
                    count(*) AS cnt, avg(top_score) AS avg_score, max(created_at) AS last_seen
               FROM rag_queries
              WHERE created_at > now() - ($1 || ' days')::interval
                AND (top_score IS NULL OR top_score < 0.45)
              GROUP BY theme
             HAVING count(*) >= $2
              ORDER BY cnt DESC, last_seen DESC",
        )
        .bind(days.to_string())
        .bind(min_count)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(theme, count, avg_score, last_seen)| GapTheme { theme, count, avg_score, last_seen })
            .collect())
    }
}
```

Export `GapTheme` from `corpus/mod.rs` (`pub use store::{... , GapTheme};`).

- [ ] **Step 4: Run tests to verify they pass**

Run: `MX_RAG_TEST_DATABASE_URL=... cargo test -p mx-lib corpus::store`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add crates/mx-lib/src/corpus/store.rs crates/mx-lib/src/corpus/mod.rs
git commit -m "feat(corpus): gap-theme aggregation over weak rag queries"
```

---

### Task 3: `mx rag gaps` CLI

**Acceptance Criteria (observable):**
- `cargo run -p mx-cli -- rag gaps --help` exits 0 documenting `--days` (default 30) and `--min-count` (default 2).
- Against the test DB with seeded weak queries, `MX_RAG_FALLBACK_DATABASE_URL=postgres://postgres@localhost:55433/mx_rag cargo run -p mx-cli -- rag gaps --min-count 1` exits 0 and prints at least one theme row with count and average score.
- `mx rag status` output now includes the `logged_queries` count line.

**Verify via:** cli

**Files:**
- Modify: `crates/mx-cli/src/commands/rag.rs`

- [ ] **Step 1: Add the subcommand**

To `RagSubcommand` add:

```rust
    /// Mine research-gap themes from weak-scoring rag queries
    Gaps {
        /// Look-back window in days
        #[arg(long, default_value_t = 30)]
        days: i64,
        /// Minimum occurrences for a theme to report
        #[arg(long, default_value_t = 2)]
        min_count: i64,
    },
```

Match arm: `RagSubcommand::Gaps { days, min_count } => self.gaps(*days, *min_count).await,` and implement:

```rust
    async fn gaps(&self, days: i64, min_count: i64) -> Result<()> {
        let cfg = RagConfig::load();
        let store = CorpusStore::connect(&cfg).await?;
        let gaps = store.gaps(days, min_count).await?;
        if gaps.is_empty() {
            println!("{} No gap themes in the last {} days (min count {}).", style("✓").green().bold(), days, min_count);
            return Ok(());
        }
        println!("{}", style(format!("Research gaps — last {} days", days)).bold());
        for g in gaps {
            let avg = g.avg_score.map(|s| format!("{:.2}", s)).unwrap_or_else(|| "n/a".into());
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
```

Import `GapTheme` implicitly via `store.gaps` (add `use mx_lib::corpus::{CorpusStore, RagConfig};` already present). In the `status()` printout add after the model line:

```rust
        println!("  {} Logged queries: {}", style("•").dim(), st["logged_queries"]);
```

- [ ] **Step 2: Verify live**

```bash
cargo run -p mx-cli -- rag gaps --help
MX_RAG_FALLBACK_DATABASE_URL=postgres://postgres@localhost:55433/mx_rag cargo run -p mx-cli -- rag gaps --min-count 1
MX_RAG_FALLBACK_DATABASE_URL=postgres://postgres@localhost:55433/mx_rag cargo run -p mx-cli -- rag status | grep -i "logged queries"
```
Expected: help exits 0; gaps prints seeded themes from Task 2's test data (re-seed via psql if the test cleaned up: `docker exec mx-rag-test psql -U postgres -d mx_rag -c "INSERT INTO rag_queries (query, tool, top_score, mode) VALUES ('graphql federation','manual',0.2,'hybrid'),('graphql federation','manual',0.3,'hybrid')"`); status shows the count.

- [ ] **Step 3: Commit**

```bash
git add crates/mx-cli/src/commands/rag.rs
git commit -m "feat(mx-cli): mx rag gaps — mine weak-query research themes"
```

---

### Task 4: Backlog, research log, provenance conventions

**Acceptance Criteria (observable):**
- `docs/development/RESEARCH_BACKLOG.md` exists with usage header + 3 seeded unchecked topics; `docs/development/RESEARCH_LOG.md` exists with header + table schema.
- `docs/development/INDEX.md` contains a "Research Provenance" section documenting `provenance`/`researched`/`sources` frontmatter and the `## Synthesis (inferred)` rule.
- `cargo run -p mx-cli -- rag ingest --dry-run` still exits 0 with 0 warnings (both new files carry valid frontmatter; category `process`).

**Verify via:** cli

**Files:**
- Create: `docs/development/RESEARCH_BACKLOG.md`
- Create: `docs/development/RESEARCH_LOG.md`
- Modify: `docs/development/INDEX.md`

- [ ] **Step 1: Create RESEARCH_BACKLOG.md**

```markdown
---
title: Research Backlog
category: process
complexity: intermediate
use_cases:
  - queueing topics for technique research
  - autonomous research topic selection
summary: Topic queue for the technique-research skill; humans and agents append, the scheduler pops the top unchecked entry.
---

# Research Backlog

Topics awaiting a research pass. The `technique-research` skill's autonomous mode pops the **top unchecked** entry. Anyone (human or agent) may append; the techniques skill appends here when `rag_context` returns weak results.

Format: `- [ ] <topic> — <one-line why> (added YYYY-MM-DD by <who>)`

## Queue

- [ ] Rust async cancellation and graceful shutdown patterns — corpus covers concurrency primitives but not structured cancellation (added 2026-07-18 by design)
- [ ] Supply-chain security for dependency ecosystems (cargo/npm) — security category is thin on tooling practice (added 2026-07-18 by design)
- [ ] Local-first sync engines (CRDTs in practice) — emerging pattern, no coverage (added 2026-07-18 by design)
```

- [ ] **Step 2: Create RESEARCH_LOG.md**

```markdown
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
```

- [ ] **Step 3: Add the provenance section to INDEX.md**

Append after the existing "Frontmatter Authoring" section:

```markdown
## Research Provenance

Docs written or updated by the `technique-research` skill carry additional frontmatter:

- `provenance: researched` (vs `curated` hand-written originals)
- `researched: YYYY-MM-DD` — refreshed on every research pass; the autonomous staleness sweep picks the oldest
- `sources:` — list of URLs backing the doc's sourced claims

Sourced claims cite inline (`[1]`-style keyed to `sources`). The agent's own contributions appear ONLY under `## Synthesis (inferred)` headings — never blended into sourced text. An improvement pass updates sections and appends sources; it does not silently delete prior content.
```

- [ ] **Step 4: Verify and commit**

```bash
cargo run -p mx-cli -- rag ingest --dry-run
```
Expected: `58 docs, ..., 0 warnings` (56 + 2 new).

```bash
git add docs/development/RESEARCH_BACKLOG.md docs/development/RESEARCH_LOG.md docs/development/INDEX.md
git commit -m "docs(development): research backlog, log, and provenance conventions"
```

---

### Task 5: The `technique-research` skill + provider registry

**Acceptance Criteria (observable):**
- `~/.claude/skills/technique-research/SKILL.md` exists with frontmatter `name: technique-research` and a trigger-rich description; `references/source-providers.md` exists with the provider contract, one `active` web provider, and five `planned` entries (context7, cross-model, hq-corpus, rss, medium-api).
- Repo snapshots `skills/technique-research/SKILL.md` + `skills/technique-research/references/source-providers.md` are byte-identical to the home copies (diff exits 0).

**Verify via:** cli

**Files:**
- Create: `~/.claude/skills/technique-research/SKILL.md`
- Create: `~/.claude/skills/technique-research/references/source-providers.md`
- Create: `skills/technique-research/SKILL.md` (snapshot)
- Create: `skills/technique-research/references/source-providers.md` (snapshot)

- [ ] **Step 1: Write SKILL.md**

```markdown
---
name: technique-research
description: 'Research mode for the self-growing techniques library. Takes a topic (or picks one autonomously), checks corpus coverage, researches external sources, then improves or authors a technique doc in mech-crate docs/development — PR-gated. Use when the user says /technique-research <topic>, "research <topic> for the library", "grow the techniques library", or a scheduled autonomous run fires. Also invoked when the techniques skill finds a gap worth researching.'
---

# Technique Research — Grow the Library

Research a topic and fold what is learned into the techniques corpus as a reviewed PR. The corpus only learns what gets merged.

**Announce at start:** "Running technique research on: <topic>" (or "…in autonomous mode").

## Phase 0 — Locate repo & pick the topic

Repo resolution order: `$MECH_CRATE_ROOT` → contents of `~/.mech-crate/config/source-root` → `~/dev/dev916/mech-crate`. All file edits and git operations happen there. Work on a fresh branch `research/<slug>` cut from the default branch (pull first).

Directed mode: topic given in the invocation. Autonomous mode ladder (stop at the first hit):
1. Top unchecked entry in `docs/development/RESEARCH_BACKLOG.md`.
2. Stalest doc: oldest frontmatter `researched:` date — docs lacking the key rank stalest; consider categories security, concurrency, api-design, database, patterns first.
3. Top theme from `mx rag gaps --days 30 --min-count 2`.
4. Tech-radar sweep: research "notable shifts in software engineering practice, last 6 months", append 3–5 proposed topics to the backlog with rationale, take the first.

## Phase 1 — Assess coverage

Run `mcp__mx__rag_search` (and `mcp__mx__rag_find_related`) on the topic. Verdict:
- **NEW** — nothing meaningful → author a new doc.
- **IMPROVE** — a doc covers it → list its concrete gaps vs current practice (missing techniques, stale APIs, absent tradeoffs).
- **FRESH** — covered and current → append a no-op row to RESEARCH_LOG.md, report, STOP. No PR.

If the corpus is offline, proceed as NEW but flag "dedup skipped — corpus offline" in the PR body.

## Phase 2 — Research via providers

Read `references/source-providers.md`. Run every provider whose **Status** is `active` and whose **Use when** matches the topic. v1: the `web` provider (invoke the deep-research skill with the topic + what Phase 1 says is missing). Collect claims with citations and confidence; reconcile disagreements explicitly. If sources are too thin to support a doc, log "insufficient sources" to RESEARCH_LOG.md and STOP without a PR.

## Phase 3 — Author

- NEW: full doc in `docs/development/<slug>.md` with standard frontmatter (title/category/languages/complexity/use_cases/summary per INDEX.md) PLUS `provenance: researched`, `researched: <today>`, `sources:` list.
- IMPROVE: surgical edits — update stale sections, append new ones. Never delete prior content without stating why in the PR body. Update `researched:`, append new `sources:`.
- Your own contributions (patterns you infer, connections you draw) go ONLY under `## Synthesis (inferred)` headings. Every other claim must trace to a citation.

## Phase 4 — Verify

- `mx rag ingest --dry-run` → must report 0 warnings.
- Code examples: type-check/compile where a toolchain is available; otherwise mark examples as illustrative.
- Re-read the doc: every claim is cited or sits under Synthesis (inferred).

## Phase 5 — Ship

1. Commit on `research/<slug>`; push; open a PR with: coverage verdict, what changed & why, full source list, inventory of inferred sections.
2. Check off the backlog entry (if any); append a row to RESEARCH_LOG.md (date, topic, verdict, source count, PR link) in the same PR.
3. Report the PR URL. Do NOT merge — a human merges. Post-merge, the next `mx rag ingest` (manual or a later run's Phase 4 dry-run reminder) picks up the delta; suggest the user run it after merging.

## Schedule management

The autonomous run is a local Claude Code cron job. On request:
- List: CronList. Pause/off: CronDelete the technique-research job. Retime: delete + CronCreate with the new cadence, prompt: "Invoke the technique-research skill in autonomous mode."

## Guardrails

- One topic per run. deep-research at most once per run.
- Never block or fail on corpus unavailability; never merge your own PR; never edit docs outside docs/development + the backlog/log.
```

- [ ] **Step 2: Write references/source-providers.md**

```markdown
# Source Providers — Registry & Contract

Phase 2 of technique-research runs every **active** provider whose **Use when** matches the topic. To add a provider: append an entry satisfying the contract, flip Status to active. Log new provider ideas as GitHub issues on Dev916/mech-crate.

## Contract

Every entry defines:
- **Status:** active | planned
- **Use when:** topic conditions that make this provider worth querying
- **Query:** the exact tool/command/skill an agent runs
- **Returns:** claims with citations (URL or source id), each tagged with confidence (high/medium/low)
- **Cost note:** token/API cost characteristics

## Providers

### web
- **Status:** active
- **Use when:** always (default provider)
- **Query:** invoke the `deep-research` skill with the topic plus Phase 1's gap list as the research question
- **Returns:** cited, adversarially-verified claims from web sources (docs, papers, engineering blogs)
- **Cost note:** token-heavy (multi-agent fan-out); at most one invocation per run

### context7
- **Status:** planned
- **Use when:** topic names a specific library/framework/SDK
- **Query:** `mcp__plugin_context7_context7__resolve-library-id` then `query-docs` for current API documentation
- **Returns:** official-doc claims, citation = library id + doc section, confidence high
- **Cost note:** cheap per query

### cross-model
- **Status:** planned (needs API key config — see GitHub issue)
- **Use when:** contested/judgment-heavy topics where a second model's blind spots differ
- **Query:** OpenAI-compatible chat call (key/base_url from env) asking the model for its approach + reasoning
- **Returns:** claims cited as "model consultation: <model>", confidence medium; must be corroborated or marked inferred
- **Cost note:** one API call per consulted model

### hq-corpus
- **Status:** planned
- **Use when:** topic may overlap internal knowledge (business/ops/prior research)
- **Query:** `mcp__hq__hq_corpus_search` with the topic
- **Returns:** internal-doc claims, citation = corpus doc path/id
- **Cost note:** cheap

### rss
- **Status:** planned (see GitHub issue)
- **Use when:** freshness-driven topics; curated feed list TBD in the issue
- **Query:** fetch + parse configured feeds, filter by topic keywords
- **Returns:** article claims with URLs, confidence per source reputation
- **Cost note:** cheap fetches, noisy signal

### medium-api
- **Status:** planned (see GitHub issue — https://mediumapi.com/)
- **Use when:** practitioner-experience topics where engineering blogs dominate
- **Query:** unofficial Medium API (key TBD) topic/tag search, fetch top articles
- **Returns:** article claims with URLs, confidence medium
- **Cost note:** third-party API key + rate limits
```

- [ ] **Step 3: Snapshot into the repo, verify, commit**

```bash
mkdir -p skills/technique-research/references
cp ~/.claude/skills/technique-research/SKILL.md skills/technique-research/SKILL.md
cp ~/.claude/skills/technique-research/references/source-providers.md skills/technique-research/references/source-providers.md
diff ~/.claude/skills/technique-research/SKILL.md skills/technique-research/SKILL.md && diff ~/.claude/skills/technique-research/references/source-providers.md skills/technique-research/references/source-providers.md
git add skills/technique-research
git commit -m "feat(skills): technique-research skill with source-provider registry"
```
Expected: both diffs exit 0.

---

### Task 6: Techniques skill feeds the backlog

**Acceptance Criteria (observable):**
- `~/.claude/skills/techniques/SKILL.md` core loop now instructs: when `rag_context` returns nothing relevant (or the weak/lexical-only note), append the topic to `docs/development/RESEARCH_BACKLOG.md` (one line, dated, attributed) when the mech-crate repo is locatable — and continue working without blocking.
- `grep -c "RESEARCH_BACKLOG" ~/.claude/skills/techniques/SKILL.md` prints ≥ 1; repo snapshot `skills/techniques/SKILL.md` matches the home copy (diff exits 0).

**Verify via:** cli

**Files:**
- Modify: `~/.claude/skills/techniques/SKILL.md`
- Modify: `skills/techniques/SKILL.md` (snapshot)

- [ ] **Step 1: Edit the core loop**

In the `## Core loop` section, replace item 4's text:

```markdown
4. **Never block on the corpus.** If tools return the offline message or nothing relevant: note it in one line and proceed with your own judgment. `mcp__mx__rag_health` diagnoses; `mx rag ingest` re-ingests; do NOT stop work to repair the corpus unless the user asks.
```

with:

```markdown
4. **Never block on the corpus.** If tools return the offline message or nothing relevant: note it in one line and proceed with your own judgment. `mcp__mx__rag_health` diagnoses; `mx rag ingest` re-ingests; do NOT stop work to repair the corpus unless the user asks.
5. **Feed the backlog when you find a gap.** If `rag_context` returned nothing relevant (or only weak, off-topic chunks) for a topic worth knowing, append one line to the mech-crate research backlog so research mode picks it up later: `- [ ] <topic> — <one-line why> (added YYYY-MM-DD by techniques-skill)` in `docs/development/RESEARCH_BACKLOG.md` (repo located via $MECH_CRATE_ROOT, then ~/.mech-crate/config/source-root, then ~/dev/dev916/mech-crate). If the repo isn't reachable, skip silently. Then continue your task — never block on this.
```

- [ ] **Step 2: Snapshot, verify, commit**

```bash
cp ~/.claude/skills/techniques/SKILL.md skills/techniques/SKILL.md
grep -c "RESEARCH_BACKLOG" ~/.claude/skills/techniques/SKILL.md
diff ~/.claude/skills/techniques/SKILL.md skills/techniques/SKILL.md
git add skills/techniques/SKILL.md
git commit -m "feat(skills): techniques skill feeds research backlog on corpus gaps"
```
Expected: grep ≥ 1, diff exit 0.

---

### Task 7: Weekly cron job

**Acceptance Criteria (observable):**
- A Claude Code cron job exists (visible via CronList) scheduled weekly (Monday 09:00 local) whose prompt invokes the technique-research skill in autonomous mode.
- The job id/name and the pause/retime commands are recorded in `skills/technique-research/SKILL.md`'s Schedule management section (already written; append the actual job id as a comment line).

**Verify via:** cli

**Files:**
- Modify: `~/.claude/skills/technique-research/SKILL.md` + repo snapshot (job id note)

- [ ] **Step 1: Create the job**

Load the cron tools via ToolSearch (`select:CronCreate,CronList`). Create:
- schedule: weekly, Monday 09:00 local (cron expression `0 9 * * 1`)
- prompt: `Invoke the technique-research skill (Skill tool, skill: technique-research) in autonomous mode: no topic given — follow its Phase 0 autonomous ladder. Follow the skill exactly.`

- [ ] **Step 2: Verify + record**

CronList shows the job. Append to the Schedule management section of SKILL.md (home + snapshot):

```markdown
<!-- active job: <id-from-croncreate>, cadence 0 9 * * 1 (Mon 09:00) -->
```

```bash
cp ~/.claude/skills/technique-research/SKILL.md skills/technique-research/SKILL.md
git add skills/technique-research/SKILL.md
git commit -m "chore(research): schedule weekly autonomous technique-research run"
```

---

### Task 8: File the follow-up GitHub issues

**Acceptance Criteria (observable):**
- `gh issue list --repo Dev916/mech-crate` shows 7 new open issues titled: "Cloud routine for technique research (always-on scheduling)", "Research provider: cross-model consultation", "Research provider: RSS feeds", "Research provider: Medium (mediumapi.com)", "mx rag research: native Rust research orchestrator (Approach B)", "Multi-agent Workflow harness for research (Approach C)", "Query-gap mining v2: embedding-based theme clustering".

**Verify via:** cli

- [ ] **Step 1: Create the issues**

For each, `gh issue create --repo Dev916/mech-crate --title "<title>" --body "<2-5 sentence body referencing docs/superpowers/specs/2026-07-18-self-growing-techniques-design.md and, for providers, the contract in skills/technique-research/references/source-providers.md>"`. The cloud-routine issue body must include the provisioning checklist: repo clone, mx toolchain build, Neon connection string, OPENAI_API_KEY, GitHub credentials for PR creation.

- [ ] **Step 2: Verify + commit nothing**

`gh issue list --repo Dev916/mech-crate --limit 10` shows all 7. No repo files change (no commit).

---

### Task 9: Supervised live verification of research mode

**Acceptance Criteria (observable):**
- **Directed run:** invoking the technique-research skill with topic "Rust async cancellation and graceful shutdown patterns" produces: a `research/` branch, a PR on Dev916/mech-crate whose body lists verdict/sources/inferred inventory, a doc under docs/development with `provenance: researched` frontmatter, `mx rag ingest --dry-run` at 0 warnings on that branch, the backlog entry checked off, and a new RESEARCH_LOG.md row — all verifiable via `gh pr view` + file inspection.
- **FRESH no-op:** running the skill on "Rust atomics memory ordering (Acquire/Release/SeqCst) selection" (squarely covered by appendix-rust-concurrency.md, complexity expert) produces NO new PR and appends a FRESH row to RESEARCH_LOG.md.
- **Autonomous selection dry check:** with the backlog non-empty, the skill's Phase 0 (asked to report its selection and stop) picks the top unchecked backlog entry.

**Verify via:** cli

- [ ] **Step 1: Directed run**

Dispatch/execute the skill exactly as written with the topic above (it is backlog item #1, so this also exercises check-off). Capture the PR URL. ONE override for this verification only (the backlog/log files exist only on the feature branch, not on main yet): cut `research/<slug>` from `feat/technique-research-mode` instead of the default branch, and open the PR with `--base feat/technique-research-mode` so the research diff stays isolated.

- [ ] **Step 2: FRESH no-op run**

Run the skill on "Rust atomics memory ordering (Acquire/Release/SeqCst) selection"; confirm no PR is created and the log row says FRESH.

- [ ] **Step 3: Autonomous selection dry check**

Invoke the skill in autonomous mode with the explicit instruction "run Phase 0 only, report the selected topic and stop." Expect it to name the (new) top unchecked backlog entry.

- [ ] **Step 4: Record + commit**

The directed run's log/backlog changes live in ITS research PR. The FRESH row from Step 2 is committed on the feature branch:

```bash
git add docs/development/RESEARCH_LOG.md
git commit -m "docs(research): log FRESH no-op verification run"
```
