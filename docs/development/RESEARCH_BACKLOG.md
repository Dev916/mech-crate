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
- [x] Supply-chain security for dependency ecosystems (cargo/npm) — security category is thin on tooling practice (added 2026-07-18 by design; covered by `supply-chain-security-cargo-npm.md` 2026-09-25)
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
- [ ] real-time streaming voice-agent pipelines (STT/LLM/TTS staging, turn-taking, barge-in) — no corpus coverage; needed for meeting-agent work (added 2026-08-26 by techniques-skill)
- [ ] feature flags for progressive rollout of write paths, per-rule kill switches, dark launch — corpus has no coverage; needed for Django task-engine rollout (added 2026-09-18 by techniques-skill)
- [ ] lookup tables vs enums vs config modules for select-list vocabularies (operator editability, referential integrity, reporting stability, A/B variants) — corpus has no coverage (added 2026-09-18 by techniques-skill)
- [ ] work-queue / task-engine domain modeling: lifecycle statechart, outcome vocabularies, rule-generated idempotent records, ranking-as-data — corpus has only generic DB/statechart material (added 2026-09-18 by techniques-skill)
- [ ] Declarative, reproducible OS fleets with NixOS (DAWO-style workplace images): when to adopt over Ansible or containers, module and flake layering, fleet update and rollback — HN 915 pts / 532 c, 2026-09-25, https://news.ycombinator.com/item?id=49841563; why: the Dutch Ministry of the Interior is building a government workplace on NixOS for reproducibility and inspectability, and the corpus has no NixOS coverage (added 2026-09-25 by technique-research)
- [ ] Rootless sandboxes for coding agents and third-party programs (user, mount, network, IPC and cgroup namespaces, optional gVisor, Landlock/seccomp): which boundary stops which failure — HN 189 pts / 63 c, 2026-09-22, https://news.ycombinator.com/item?id=49801329; why: Drop isolates agent runs without root or a container image, the corpus mentions gVisor only inside the Python doc and has no namespace-level guidance (added 2026-09-25 by technique-research)
- [ ] Dead-man's-switch heartbeat monitoring for scheduled jobs (expected interval plus grace window, ping-on-success, missed-run alerting) for cron and launchd agents — HN 81 pts / 35 c, 2026-09-19, https://news.ycombinator.com/item?id=49765354; why: our own weekly research LaunchAgent has no missed-run alerting and the corpus is silent on heartbeat monitors (added 2026-09-25 by technique-research)
