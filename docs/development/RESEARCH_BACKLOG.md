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
