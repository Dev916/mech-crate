# Repo Profiles in the Techniques Corpus — Design

**Date:** 2026-09-01 · **Status:** proposed (epic created in beads, see §10) · **Owner:** Mike

## 1. Goal

Teach the techniques corpus — mech-crate `docs/development`, ingested by `mx rag ingest` into pgvector on Neon and queried by every agent in every project through `mcp__mx__rag_context` — everything about **our own repositories**: what each one does and is capable of, how it was built, in what language and stack, how to build/run/test it, and how it relates to the others. After this lands an agent can ask "what does hq do", "how do I run pongballz", or "which repo handles meeting capture" and get a sourced answer instead of grepping the filesystem.

Scope: **36 repositories → 37 profile docs** (gnn gets a monorepo overview plus a Nexus City app profile) **+ 1 cross-repo map**, all under `docs/development/repos/`.

**Non-goals.** Publishing profiles on mechcrate.dev (they stay internal by default — §3 D3). Modifying any target repository. Re-documenting mx techniques the corpus already holds (the mech-crate profile links to them). Building a new retrieval mode — profiles must retrieve through the existing tools.

## 2. Context: how the corpus actually works (verified from source)

| Fact | Where | Consequence |
|---|---|---|
| `mx rag ingest` walks `docs/development` **recursively** for `*.md`, skipping only `INDEX.md` | `crates/mx-lib/src/corpus/ingest.rs` `scan_dir` (walkdir) | a `repos/` subdirectory is ingested; stored `path` becomes `repos/<slug>.md` |
| Frontmatter struct is `#[serde(default)]`, free-form `category`, unknown keys ignored; malformed YAML → warning + heuristics | `corpus/frontmatter.rs` | we can add `repo:`, `local_path:`, `status:`, `publish:` keys safely; `mx rag ingest --dry-run` must stay at 0 warnings |
| Chunker splits on `##`, packs paragraphs to 1,200 chars, prefixes every chunk with `Doc Title > Heading` | `corpus/chunk.rs` | a fixed `##` section set makes every chunk self-identifying ("hq: … (Repo Profile) > Capabilities") |
| Store keeps `category` per chunk and filters `c.category = $n`; the MCP tools only *describe* categories in prose, they do not validate them | `corpus/store.rs`, `crates/mx-mcp-server/src/tools/mod.rs:923,961,971` | a new `repos` category needs prose updates only |
| The mechcrate.dev loader reads **only files directly in** `docs/development` (`readdirSync` + `isFile`) and respects `publish: false` | `site/apps/site/src/loaders/lib/sources.ts`, `lib/pipeline.ts` | a subdirectory is outside the publish scope — we make that an explicit contract (§3 D3) |
| Corpus today: 66 docs, 2,148 chunks, 14 categories; `mx rag status` healthy on Neon | `mx rag status` 2026-09-01 | ~37 profiles ≈ +900 chunks — retrieval pollution must be measured (§8) |
| The MCP server currently searches **lexical-only** ("no embedding key configured") because `.mcp.json` passes no key and `config.rs:87` only reads env / `rag.toml` | `mcp__mx__rag_search` output; `corpus/config.rs` | framework task F4 fixes this before the first wave gate |
| hq already holds a project registry (14 projects × 7 clients) and `hq_projects` exposes it | `mcp__hq__hq_projects` | the ownership axis for the cross-repo map |
| The corpus knows nothing about hq, a2a, meetnotes, cupcake, … today; it knows mx well (5 docs) and forst via `tries-and-radix-dispatch.md` | `rag_search` probes | verdict NEW for 35 repos; mech-crate is an umbrella/IMPROVE |

## 3. Decisions

**D1 — Location.** `docs/development/repos/<slug>.md`. One file per repo; slug = repo name lower-cased with dots → hyphens (`unyform-ai`, `nyvorin-com`, `pricelove-co`), `.codex` → `codex-config`, gnn's flagship app → `gnn-nexus-city`. The map is `repos/repo-map.md`.

**D2 — Category.** New `category: repos`. `rag_search_category` with `repos` works immediately (string filter); the tool descriptions and the techniques skill list it after F2. Complexity is always `intermediate` (profiles are reference, not theory).

