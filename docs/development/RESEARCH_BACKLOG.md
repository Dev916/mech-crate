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

- [x] Rust async cancellation and graceful shutdown patterns — corpus covers concurrency primitives but not structured cancellation (added 2026-07-18 by design)
- [ ] Supply-chain security for dependency ecosystems (cargo/npm) — security category is thin on tooling practice (added 2026-07-18 by design)
- [ ] Local-first sync engines (CRDTs in practice) — emerging pattern, no coverage (added 2026-07-18 by design)
- [ ] mx infra: cloudflare provisioning, wrangler containers, terraform flow front to back — completes the deploy half of the inventory (added 2026-07-19 by technique-research)
- [ ] mx docs compilation pipeline (mx_docs_compile, md2pdf) and cc-plugin (install/session/stop audit) — remaining CLI surface for the full functional inventory (added 2026-07-19 by technique-research)
- [ ] mx router production HTTPS: letsencrypt/ACME + mkcert local certs operational guide (added 2026-07-19 by technique-research)
- [ ] unyform integration beyond recipes: auth model, blueprint generation from connected repos, org management (added 2026-07-19 by technique-research)
- [ ] mx testing & CI strategy: wiring tests/testbed scaffold smoke tests into CI, recipe.json validation (added 2026-07-19 by technique-research)
- [ ] Unix-socket IPC + singleton daemon patterns (flock lifetime locks, stale-socket handling, peer creds) — corpus silent; needed for local broker daemons (added 2026-08-14 by techniques-skill)
- [ ] POSIX child-process supervision in Rust (process groups, PID-reuse witnesses, zombie reaping, detachment) — corpus silent; recurring need for CLI-spawned daemons (added 2026-08-14 by techniques-skill)
- [x] LLM multi-agent orchestration patterns (brief design, blocked/escalation protocols, trust-but-verify, fleet observability) — corpus has no agentic-systems docs; a2a project built on general distributed-systems analogies (added 2026-08-14 by techniques-skill; covered by `multi-agent-systems-in-practice.md` 2026-08-14)
- [ ] Secrets handling for headless AI agents (ingress/egress redaction, env allowlists, sandbox+network exfiltration surfaces) — security category only covers SEC compliance (added 2026-08-14 by techniques-skill)
- [ ] OTel GenAI semantic conventions stability watch — every `gen_ai.*` element still "Development", no cost attributes; re-check for a 1.0 before building telemetry against them (added 2026-08-14 by technique-research)
- [ ] PostgreSQL 19 GA (Sept 2026): verify pgvector compatibility before any Neon/local PG major upgrade (added 2026-08-14 by technique-research)
- [ ] Local-inference hardware re-check: rumored high-memory M5 Ultra Mac Studio (~Oct 2026) vs the current 96GB ceiling; revisit local-vs-API math only if it ships (added 2026-08-14 by technique-research)
- [ ] MCP Tasks V2 wire protocol — V1 was too involved for client adoption; re-check when the redesign lands in released SDKs (added 2026-08-14 by technique-research)
