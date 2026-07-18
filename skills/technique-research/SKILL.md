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

Read `references/source-providers.md`. Run every provider whose **Status** is `active` and whose **Use when** matches the topic — v1 actives: `web` (deep-research skill, always), plus `x` and `hackernews` for innovation/discovery topics. Collect claims with citations and confidence; reconcile disagreements explicitly. Discovery-grade claims (x, hackernews, reddit) must be corroborated by a primary source before being stated as fact — otherwise they go under Synthesis (inferred) or are dropped. If sources are too thin to support a doc, log "insufficient sources" to RESEARCH_LOG.md and STOP without a PR.

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

<!-- active job: 27bd29fa, cadence 0 9 * * 1 (Mon 09:00) -->

## Guardrails

- One topic per run. deep-research at most once per run.
- Never block or fail on corpus unavailability; never merge your own PR; never edit docs outside docs/development + the backlog/log.