**D3 — Not published by default.** Two independent guards: the site loader's flat scan (made an explicit, tested contract in F2 and named in the site spec's "Never published" list) **and** `publish: false` in every profile. Private repos' internals must never reach mechcrate.dev by accident; public repos can be flipped later, deliberately.

**D4 — Frontmatter.** Standard keys the ingester reads (`title`, `category`, `languages`, `complexity`, `use_cases`, `summary`) plus provenance keys the site/humans read (`provenance: researched`, `researched`, `sources`, `publish`) plus repo keys nobody parses yet but the refresh task will (`repo`, `local_path`, `status`, `visibility`, `owner`, `hq_project`). Title format is fixed so every chunk names its repo: `"<name>: <one-line what it is> (Repo Profile)"`.

**D5 — Fixed section set.** The eleven `##` headings in §4, in order, always present (write "none found" rather than dropping one). `###` sub-headings are free. Rationale: the chunker's `##` split + heading prefix means consistent headings give consistent retrieval ("… > Capabilities" answers "what can X do", "… > How It Was Built" answers "how do I build X").

**D6 — Evidence rules (inherited from technique-research).** Every capability claim traces to a repo-relative path. README claims are verified in code or marked "README claims, not verified". The author's own inferences live only under `### Synthesis (inferred)`. Configuration is documented by **name and purpose only — never values**; a secret-pattern grep gates every profile. Unimplemented/aspirational features are labelled as such.

**D7 — Sizing.** 120–500 lines (unyform.ai may reach 600). Monorepos get one overview profile; a component gets its own profile only when the roster says so (today: gnn → gnn-nexus-city).

**D8 — mech-crate is one profile with `mx`.** The corpus already holds `mx-app-playbook`, `MX_RUST_CLI_AND_MCP_SERVER`, `mx-recipes-and-build`, `mx-cloudflare-deploy`, `mx-mcp~usage`. The profile is an umbrella — identity, full capability inventory (CLI, MCP tools, `mx-ingest`, router, recipes, site, skills, research automation) and pointers — not a sixth copy.

**D9 — Read-only research.** Profile authors never modify the target repo, never install dependencies in it, never run its build unless it is a documented no-side-effect command. Four repos are not checked out; F3 clones them first.

**D10 — Shared files are off-limits to profile tasks.** `INDEX.md`, `RESEARCH_LOG.md`, `RESEARCH_BACKLOG.md` are touched only by the wave gates and the wrap-up task, so parallel workers cannot conflict — each profile branch changes exactly one file.

## 4. The profile template

