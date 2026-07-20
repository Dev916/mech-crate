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
