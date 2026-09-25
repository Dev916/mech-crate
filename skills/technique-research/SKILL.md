---
name: technique-research
description: 'Research mode for the self-growing techniques library. Takes a topic (or picks one autonomously), checks corpus coverage, researches external sources, then improves or authors a technique doc in mech-crate docs/development — PR-gated. Use when the user says /technique-research <topic>, "research <topic> for the library", "grow the techniques library", or a scheduled autonomous run fires. Also invoked when the techniques skill finds a gap worth researching.'
---

# Technique Research — Grow the Library

Research a topic and fold what is learned into the techniques corpus as a reviewed PR. The corpus only learns what gets merged.

**Announce at start:** "Running technique research on: <topic>" (or "…in autonomous mode").

## Phase 0 — Locate repo & pick the topic

Repo resolution order: `$MECH_CRATE_ROOT` → contents of `~/.mech-crate/config/source-root` → `~/dev/dev916/mech-crate`. All file edits and git operations happen there. Work on a fresh branch `research/<slug>` cut from the default branch (pull first).

Repo state: `git fetch origin main` first and cut `research/<slug>` from `origin/main`. Never assume `main` is checked out in the working tree — the headless weekly run works in a dedicated worktree (`~/.mech-crate/research-worktree`, branch `research-bot-main`) so the owner's checkout is never touched.

Directed mode: topic given in the invocation.

Autonomous mode, step 0 — **Hacker News trend pulse (every autonomous run, before the ladder).** The corpus should learn what practitioners are arguing about this week, not only what is already in the backlog. Run the three Algolia queries in `references/source-providers.md` (hackernews → "Trend pulse"): top stories of the last 7 days, Ask HN / Show HN of the last 7 days, and one keyword query per corpus category that is thin (`mcp__mx__rag_health` → `by_category`). Screen the titles for software-engineering relevance (languages, runtimes, databases, infra, security, agents/LLM tooling, protocols); drop product launches, politics and general news. For each surviving story that suggests a technique, check corpus coverage with `mcp__mx__rag_search` and skip anything already covered. Append up to 3 new unchecked entries to `docs/development/RESEARCH_BACKLOG.md`, each on one line: `- [ ] <topic as a technique question> — HN <points> pts / <comments> c, <YYYY-MM-DD>, <HN item URL>; why: <one clause> (added <today> by technique-research)`. Do not add an entry the backlog already holds in substance. These entries ship in the same PR as the run's doc (or in a backlog-only PR if the run ends FRESH). Budget: at most 10 minutes and no WebFetch beyond the linked article of a story you are about to add.

Autonomous mode ladder (stop at the first hit):
1. Top unchecked entry in `docs/development/RESEARCH_BACKLOG.md` (owner-added entries sit above pulse-added ones, so owner priorities drain first).
2. Stalest doc: oldest frontmatter `researched:` date — docs lacking the key rank stalest; consider categories security, concurrency, api-design, database, patterns first.
3. Top theme from `mx rag gaps --days 30 --min-count 2`.
4. Tech-radar sweep: research "notable shifts in software engineering practice, last 6 months" (web + the HN pulse results above), append 3–5 proposed topics to the backlog with rationale, take the first.

## Phase 1 — Assess coverage

Run `mcp__mx__rag_search` (and `mcp__mx__rag_find_related`) on the topic. Verdict:
- **NEW** — nothing meaningful → author a new doc.
- **IMPROVE** — a doc covers it → list its concrete gaps vs current practice (missing techniques, stale APIs, absent tradeoffs).
- **FRESH** — covered and current → append a no-op row to RESEARCH_LOG.md, report, STOP. No PR.

If the corpus is offline, proceed as NEW but flag "dedup skipped — corpus offline" in the PR body.

## Phase 2 — Research via providers

Read `references/source-providers.md`. Run every provider whose **Status** is `active` and whose **Use when** matches the topic — v1 actives: `web` (always) and `hackernews` (always: it is free, needs no auth, and is where practitioners argue about the topic), plus `x` for innovation/discovery topics. Collect claims with citations and confidence; reconcile disagreements explicitly. Discovery-grade claims (x, hackernews, reddit) must be corroborated by a primary source before being stated as fact — otherwise they go under Synthesis (inferred) or are dropped. If sources are too thin to support a doc, log "insufficient sources" to RESEARCH_LOG.md and STOP without a PR.

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

The autonomous run is a **launchd user agent** (`~/Library/LaunchAgents/com.mechcrate.research-weekly.plist`, label `com.mechcrate.research-weekly`, Mondays 09:03 local) running `scripts/research-weekly.sh` from the mech-crate repo — headless `claude -p` in a dedicated worktree, logs to `~/.mech-crate/research-cron.log`. It is a LaunchAgent, not a crontab entry, because cron jobs on macOS cannot read the login Keychain where Claude Code keeps its OAuth credentials (every cron run from 2026-07-20 to 2026-09-14 failed with "Not logged in"), and because launchd runs a missed calendar slot after the Mac wakes, where cron silently skips it. On request:
- Inspect: `launchctl print gui/$(id -u)/com.mechcrate.research-weekly | head -20` (state, last exit code, next run)
- Run now: `launchctl kickstart gui/$(id -u)/com.mechcrate.research-weekly`
- Pause/off: `launchctl bootout gui/$(id -u)/com.mechcrate.research-weekly`; resume: `launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/com.mechcrate.research-weekly.plist`
- Retime: edit `StartCalendarInterval` in the plist, then bootout + bootstrap
- Logs/last run: `tail ~/.mech-crate/research-cron.log`
- Template and installer: `scripts/launchd/com.mechcrate.research-weekly.plist` and `scripts/launchd/install-research-weekly.sh` in the repo. The agent runs from a bot-owned clone (`~/.mech-crate/research-source`, fast-forwarded to `origin/main` at every launch), and `~/.claude/skills/technique-research` is a symlink into that clone, so a change to this skill reaches the bot the Monday after it merges with no manual copy.

(Claude Code's in-session CronCreate is NOT used — those jobs are session-only and expire after 7 days. The old `3 9 * * 1` crontab line must not be re-added.)

## Guardrails

- One topic per run. deep-research at most once per run.
- Never block or fail on corpus unavailability; never merge your own PR; never edit docs outside docs/development + the backlog/log.
