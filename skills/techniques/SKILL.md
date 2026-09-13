---
name: techniques
description: 'Consult the mx techniques corpus (RAG over mech-crate docs/development: theory, patterns, architecture, concurrency, API design, databases, Docker, FSM/FRP, security) when deciding HOW to implement something. Use when choosing between approaches/architectures/patterns, starting feature work in a covered domain, writing an implementation plan, or the user asks "what is the best way to build X". Triggers: /techniques, "check the techniques library", "what patterns apply".'
---

# Techniques — Corpus-Backed Building Patterns

Query the mx MCP techniques corpus before committing to an implementation approach. The corpus holds curated engineering technique docs (mech-crate `docs/development`), chunked and searchable by meaning + metadata.

**Announce at start:** "Consulting the techniques corpus."

## Core loop

1. **Describe the work.** Call `mcp__mx__rag_context` with `working_on` = 1–2 sentences about the current task. Add `language` (rust/typescript/php/python/...) and `category` (theory | patterns | architecture | concurrency | api-design | database | frontend | docker | infra | shell | blockchain | ml | security | process | repos) when obvious. `repos` holds profiles of our own repositories — use it for "what does <repo> do / how is it built / which repo handles X". Keep `limit` at 5 or less.
2. **Drill down only if deciding.** Choosing between two approaches → `mcp__mx__rag_compare_approaches`. Need code shape for a chosen pattern in a language → `mcp__mx__rag_find_implementation`. Expanding around a chosen doc → `mcp__mx__rag_find_related`.
3. **Apply as advisory, not gospel.** Returned techniques are patterns, not requirements. Adopt what fits the codebase's existing conventions; skip what doesn't. When a technique shapes a plan or PR, cite its source doc path (e.g. `docs/development/appendix-rust-concurrency.md`).
4. **Never block on the corpus.** If tools return the offline message or nothing relevant: note it in one line and proceed with your own judgment. `mcp__mx__rag_health` diagnoses; `mx rag ingest` re-ingests; do NOT stop work to repair the corpus unless the user asks.
5. **Feed the backlog when you find a gap.** If `rag_context` returned nothing relevant (or only weak, off-topic chunks) for a topic worth knowing, append one line to the mech-crate research backlog so research mode picks it up later: `- [ ] <topic> — <one-line why> (added YYYY-MM-DD by techniques-skill)` in `docs/development/RESEARCH_BACKLOG.md` (repo located via $MECH_CRATE_ROOT, then ~/.mech-crate/config/source-root, then ~/dev/dev916/mech-crate). If the repo isn't reachable, skip silently. Then continue your task — never block on this.

## When to consult

- Writing or reviewing an implementation plan (writing-devloop-plans does this automatically)
- Starting a task in a covered domain (concurrency, DB schema, API design, Docker builds, FSM/FRP state, functional patterns)
- Torn between two designs — get the corpus's tradeoff framing before deciding
- The user explicitly asks for "the right way" / "best practice" / "what pattern"

## When NOT to consult

- Trivial mechanical edits (rename, typo, version bump)
- Domains the corpus doesn't cover — check `rag_health` by_category if unsure
- Re-querying the same question in the same session — reuse what you got