```markdown
---
title: "hq: Local-First Project Command Center (Repo Profile)"
category: repos
languages: [rust, typescript]
complexity: intermediate
use_cases:
  - "understanding what hq does and where its code lives"
  - "finding hq's CLI / MCP surface before extending it"
  - "answering 'which repo handles clients, meetings, todos, agent jobs'"
  - "resuming work on hq in a fresh session"
summary: "One paragraph: what it is, what it can do, the stack, and its status — written so a retrieval hit on the summary alone answers the question."
provenance: researched
researched: 2026-09-01
publish: false
repo: https://github.com/Dev916/hq
local_path: ~/dev/hq
status: active            # active | maintained | dormant | archived | template
visibility: private
owner: PriceLove LLC (Dev916)   # or Personal (nyvorin) / Unyform
hq_project: hq            # hq registry slug when one exists, else omit
sources:
  - README.md (hq repo)
  - crates/hq-core/src/lib.rs (hq repo)
---

# hq

> Elevator pitch, one paragraph: what it is, who/what uses it, the problem it solves. This preamble becomes the first chunk.

## Identity
| Field | Value |
|---|---|
| Repository | `Dev916/hq` (private) — default branch `main` |
| Local path | `~/dev/hq` |
| Owner / org | PriceLove LLC (Dev916) · hq project `hq` |
| Status | active — last commit 2026-08-26, 294 commits, profiled at `abc1234` |
| Languages (by file count) | Rust 105 · Markdown 56 · YAML 41 · TypeScript 17 |
| Build system | Cargo workspace (7 crates), npm (frontend), Tauri, Makefile |
| Runtime deps | Valkey, macOS |
| License | … |
| CI / release | … |

## What It Does
Problem → solution → who uses it (humans, agents, other repos). What "done" looks like for a user.

## Capabilities
Verified inventory, grouped by surface. Every bullet ends with a path.
### CLI
- `hq agenda` — … (`crates/hq-cli/src/commands/agenda.rs`)
### MCP tools
- `hq_projects` — … (`crates/hq-server/src/mcp/tools.rs`)
### HTTP API / UI / Skills / Libraries / Background jobs
…
### Not (yet) implemented
Anything the README/specs promise that the code does not do.

## Architecture
Stack with versions; component map (crates/packages/apps); data flow (short ASCII diagram welcome); storage (databases, files, formats); external integrations; process/concurrency model; security model (auth, where secrets live — names only).

## Repository Layout
```
crates/hq-core/     domain model …
crates/hq-server/   axum server + MCP …
```
Entry points listed explicitly.

## How It Was Built
Toolchain and versions; build/run/test/lint commands as they really are; dev loop (make targets, mx usage, router URL rule if applicable); CI/CD and release/deploy path; configuration and environment variable names with purpose (never values); provenance — design specs (`docs/superpowers/specs/…`), beads usage, agent-built history.

## Relationships
Depends on (our repos) · used by · shares code or patterns with · supersedes / superseded by · canonical-copy notes. Link sibling profiles by path (`docs/development/repos/meetnotes.md`).

## Notable Techniques
Patterns worth knowing or extracting; link existing corpus docs where they exist; list candidates for RESEARCH_BACKLOG.md (do not append there — the wrap-up does).

## State, Gaps and Drift
Maturity; README-vs-code drift; TODO/FIXME density; open beads count (`bd stats` in the repo); dead code; risks.
### Synthesis (inferred)
Your own conclusions, clearly separated.

## Quick Reference
| Task | Command / path |
|---|---|
| Build | `cargo build --release` |
| Run | … |
| Tests | … |
| URL | `http://hq.localhost` (via mx router) |

## Sources
Repo-relative paths read for this profile (feeds the `sources:` list).
```

## 5. Author checklist (acceptance for every profile task)

- [ ] File at `docs/development/repos/<slug>.md`; branch `corpus/repo-<slug>`; `git diff --stat` touches only that file.
- [ ] Frontmatter has every key in §4; `title` ends with `(Repo Profile)`; `category: repos`; `publish: false`.
- [ ] All eleven `##` sections present in order; Identity records HEAD short sha + date.
- [ ] Every capability bullet cites a repo-relative path; README claims verified or labelled; inferences only under `### Synthesis (inferred)`.
- [ ] No secret values. `grep -nE 'postgres://[^l]|sk-[A-Za-z0-9]{8,}|AKIA[0-9A-Z]{12,}|[Bb]earer [A-Za-z0-9._-]{16,}' <file>` is empty.
- [ ] 120–500 lines (600 for unyform-ai).
- [ ] `mx rag ingest --dry-run` → 0 warnings.
- [ ] Target repo untouched: `git -C <repo> status --porcelain` identical before and after.
- [ ] Did not edit INDEX.md / RESEARCH_LOG.md / RESEARCH_BACKLOG.md.

## 6. Execution model

**Waves.** Repos are grouped so a worker holds related context and cross-links land together:

| Wave | Cluster | Repos | Priority |
|---|---|---|---|
| 1 | Agentic core | mech-crate (mx), a2a, hq, meetnotes, understudy, devloop, cupcake | P1 |
| 2 | Skills & personal automation | claude-skills, codex-skills, .codex, autologin, qr2fa, devtime, x-skill, docusign-skill, slack-skills, aicommit | P2 |
| 3 | Products & platforms | unyform.ai, gnn (+ gnn-nexus-city), nexus-client, nexus-tokyo, agentic-pricelove | P2 |
| 4 | Web, sites & Leptos | nyvorin.com, pricelove-website, pricelove.co, leptos-store, leptos-actix, wikimech, devmesh-traefik | P3 |
| 5 | Libraries, Solana & dormant | forst, badwords, compress-json-rs, Solara, pongballz, mr-robot | P3 |

