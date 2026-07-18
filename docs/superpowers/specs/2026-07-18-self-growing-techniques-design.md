# Self-Growing Techniques Library (Research Mode) — Design Spec

**Date:** 2026-07-18
**Status:** Approved
**Repo:** mech-crate
**Builds on:** `2026-07-15-techniques-rag-design.md` (the live pgvector techniques corpus)

## Overview

Turn the techniques corpus into a self-improving library. A **research mode** takes a topic, assesses existing coverage, researches external sources, and either improves an existing technique doc or authors a new one — including clearly-labeled original synthesis — so the corpus gets smarter over time across performance, security, maintainability, and capability domains.

Research judgment, source evaluation, and authoring are agent work, so the engine is a **skill** (`technique-research`); Rust gains only what agents cannot persist themselves (query logging for gap mining). All growth is **PR-gated**: the corpus remains merged-truth only.

## Decisions (settled during brainstorming)

| Decision | Choice |
|---|---|
| Trigger | Both: on-demand `/technique-research <topic>` AND a scheduled autonomous run. Schedule must be user-controllable (retime, pause, off). |
| Quality gate | PR-gated. Each run commits to a `research/<slug>` branch and opens a PR; `mx rag ingest` picks up the delta only after merge. |
| Sources | Provider registry with a documented contract. v1 active provider: **web** (wraps the `deep-research` skill). Planned, contract-conforming: `context7`, `cross-model` (GPT/Gemini), `hq-corpus`, `rss`, `medium-api` (https://mediumapi.com/). Future providers logged as GitHub issues. |
| Autonomous topic selection | Ladder: backlog file → staleness sweep → query-gap mining → tech-radar discovery. All four in scope (gap mining requires Rust query logging). |
| Runtime | Local Claude Code cron now; cloud routine later (issue filed with provisioning checklist). |
| Approach | A (skill-centric). Approaches B (Rust orchestrator) and C (dedicated Workflow harness) filed as issues for later. |

## Architecture

```
~/.claude/skills/technique-research/
  SKILL.md                        — the pipeline (phases 0-5 below)
  references/source-providers.md  — provider registry + contract
(repo snapshots under skills/technique-research/ for versioning)

docs/development/
  RESEARCH_BACKLOG.md             — topic queue (humans + agents append)
  RESEARCH_LOG.md                 — append-only run audit (topic, verdict, PR link)
  <technique docs>                — gain provenance frontmatter when researched

crates/mx-lib/
  migrations/0002_rag_queries.sql — query log table
  src/corpus/store.rs             — fire-and-forget query logging in search();
                                    gaps aggregation; logged_queries in status()
crates/mx-cli/
  src/commands/rag.rs             — new `mx rag gaps` subcommand

Local cron job (weekly) → invokes the skill in autonomous mode
```

## The pipeline (skill phases)

**Phase 0 — Topic intake.**
The skill first locates the mech-crate repo (in order: `MECH_CRATE_ROOT` env, the recorded source root at `~/.mech-crate/config/source-root`, `~/dev/dev916/mech-crate`) — all file edits, backlog/log updates, and git operations happen there regardless of which project the session is in.

Directed mode: topic from invocation args. Autonomous ladder:
1. Top unchecked entry in `docs/development/RESEARCH_BACKLOG.md`.
2. Else stalest doc by frontmatter `researched:` (docs without the key rank stalest of all within high-value categories: security, concurrency, api-design, database, patterns first).
3. Else top theme from `mx rag gaps`.
4. Else tech-radar sweep: research "what changed in software engineering recently", append 3–5 proposed topics to the backlog, take #1.

**Phase 1 — Corpus assessment.**
`rag_search` + `rag_find_related` on the topic. Verdict:
- **NEW** — no meaningful coverage → author a new doc.
- **IMPROVE** — a doc covers it; enumerate its gaps vs current practice → surgical update.
- **FRESH** — covered and current → log "no action" to RESEARCH_LOG.md and stop (no PR).

**Phase 2 — Research.**
Run every *active* provider whose *Use when* matches the topic (v1: web via the `deep-research` skill — fan-out search, source fetch, adversarial claim verification, cited synthesis). Providers return claims + citations + confidence; merge and reconcile disagreements.

**Phase 3 — Author.**
- NEW: full doc in `docs/development` with standard frontmatter + provenance fields.
- IMPROVE: update stale sections, append new sections; never delete prior content without stating why in the PR body; update `researched:` and append to `sources:`.
- The agent's own contributions live ONLY under `## Synthesis (inferred)` headings — never blended into sourced text.

**Phase 4 — Verify.**
`mx rag ingest --dry-run` reports 0 warnings; code examples type-check/compile where a toolchain is available; every claim traces to a Phase-2 citation or sits in an inferred section.

**Phase 5 — Ship.**
Branch `research/<slug>` → PR containing: coverage verdict, what changed and why, full source list, inventory of inferred content. Check off the backlog entry; append the run to RESEARCH_LOG.md. Post-merge, any `mx rag ingest` (manual or the next run's) ingests the delta.

## Provider registry contract

`references/source-providers.md` entries:

```markdown
### <provider-name>
- **Status:** active | planned
- **Use when:** <topic conditions>
- **Query:** <exact tool/command/skill an agent runs>
- **Returns:** claims with citations, each tagged with confidence
- **Cost note:** <tokens/API characteristics>
```

v1 entries: `web` (active), `context7`, `cross-model`, `hq-corpus`, `rss`, `medium-api` (planned). Adding a provider = add an entry, flip status; Phase 2 automatically includes it.

## Doc conventions (provenance)

```yaml
provenance: researched      # vs curated (hand-written originals)
researched: 2026-07-18      # refreshed on every research pass; staleness key
sources:
  - https://...
```

Sourced claims cite inline (`[1]`-style keyed to `sources`). Inferred content is quarantined under `## Synthesis (inferred)`. The unknown-key-tolerant frontmatter parser already accepts these fields; they are not stored in the DB (staleness sweeps read files directly). `docs/development/INDEX.md`'s authoring guide gains a section documenting these conventions.

## Rust additions

**Migration `0002_rag_queries.sql`:**

```sql
CREATE TABLE IF NOT EXISTS rag_queries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    query TEXT NOT NULL,
    tool TEXT NOT NULL,              -- rag_context | rag_search | ...
    category TEXT,
    language TEXT,
    top_score DOUBLE PRECISION,      -- NULL when zero results
    mode TEXT NOT NULL,              -- hybrid | trigram_only
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS rag_queries_created_idx ON rag_queries (created_at);
```

**Store:** `search()` logs each query fire-and-forget (spawned task; a logging failure NEVER fails or slows the search — errors are traced and dropped). `status()` gains `logged_queries` count. New `gaps(days, min_count)` aggregation: queries with `top_score < 0.45 OR top_score IS NULL`, grouped by normalized query text (lowercase, whitespace-collapsed), returning theme, count, avg score, last seen — ordered by count desc.

**CLI:** `mx rag gaps [--days 30] [--min-count 2]` prints the gap report. The `tool` attribution requires the MCP handlers to pass their tool name into search — a small `TechQuery.tool` field.

## Scheduling

Local Claude Code cron job, default weekly (Monday 09:00 local), prompt: invoke the `technique-research` skill in autonomous mode. The skill doc includes exact commands to pause (delete job), retime (recreate), and inspect the schedule so any session can manage it on request. Cloud routine deferred (issue filed with provisioning checklist: repo clone, mx toolchain, Neon/OpenAI/GitHub secrets).

## Error handling

| Condition | Behavior |
|---|---|
| Corpus offline during assessment | Proceed as NEW-topic research but flag in PR that dedup was skipped; never block research on the corpus |
| deep-research yields thin/conflicting results | Lower confidence, note it in the PR; below a usable threshold → log "insufficient sources" to RESEARCH_LOG.md, no PR |
| Backlog empty + all docs fresh + no gaps | Tech-radar sweep; if even that proposes nothing, log a no-op run |
| Query logging fails | Trace + drop; search results unaffected |
| Scheduled run on a machine that's off | Nothing happens; next scheduled fire proceeds normally (cron is stateless) |
| PR conflicts with a since-merged change | Standard git conflict flow in the research branch; the skill rebases before opening the PR |

## Testing

- **Rust:** unit/integration (existing test-DB pattern, port 55433): insert-on-search, NULL score on empty results, never-fail logging semantics, gaps aggregation correctness (threshold, grouping, ordering), `logged_queries` in status.
- **Skill, supervised live runs (devloop cli criteria):**
  - Directed run on a real topic produces a `research/<slug>` PR with citations, provenance frontmatter, dry-run 0 warnings.
  - FRESH verdict produces no PR (no-op path proven, logged).
  - Autonomous run against a seeded backlog pops the top entry and completes the same flow.
- **Cron:** job created, visible, and deletable; its prompt invokes the skill.

## Cost guardrails

One topic per scheduled run; deep-research invoked at most once per run; provider fan-out limited by *Use when* matching; tech-radar sweep only when the rest of the ladder is empty.

## GitHub issues to file during implementation

1. Cloud routine for research mode (provisioning checklist).
2. Provider: cross-model consultation (key config design).
3. Provider: RSS feeds.
4. Provider: Medium via mediumapi.com.
5. Approach B: native `mx rag research` orchestrator.
6. Approach C: dedicated multi-agent Workflow harness.
7. Query-gap mining v2: embedding-based theme clustering.

## Out of scope

- Cloud/always-on scheduling (issue #1 above).
- Non-web providers beyond registry stubs.
- Automatic PR merging — a human always merges.
- Corpus schema changes for provenance (files are the source of truth for those fields).