**Workers.** Each profile is one task written for a memoryless worker. Two mechanisms fit; the brief is the same (`skills/repo-profile/SKILL.md`):
- *Claude subagents* (`superpowers:dispatching-parallel-agents`, 5–7 per wave) — recommended for wave 1: they can call `mcp__mx__rag_search` for dedup and `gh` for remote metadata, and they inherit no conversation state to leak.
- *a2a Codex workers* (impl lane, `files_in_scope: docs/development/repos/<slug>.md`, `verify: mx rag ingest --dry-run` + the secret grep) — a good fit for later waves; `a2a verify` gives the mechanical gate for free.

**Branches and PRs.** One worktree and branch `corpus/repo-<slug>` per profile (each changes one file → no conflicts). The wave gate merges the wave's branches into `corpus/repos-wave-<n>` and opens **one PR per wave**; a human merges (agents never merge their own work). After merge: `mx rag ingest`, then the retrieval smoke test, then RESEARCH_LOG rows.

**Retrieval smoke test (per profile, in the gate).** Three questions via `mcp__mx__rag_context`: "what does <name> do", "how do I build and run <name>", one capability phrase from its Capabilities section. PASS = profile in the top 3 for ≥ 2 of 3. Fix wording (summary, use_cases, headings), not the retriever. **Negative check:** five technique queries that work today ("rust async cancellation", "trie dispatch", "pgvector batch embedding", "docker compose recipe", "mx router URL") must still return their technique doc first.

**Framework tasks before wave 1:** F1 repo-profile skill + template (dry-run on devloop) · F2 `repos` category plumbing + site exclusion contract · F3 clone the four missing repos · F4 fix MCP lexical-only retrieval.

**After wave 5:** `repo-map.md` (clusters, relationship graph, ownership axis from GitHub org × hq project, "which repo for X" table, status board, canonical-copy decisions) → wrap-up (INDEX section, techniques-skill guidance, `bd remember`, memory file) → refresh policy (staleness = recorded HEAD sha drift > 25 commits or `researched:` > 90 days for active repos; a "stalest repo profile" rung in technique-research's autonomous ladder so the existing weekly cron refreshes them).

## 7. Roster

Resolved on 2026-09-01 from local checkouts (`git remote`, `git log`, file counts) and `gh repo list Dev916 / nyvorin`. Remotes written `web-mech/*` redirect to `nyvorin/*`. Local directory names that differ from the repo name are called out.

| # | Repo | Remote | Local path | Stack | Status · last commit | Wave |
|---|---|---|---|---|---|---|
| 1 | `mech-crate` — mech-crate (mx) | `Dev916/mech-crate` (public) | `~/dev/dev916/mech-crate` | Rust, Shell, TypeScript | active · 2026-09-01 | 1 |
| 2 | `a2a` — a2a | `Dev916/a2a` (private) | `~/dev/a2a` | Rust | active · 2026-08-31 | 1 |
| 3 | `hq` — hq | `Dev916/hq` (private) | `~/dev/hq` | Rust | active · 2026-08-26 | 1 |
| 4 | `meetnotes` — meetnotes | `Dev916/meetnotes` (private) | `~/dev/meetnotes` | Python | active · 2026-08-27 | 1 |
| 5 | `understudy` — understudy | `Dev916/understudy` (private) | `~/dev/understudy` | Python, Swift | active · 2026-08-31 | 1 |
| 6 | `devloop` — devloop | `nyvorin/devloop` (public) | `~/dev/devloop` | Markdown | active · 2026-04-22 | 1 |
| 7 | `cupcake` — cupcake | `nyvorin/cupcake` (private) | `~/dev/dev916/cupcake` | Python | active · 2026-09-01 | 1 |
| 8 | `claude-skills` — claude-skills (~/.claude/skills) | `nyvorin/claude-skills` (private) | `~/.claude/skills` | Markdown, Python | active · 2026-08-30 | 2 |
| 9 | `codex-skills` — codex-skills (~/.codex/skills) | `nyvorin/codex-skills` (private) | `~/.codex/skills` | Markdown, JavaScript | active · 2026-08-20 | 2 |
| 10 | `codex-config` — .codex (Codex CLI home) | `nyvorin/.codex` (private) | `~/.codex` | Markdown, YAML, Python | active · 2026-08-20 | 2 |
| 11 | `autologin` — autologin | `Dev916/autologin` (private) | `~/dev/autologin` | Python | active · 2026-07-19 | 2 |
| 12 | `qr2fa` — qr2fa | `Dev916/qr2fa` (private) | `~/dev/qr2fa` | Python | active · 2026-07-18 | 2 |
| 13 | `devtime` — devtime | `Dev916/devtime` (public) | `~/dev/devtime` | Python, Markdown | active · 2026-07-02 | 2 |
| 14 | `x-skill` — x-skill | `Dev916/x-skill` (private) | `~/dev/x-skill` | Python, Markdown | active · 2026-07-06 | 2 |
| 15 | `docusign-skill` — docusign-skill | `Dev916/docusign-skill` (private) | `~/dev/docusign-skill` | Python, Markdown | active · 2026-07-04 | 2 |
| 16 | `slack-skills` — slack-skills | `Dev916/slack-skills` (public) | `~/dev/slack-skills` | Python, Markdown | active · 2026-07-01 | 2 |
| 17 | `aicommit` — aicommit | `nyvorin/aicommit` (public) | `~/dev/dev916/aicommit` | Python | dormant · 2025-09-11 | 2 |
| 18 | `unyform-ai` — unyform.ai | `Dev916/unyform.ai` (private) | `~/dev/dev916/unyform.ai` | Rust | active · 2026-08-25 | 3 |
| 19 | `gnn` — gnn (GhostNN monorepo) | `Dev916/gnn` (private) | `~/dev/dev916/gnn` | TypeScript, Rust, Python | active · 2026-08-13 | 3 |
| 20 | `gnn-nexus-city` — Nexus City (gnn/apps/nexus-city) | `Dev916/gnn` (private) | `~/dev/dev916/gnn/apps/nexus-city` | TypeScript | active · 2026-08-13 | 3 |
| 21 | `nexus-client` — nexus-client | `Dev916/nexus-client` (public) | `(not checked out — clone to ~/dev/dev916/nexus-client; embedded copy at gnn/apps/nexus-city/nexus-client)` | Rust | active · 2026-07-18 | 3 |
| 22 | `nexus-tokyo` — nexus-tokyo (local dir gnn2) | `nyvorin/nexus-tokyo` (private) | `~/dev/dev916/gnn2` | TypeScript/JavaScript | active · 2026-06-25 | 3 |
| 23 | `agentic-pricelove` — agentic-pricelove | `Dev916/agentic-pricelove` (private) | `~/dev/dev916/agentic-pricelove` | Markdown, Shell | dormant · 2026-06-13 | 3 |
| 24 | `nyvorin-com` — nyvorin.com | `nyvorin/nyvorin.com` (private) | `~/dev/dev916/nyvorin.com` | Rust | active · 2026-08-27 | 4 |
| 25 | `pricelove-website` — pricelove-website | `Dev916/pricelove-website` (private) | `(not checked out — clone to ~/dev/dev916/pricelove-website)` | TypeScript | active · 2026-08-27 | 4 |
| 26 | `pricelove-co` — pricelove.co | `Dev916/pricelove.co` (private) | `~/dev/dev916/pricelove-2/pricelove/pricelove.co` | Rust | dormant · 2025-10-21 | 4 |
| 27 | `leptos-store` — leptos-store | `nyvorin/leptos-store` (public) | `~/dev/dev916/leptos-store` | Rust | maintained · 2026-03-18 | 4 |
| 28 | `leptos-actix` — leptos-actix | `Dev916/leptos-actix` (public) | `(not checked out — clone to ~/dev/dev916/leptos-actix)` | Rust | template · 2025-12-23 | 4 |
| 29 | `wikimech` — wikimech | `Dev916/wikimech` (private) | `(not checked out — clone to ~/dev/dev916/wikimech)` | Shell, Markdown | dormant · 2026-01-12 | 4 |
| 30 | `devmesh-traefik` — devmesh-traefik (local dir stack) | `nyvorin/devmesh-traefik` (public) | `~/dev/dev916/stack` | Shell, YAML | dormant · 2025-11-14 | 4 |
| 31 | `forst` — forst | `nyvorin/forst` (public) | `~/dev/dev916/forst` | JavaScript | dormant · 2025-11-07 | 5 |
| 32 | `badwords` — badwords (npm bad-words) | `nyvorin/badwords` (public) | `~/dev/dev916/badwords` | TypeScript, JavaScript | maintained · 2026-07-19 | 5 |
| 33 | `compress-json-rs` — compress-json-rs | `nyvorin/compress-json-rs` (public) | `~/dev/dev916/compress-json-rs` | Rust | dormant · 2026-01-01 | 5 |
| 34 | `solara` — Solara | `Dev916/Solara` (private) | `~/dev/dev916/Solara` | TypeScript/JavaScript, Rust | dormant · 2025-04-27 | 5 |
| 35 | `pongballz` — pongballz | `nyvorin/pongballz` (private) | `~/dev/dev916/pongballz` | JavaScript, Rust | maintained · 2026-07-19 | 5 |
| 36 | `mr-robot` — mr-robot (trade-bot-v2) | `Dev916/mr-robot` (private) | `~/dev/dev916/mr-robot` | TypeScript | dormant · 2025-09-25 | 5 |

Interpretation notes: **mech-crate and `mx`** are one repository, one profile (the profile covers the research subsystem — RESEARCH_BACKLOG/LOG, technique-research, weekly cron — explicitly). **claude-skills** is `~/.claude/skills` (nyvorin/claude-skills), not revenium/test-bench/claude-skills (a client repo that moved into the revenium plugin). **.codex** is `~/.codex` (nyvorin/.codex) and **codex-skills** is the repo nested at `~/.codex/skills`. **nexus-tokyo** lives locally as `~/dev/dev916/gnn2`; **devmesh-traefik** as `~/dev/dev916/stack`. **nexus-client** is public and also embedded in gnn.

## 8. Risks and mitigations

| Risk | Mitigation |
|---|---|
| ~900 repo chunks displace technique retrieval | wave-gate negative checks; `category` filter in `rag_context`; if measured, a follow-up ranks `repos` below techniques unless asked |
| A profile leaks a credential | names-only rule, secret grep in every acceptance, `publish: false` + loader scope keep it off the web even if it slips |
| Profiles rot | Identity records HEAD sha; refresh task adds staleness detection to the weekly run |
| Workers "helpfully" fix the target repo | read-only rule; acceptance checks the target repo's `git status` is unchanged; a2a's scope snapshot for Codex workers |
| Parallel branches collide | one file per branch; shared files reserved for gates/wrap-up |
| MCP retrieval is lexical-only today | F4 before any gate |

## 9. Open decisions (defaults chosen; override any)

1. **Publish public repos' profiles on mechcrate.dev?** Default **no** for all 37; revisit per repo after wave 5.
2. **One PR per wave vs per profile?** Default **per wave** (5–10 one-file diffs per review). Per profile is a one-line change to the gate task.
3. **Claude subagents vs a2a Codex workers?** Default **Claude subagents for wave 1**, reassess after.
4. **Where to clone the 4 missing repos?** Default `~/dev/dev916/<name>` (F3).
5. **Is "research time" a separate repository?** Read as the research subsystem inside mech-crate. If it is its own repo, add a roster row.

## 10. Tracking

Beads epic in mech-crate (`bd list --parent <epic-id>`): 1 epic · 4 framework tasks (F1–F4) · 37 profile tasks (labels `profile`, `wave-1…5`) · 5 wave gates (`gate`) · `repo-map.md` · wrap-up · refresh policy = 50 issues. Dependencies: every profile ← F1 (and ← F3 for the four clones); every gate ← its wave's profiles + F2 + F4; repo-map ← all gates; wrap-up ← repo-map; refresh ← wrap-up. Epic id: `mech-crate-965` (children are hierarchical: `mech-crate-965.1`–`.4` framework, `.5`–`.40` profiles, `.41`–`.45` gates, `.46` repo-map, `.47` wrap-up, `.48` refresh).
