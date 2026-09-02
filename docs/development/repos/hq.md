---
title: "hq: local-first project command center — clients, projects, meetings, triage, work and corpus behind one CLI/REST/MCP surface (Repo Profile)"
category: repos
languages: [rust, typescript, vue, astro, sql, markdown]
complexity: intermediate
use_cases:
  - "understanding what hq does and where its code lives"
  - "finding hq's CLI, MCP and REST surface before extending it"
  - "answering 'which repo owns the client/project registry the other repos key off'"
  - "resuming work on hq in a fresh session"
summary: "hq is a private Rust workspace (seven crates, ~66k lines) that turns one laptop into a project command center: a `registry.toml` describes clients, projects and their syncable sources; hq-server on 127.0.0.1:7717 mirrors that registry into Postgres, runs a Valkey-backed job queue, and exposes the same store over three identical faces — the `hq` CLI, a REST API, and a 16-tool MCP server (`hq_projects`, `hq_agenda`, `hq_items`, `hq_inbox`, `hq_work`, `hq_infra`, `hq_jobs`, `hq_corpus_search`, `hq_push_item`, `hq_push_note`, `hq_sync`, `hq_run_agent`, `hq_write_work`, `hq_schedule_meeting`, `hq_triage`, `hq_reply`). Six sync sources (calendar, gmail, slack, tracker, git, corpus) run through one `Channel` trait engine with a persisted per-binding state machine, backoff/quarantine, a one-writer lease, and a transactional write outbox that makes double-sends impossible; every outbound send or tracker write is human-approved by default and there is deliberately no MCP approve tool. An Astro+Vue dashboard on :7718 and a Tauri v2 tray shell wrap it. Its 14 projects across 7 clients (Blackmast / Fenzi / Nexian / Personal / PriceLove LLC / Revenium / Unyform) are the ownership axis the other repos are mapped against, and its corpus subsystem is the direct ancestor of mech-crate's techniques corpus. Active: 310 commits, last 2026-09-02."
provenance: researched
researched: 2026-09-02
publish: false
repo: https://github.com/Dev916/hq
local_path: ~/dev/hq
status: active
visibility: private
owner: PriceLove LLC (Dev916)
hq_project: hq
sources:
  - README.md, CLAUDE.md, AGENTS.md, Cargo.toml (target repo)
  - registry.example.toml, config.example.toml, .env.example, .gitignore (target repo)
  - docs/superpowers/specs/2026-07-12-hq-command-center-design.md (target repo)
  - docs/adr/0001-channel-trait.md (target repo)
  - skills/hq/SKILL.md, skills/hq-onboard/SKILL.md, skills/hq-dev/references/architecture.md (target repo)
  - crates/hq-cli/src/{main.rs,up.rs} (target repo)
  - crates/hq-server/src/{main.rs,api.rs,background.rs,launcher.rs} (target repo)
  - crates/hq-server/src/mcp/{tools_read.rs,tools_write.rs,tools_triage.rs} (target repo)
  - crates/hq-jobs/src/handlers.rs, crates/hq-channels/src/port.rs (target repo)
  - crates/hq-core/src/{config.rs,store/mirror.rs,store/corpus.rs} (target repo)
  - crates/hq-corpus/src/ingest/chunk.rs (target repo)
  - migrations/ (target repo, 15 files)
  - frontend/{astro.config.mjs,package.json,src/pages/api/sync.ts,src/lib/capabilities.ts} (target repo)
  - tauri/README.md, tauri/src-tauri/Cargo.toml (target repo)
  - scripts/dev-check.sh, .beads/issues.jsonl (target repo)
  - crates/mx-lib/src/corpus/{mod.rs,chunk.rs,store.rs} (mech-crate)
  - skills/technique-research/references/source-providers.md (mech-crate)
---

# hq

> hq is the **command center for a consulting practice run by one person and a
> fleet of agents**. A gitignored `registry.toml` declares the world — clients,
> their projects, and each project's syncable sources (calendar, mailbox, Slack
> workspace, issue tracker, git repos) — and `hq-server` mirrors that world into
> Postgres, pulls each source on a schedule through a pluggable channel engine,
> and serves the result over three interchangeable faces: the `hq` CLI, a REST
> API, and an MCP server. Agents read the same store a human reads. Everything
> that touches the outside world is a **job** with a row, an event, and (for
> anything that sends) a human approval gate that no MCP tool can bypass. The
> dashboard is Astro+Vue on :7718; a Tauri tray shell wraps it. hq's 14
> projects across 7 clients are the slugs the rest of this corpus uses as its
> ownership axis, and hq's corpus subsystem is where mech-crate's techniques
> corpus was ported from.

## Identity
| Field | Value |
|---|---|
| Repository | `Dev916/hq` (private) — default branch `main` |
| Local path | `~/dev/hq` (directory name matches the repo name) |
| Owner / org | PriceLove LLC (Dev916) · hq project `hq` ("hq Command Center") |
| Status | active — last commit 2026-09-02, 310 commits, profiled at `346cb2a` (local `main` level with `origin/main`) |
| Languages (by file count) | Rust 108 (105 in `crates/`, 3 in `tauri/`) · Markdown 57 · PNG 53 · Vue 17 · TypeScript 15 · SQL 15 · TOML 12 · JSON 7 · Astro 6 · shell 3 — 315 tracked files; ~66.6k lines of Rust, ~12.7k lines under `frontend/src` |
| Build system | Cargo workspace (7 members) + a workspace-**excluded** Tauri crate + npm/Astro for the dashboard |
| Runtime deps | Postgres with `vector` + `pg_trgm` extensions (Neon), Valkey (Homebrew), Node (dashboard), macOS launchd, Google Chrome (profile-aware launcher); shelled-out binaries `bd`, `git`, `gh`, `slackcli` |
| License | none declared at the repo root; the standalone `hq-shell` Tauri crate declares MIT in its own `Cargo.toml` |
| CI / release | none — no `.github/` directory, no tags, no releases. The gate is `scripts/dev-check.sh` run locally |

## What It Does

The problem: a one-person consultancy with seven clients has its state scattered
across seven Slack workspaces, several Google accounts, three different issue
trackers, a dozen repos, an AWS account per engagement, and two years of
documents — and the agents doing the work can see none of it. Every "what is on
my plate" question means opening nine tabs.

hq's answer is a single local store with a single description of the world.
`registry.toml` names each client, each project under it, and each project's
channels; `mirror_registry` upserts that into `clients` / `projects` /
`accounts` on boot and on every registry save
(`crates/hq-core/src/store/mirror.rs`, `crates/hq-server/src/main.rs`). From
there, four background schedulers enqueue jobs that pull calendars, mailboxes,
Slack, trackers and git into the same tables, and a summarizer LLM turns unread
Slack threads and email into **action items** that land on the same project card
as meetings and todos (`crates/hq-server/src/background.rs`,
`crates/hq-channels/src/triage.rs`).

Its users are both the human and the agents. Reads never block on a remote API —
they are Postgres queries — so an agent asking `hq_agenda` gets an answer in
milliseconds. Writes are asymmetric on purpose: pushing an item or a note is
immediate, but anything that *leaves the machine* (a Slack reply, an email, a
client tracker write, a calendar booking) becomes a `pending_approval` job that
only a human can release. "Done" for a user looks like `hq status` green, the
dashboard showing every client's meetings/todos/attention items, and an approvals
queue they clear by hand (`skills/hq/SKILL.md`, `crates/hq-jobs/src/enqueue.rs`).

## Capabilities

### CLI (`hq`, a thin HTTP client onto :7717 — all in `crates/hq-cli/src/main.rs` unless noted)
- `hq up [--no-app] [--build-app]` / `hq down [--valkey]` — idempotent one-command stack: Valkey, both launchd agents, and the desktop app; resolves the repo from `HQ_ROOT` → its own symlink → `~/dev/hq` (`crates/hq-cli/src/up.rs`)
- `hq status` · `hq projects` · `hq items` · `hq agenda [--week]` · `hq join next` · `hq push item` · `hq jobs list|tail|approve`
- `hq sync all|calendar|retry <binding_key>`, with `--force` to bypass FSM gating for one pass
- `hq triage` · `hq inbox` · `hq reply <n|uuid>` — the attention loop; reply only ever drafts
- `hq work [--project]` · `hq mine` — tracker board plus correlated PR/branch/CI state
- `hq issue create|move|comment|edit` and `hq meet <project> "title" [--at] [--with]` — tracker and calendar write-back drafts
- `hq approvals` · `hq approve [<id>|--all [--type]]` · `hq reject <id>` — the only release paths, human-only
- `hq settings [set <dial> on|off]` — the three autopilot dials, with a y/N confirm on the risky direction
- `hq corpus search|status|ingest|import-blackmast` · `hq auth google|msgraph <account>`

### MCP tools (16, `POST /mcp`, stateless streamable HTTP, JSON-RPC 2.0, protocol `2025-06-18`)
- Read (8): `hq_projects`, `hq_agenda`, `hq_items`, `hq_inbox`, `hq_infra`, `hq_jobs`, `hq_work`, `hq_corpus_search` (`crates/hq-server/src/mcp/tools_read.rs`)
- Write/control (6): `hq_push_item`, `hq_push_note`, `hq_sync`, `hq_run_agent`, `hq_write_work`, `hq_schedule_meeting` (`crates/hq-server/src/mcp/tools_write.rs`)
- Triage (2): `hq_triage`, `hq_reply` (`crates/hq-server/src/mcp/tools_triage.rs`)
- **No `hq_approve` / `hq_reject` tool exists, and `scripts/dev-check.sh` asserts their absence** — approval is human-only by construction (`scripts/dev-check.sh`)
- Registered user-scope for Claude Code as `hq` → `http://127.0.0.1:7717/mcp` (confirmed live in `~/.claude.json`)

### HTTP API (actix-web, bound to `127.0.0.1` only, CORS allow-listing :7718 + `tauri://localhost`)
- 41 routes in one table: status/projects/agenda/items/inbox/triage/infra/work/notes/jobs/settings/corpus/config plus `/actions/open-url`, `/actions/open-meeting`, `/mcp` and the SSE stream `/api/events` (`crates/hq-server/src/api.rs`)
- `GET /api/channels` — the capability API: one row per discovered binding with `{binding_key, kind, provider, project_slug, state, fail_count, next_retry_at, capabilities, actions}`, where `capabilities` is what the channel *can* do and `actions` is what its current FSM state *allows* (`crates/hq-server/src/api.rs`)
- Config CRUD: `PUT`/`DELETE` on `/api/config/clients/{slug}` and `/api/config/projects/{slug}` rewrite `registry.toml` through `toml_edit`, re-mirror, and hot-swap the in-memory registry (`crates/hq-core/src/registry_edit.rs`, `crates/hq-server/src/api.rs`)

### Dashboard (Astro 5 SSR + Vue 3 islands, `127.0.0.1:7718`)
- Server-rendered shell plus 17 Vue islands — approvals queue, triage panel, work board, jobs ticker, config editor, infra panel, quick-add (`frontend/src/components/`)
- A **direct-to-queue** server route: `POST :7718/api/sync` writes the `jobs` row and `XADD`s to Valkey itself, bypassing hq-server, and hard-rejects `agent.run` with 400 (`frontend/src/pages/api/sync.ts`)
- Affordances read `/api/channels` and **fail open** — no capability rows means the button stays enabled, because the server enforces every gate anyway (`frontend/src/lib/capabilities.ts`)

### Desktop shell (Tauri v2, `hq-shell`)
- Wraps `http://127.0.0.1:7718` in a window, adds a menu-bar tray showing the next joinable meeting as `HH:MM title`, and fires a one-shot T-2min notification (`tauri/src-tauri/src/tray.rs`, `tauri/README.md`)
- GUI-free logic (`next_meeting`, `should_notify`, `tray_label`) is unit-tested in place; a test also asserts `tauri.conf.json`'s url, identifier and tray (`tauri/README.md`)

### Skills (shipped in-repo, not installed)
- `hq` — the agent-facing contract: tool arg shapes, push doors, approval policy, corpus gates, ops (`skills/hq/SKILL.md`, 332 lines)
- `hq-dev` — the contributor's map: crate boundaries, boot flow, job system, CRUD recipes, entity reference, devloop testing (`skills/hq-dev/SKILL.md` + 4 references)
- `hq-onboard` — LLM-assisted registry construction: discover accounts/infra from the machine read-only, confirm with the user, write and validate `registry.toml` (`skills/hq-onboard/SKILL.md`)

### Background jobs (14 handlers on a Valkey stream)
- Sync/ingest: `sync.all` (planner-driven fan-out), `sync.calendar`, `sync.tracker`, `sync.git`, `corpus.scan`, `registry.reload` (`crates/hq-jobs/src/handlers.rs`)
- Attention: `triage.slack`, `triage.email`, `triage.sweep` (`crates/hq-jobs/src/handlers.rs`)
- Write plane: `slack.send`, `email.send`, `work.write`, `calendar.create` — all through the outbox (`crates/hq-jobs/src/handlers.rs`)
- Agent: `agent.run` — spawns `[agent].cmd` (default `claude -p {prompt}`) in a project cwd, SIGKILLs the process group at `timeout_secs` (default 600), and files the last 4000 chars as an `items` row of kind `fact`, source `agent` (`crates/hq-jobs/src/handlers.rs`)

### Not (yet) implemented
- **The planner agent.** The spec designs a morning job that reads the whole agenda and writes `items` of `kind=plan, status=proposed` with per-line approve/dismiss; only the `agent.run` machinery exists, there is no `hq agent plan` verb and no plan-proposal UI (`docs/superpowers/specs/2026-07-12-hq-command-center-design.md`)
- **Harvest / time tracking.** `harvest_project_id` is carried in the registry and mirrored into `projects` (25 references) but no Harvest adapter exists (`crates/hq-core/src/store/mirror.rs`)
- **Keeper credential integration** — spec backlog item `ops-joc`, open as `hq-k3a`
- **`hq corpus search --category`** — documented in both `README.md` and `skills/hq/SKILL.md`; the store, the REST route and the MCP tool all support `category`, but the clap enum has no such flag (`crates/hq-cli/src/main.rs`)
- **Capability API on MCP** — `/api/channels` is deliberately REST/dashboard-only this phase; "the tool surface stays pinned" (`README.md`)
- **Per-card staleness badges** — spec'd for v1, not rendered (`hq-4cq`)

## Architecture

**Stack.** Rust stable (`rust-toolchain.toml` pins only `channel = "stable"`),
tokio + actix-web 4, sqlx 0.8 (Postgres, rustls), `pgvector` pinned to `=0.4.1`,
`redis` 0.27 for the Valkey stream, `toml` + `toml_edit` for the registry,
`reqwest` 0.12 for every provider, plus `pdf-extract` / `zip` / `quick-xml` /
`mime_guess` for corpus extraction and `wiremock` 0.6 for hermetic provider
tests (`Cargo.toml`). The dashboard is Astro 5 on the Node standalone adapter
with a Vue 3 integration; the desktop shell is Tauri 2.

**Component map (7 workspace crates + 1 excluded).**

| Crate | Owns | Never does |
|---|---|---|
| `hq-core` | `store/*` (one module per table), `config.rs`, `registry.rs`, `registry_edit.rs`, `secrets.rs` | remote I/O |
| `hq-adapters` | external I/O: google calendar/gmail, msgraph, slack, linear/jira/beads, git/gh, `token_source`, `timeouts` | store schema, HTTP serving |
| `hq-corpus` | ingest pipeline, extraction, chunking, embeddings, the LLM clients and the `mine` contract | the job queue |
| `hq-channels` | the `Channel` port, the pure planner, the engine (`run_binding`, `run_write`), six concrete channels, the shared triage pass | job semantics, HTTP serving |
| `hq-jobs` | `queue.rs`, `worker.rs`, `repo.rs`, `enqueue.rs`, `handlers.rs` | HTTP serving |
| `hq-server` | REST, MCP, SSE, four schedulers, registry watcher, `main.rs` | new store schema |
| `hq-cli` | the `hq` clap binary over HTTP | direct store access (except `up`/`down`/`auth`/`corpus import-blackmast`) |
| `hq-shell` (excluded) | Tauri window + tray + notification; own `Cargo.lock` so the Tauri tree never slows `cargo build` | anything the server owns |

**Data flow.**

```
registry.toml ──▶ mirror_registry ──▶ clients / projects / accounts
                      ▲                        │
                      │ registry watcher       │  Channel::discover  (PURE)
                      │ + registry.reload      ▼
   schedulers ──▶ hq_channels::plan ──▶ bindings ──▶ enqueue (coalesced)
                                                        │
                                jobs row + XADD hq:jobs │
                                                        ▼
                                      worker pool (default 4)
                                                        │
                              engine.run_binding: lease ─┼─ Channel::sync  ──▶ provider
                              engine.run_write:  outbox ─┘  Channel::write ──▶ provider
                                                        │
                            store updated ──▶ PUBLISH hq:events ──▶ SSE ──▶ dashboard
   read path:  CLI | REST | MCP ──▶ hq_core::store ──▶ Postgres   (no remote calls)
```

**Storage.** Postgres (Neon) holds 17 tables created by 15 checked-in
migrations: `clients`, `projects`, `accounts`, `infra`, `meetings`, `items`,
`notes`, `jobs`, `work_items`, `git_refs`, `sync_state`, `triage_state`,
`slack_users`, `settings`, `write_outbox`, `corpus_docs`, `corpus_chunks`
(`migrations/`). Two extensions are required: `vector` (1536-dim embeddings with
an HNSW cosine index) and `pg_trgm`. Valkey holds exactly two things: the
`hq:jobs` stream with a `workers` consumer group, and the `hq:events` pub/sub
channel the SSE bridge listens on. OAuth refresh tokens live as `0600` files
under `.secrets/` (gitignored), keyed by account.

**External integrations.** Google Calendar + Gmail, Microsoft Graph calendar,
Slack (via the `slackcli` binary), Linear, Jira, beads (`bd`), GitHub (`gh`) and
local `git`, plus an OpenAI-compatible LLM endpoint and an Anthropic Messages
client for summarization and embeddings.

**Process / concurrency model.** One `hq-server` process hosting everything: an
actix HTTP server, N job workers (`[work] worker_count`, default 4), four
scheduler loops (`sync_all_minutes` 0, `corpus_scan_minutes` 0, `triage_minutes`
10, `work_minutes` 15) and a registry file watcher. Concurrency safety is
per-binding, not global: `SyncStarted` is an atomic lease acquisition
(`state='syncing', lease_until = now()+deadline` where the lease is free), a held
lease makes a second attempt busy-skip and record nothing, and a boot sweep flips
orphaned `syncing` rows back to failed. No long transactions anywhere — the
design note calls this "Neon-friendly, crash-self-healing"
(`crates/hq-core/src/store/sync_fsm.rs`, `README.md`).

**Security model.** Loopback-bound (`bind(("127.0.0.1", port))`), CORS allow-list
of exactly three origins, no authentication because there is no remote surface.
The governing rule from the spec is **"the panel stores identifiers, never
credentials"**: AWS profile names, account ids, login URLs, Terraform workspace
names and SSH host aliases are stored; keys, passwords and session tokens are
not, and stay in `~/.ssh`, `~/.aws` and Keychain referenced by name. Credential
*reuse* is the second rule — existing local stores (the workspace-mcp Google
credential directory, the `atlassian` MCP env block, the slackcli token store)
are read read-only and never written, with hq-native OAuth under `.secrets/` as
the fallback only. Config names, no values: `DATABASE_URL`, `VALKEY_URL`,
`HQ_CONFIG`, `HQ_ROOT`, `HQ_SECRETS_DIR`, `HQ_SERVER`, `HQ_NODE_BIN`,
`HQ_LAUNCHER_DRY_RUN`, and `LLM_API_KEY` as the fallback for the
`.secrets/llm.api_key` file (`docs/superpowers/specs/2026-07-12-hq-command-center-design.md`,
`crates/hq-core/src/config.rs`).

## Repository Layout

```
crates/
  hq-core/        registry parse+edit, config, secrets, store/* (one module per table)
  hq-adapters/    provider I/O: google, msgraph, slack, linear, jira, beads, git/gh, timeouts
  hq-corpus/      ingest (scan/extract/chunk/categorize/pipeline), llm/ clients, mine contract
  hq-channels/    Channel port, planner, engine, six channels, shared triage pass, fakes
  hq-jobs/        queue, worker, repo (jobs SQL), enqueue (approval policy), handlers (14)
  hq-server/      api.rs (REST), mcp/ (16 tools), background.rs (4 schedulers), sse.rs, main.rs
  hq-cli/         main.rs (clap), client.rs (HTTP), up.rs (launchd stack control)
frontend/         Astro 5 SSR + Vue 3 dashboard on :7718 (components/, lib/, pages/api/)
tauri/src-tauri/  hq-shell — workspace-EXCLUDED Tauri v2 window + tray (own Cargo.lock)
migrations/       001..015 SQL, applied by sqlx
skills/           hq/ (agent contract), hq-dev/ (contributor map), hq-onboard/ (registry wizard)
scripts/          dev-check.sh (the gate), install-launchd.sh, uninstall-launchd.sh
docs/             superpowers/{specs,plans}, adr/, runbooks/, onboarding/, brand/, screenshots/
registry.toml     THE WORLD — clients/projects/channels/infra (gitignored; example checked in)
config.toml       runtime dials (gitignored; example checked in)
```

Entry points: `crates/hq-server/src/main.rs` (the daemon that is also the worker
pool, the scheduler and the MCP server), `crates/hq-cli/src/main.rs` (the `hq`
binary), `frontend/src/pages/index.astro` (the dashboard),
`tauri/src-tauri/src/main.rs` (the shell), and `skills/hq/SKILL.md` (what an
agent loads to drive hq).

## How It Was Built

**Toolchain.** Rust stable, Node for the dashboard, `cargo tauri` only for the
shell. No pinned versions beyond the workspace dependency table and the
`pgvector = "=0.4.1"` exact pin.

**Build / run / test — as they really are.** `cargo build` produces `hq-server`
and `hq`; `(cd frontend && npm install && npm run build)` produces the dashboard
bundle; `cargo build --release` is not part of the loop — the launchd plist runs
`target/debug/hq-server` deliberately, to avoid multi-minute release builds. The
test surface is large and in-tree: 381 `#[test]`, 180 `#[tokio::test]` and 9
`#[sqlx::test]` across the workspace, with `wiremock` standing in for every
provider. The single end-to-end gate is `scripts/dev-check.sh` (931 lines), which
stands up a hermetic server and drives an MCP gate (initialize + `tools/list`
asserting exactly 16 tools, `hq_projects` present, `hq_approve`/`hq_reject`
absent), a triage gate, a work gate and a work-write gate — the last of which
flips `auto_write` on, proves a client write can only reach an unreachable
localhost endpoint, and flips it back off.

**Dev loop.** `hq up` brings the whole stack to green idempotently; the two
launchd agents (`co.dev916.hq.server` on :7717, `co.dev916.hq.frontend` on :7718)
survive logout and restart on crash, with logs under `~/Library/Logs/hq/`.
`scripts/install-launchd.sh` performs a takeover — it boots out prior copies,
kills whatever squats on the ports, resolves the real `node` path (launchd has no
nvm), and blocks until `/health` answers. Valkey stays under Homebrew and is not
managed by the agents.

**CI/CD and deploy path.** There is none, and that is consistent: hq is a
single-user local daemon, so "deploy" is `cargo build` plus re-running the
launchd installer.

**Configuration.** `config.toml` (gitignored, example checked in) carries
`[schedule]`, `[policy]`, `[agent]`, `[launcher]`, `[adapters]`, `[llm]`,
`[triage]`, `[work]`, `[calendar]`; `.env` carries `DATABASE_URL`, `VALKEY_URL`,
`HQ_CONFIG`. `registry.toml` is also gitignored because it holds real client
identifiers. Both are loaded from the process working directory, which is why the
launchd plists set `WorkingDirectory` to the repo root and the frontend's to
`frontend/`.

**Provenance.** 310 commits between 2026-07-12 and 2026-09-02, every one authored
by `web-mech`, in an unmistakable agent-built cadence: 248 commits in July, 46 in
August, 16 so far in September. The rhythm is spec → plan → devloop → beads sync,
visible in 12 design specs and 27 implementation plans under
`docs/superpowers/`, one ADR, and commit subjects that name the beads issue
(`feat(hq-oze P2-T5): …`, `chore: beads sync (…)`). The repo tracks its own work
in beads: 180 records, 141 closed, 27 open, 1 in progress. `CLAUDE.md`
additionally documents a per-session actor convention (`hq-agp`) in which every
Claude Code session exports `BEADS_ACTOR="web-mech/cc-$CLAUDE_CODE_SESSION_ID"`,
making `bd update --claim` a cross-session mutex whose failure message names the
owning session uuid.

## Relationships

- **The ownership axis for this corpus.** hq's registry is the canonical map of
  who owns what: 7 clients (`blackmast`, `fenzi`, `nexian`, `personal`,
  `pricelove`, `revenium`, `unyform`) and 14 projects (`blackmast-prospector`,
  `fenzi-fdsa`, `nexian-ghl`, `personal`, `webmech`, `ghostnn`, `hq`,
  `meetnotes`, `pricelove`, `revenium-cadmv`, `revenium-emea`, `revenium-iniot`,
  `revenium-platform`, `unyform`). Verified live via `hq_projects` against
  `registry.toml` — the two agree exactly. The path from TOML to that list is:
  `registry::load` parses and validates → `mirror_registry` upserts `clients`
  and `projects` in one transaction, keyed on `slug` with `ON CONFLICT DO
  UPDATE` → `list_projects` is what the CLI, `/api/projects` and `hq_projects`
  all read. Other repo profiles should use the **project slug** as the
  cross-repo ownership key, and set their own `hq_project:` frontmatter to it.
- **Ancestor of mech-crate's techniques corpus.** `crates/mx-lib/src/corpus/mod.rs` in mech-crate opens with "Ported from the hq corpus pattern (~/dev/hq)", its `chunk.rs` names its paragraph packer the "hq `chunk_text` port", and `embed.rs` cites an "hq convention" for the embeddings batch size. The inheritance is deeper than the comments say: both share `DEFAULT_CHUNK_CHARS = 1200`, and both compute the identical hybrid score `0.85 * (1 - (embedding <=> $1)) + 0.15 * similarity(content, $2)` (`crates/hq-core/src/store/corpus.rs` vs `crates/mx-lib/src/corpus/store.rs`). mech-crate's chunker is the *evolved* one — it added heading-aware `##` splitting and `Doc Title > Heading` prefixing on top of hq's flat packer. hq's own lineage goes one step further back: its `chunk.rs` header records it as ported from blackmast-prospector, which is also where its corpus content was seeded from via `hq corpus import-blackmast` (embeddings copied, not re-computed). See `docs/development/repos/mech-crate.md`.
- **meetnotes** — a two-way relationship. hq's summarizer engine chain can delegate `mine` to meetnotes over HTTP (`[triage] meetnotes_url`) or stdio (spawning `~/.claude/skills/meetnotes/meetnotes_mcp.py`), falling back to the native engine on failure; and meetnotes pushes notes back through hq's `hq_push_note` door. A pinned `mcp<2` in the stdio command is a live workaround: "mcp 2.x renamed FastMCP and silently killed the fallback engine" (commit, 2026-09-01). Open issue `hq-0gl` would split the meeting Join button to start a meetnotes recording. See `docs/development/repos/meetnotes.md`.
- **mech-crate's technique-research skill** lists `hq-corpus` as a *planned* source provider — "topic may overlap internal knowledge (business/ops/prior research)", queried via `mcp__hq__hq_corpus_search` (`skills/technique-research/references/source-providers.md`). Not yet wired.
- **beads (`bd`)** is both hq's own tracker and one of its three tracker providers: a project whose registry `tracker` block names `provider = "beads"` gets its board synced by `sync.tracker`, and `work.write` creates against a beads repo flow **auto** while linear/jira creates are approval-gated.
- **Sibling profiles** (forward links, may not exist yet): `docs/development/repos/meetnotes.md`, `docs/development/repos/understudy.md`, `docs/development/repos/a2a.md`, `docs/development/repos/mech-crate.md`. No code, config or documentation reference to a2a, understudy or mx exists anywhere in hq — the coupling runs the other way.
- **Canonical copy.** There is exactly one: `~/dev/hq`, level with `origin/main`. The three skills under `skills/` are **not** installed under `~/.claude/skills` or `~/.codex/skills` (both checked — no `hq*` entries), so no forked copy can drift. The MCP server, by contrast, *is* registered user-scope, which means agents get hq's tools everywhere but its documented contract only inside the repo.

## Notable Techniques

- **The strangler fig, run to completion and then torn down.** `hq-kgw` inverted six hard-coded sync paths into a `Channel` trait engine across five phases, proved byte-for-byte parity live before each cutover (19 `sync_state` rows and 151 `git_refs` rows identical on both paths), then on 2026-07-29 *deleted* the eleven legacy branches, the cutover flag and the pre-engine writers. The ADR is amended rather than rewritten and carries a reading note that earlier amendments are history. "Rollback is `git revert`, not a config dial" (`docs/adr/0001-channel-trait.md`).
- **A transactional outbox that makes double-sends structurally impossible.** One `write_outbox` row per write job (`job_id UNIQUE`), status ∈ `pending|sending|sent|failed_clean|failed_ambiguous`, and a fixed handler order: find-or-insert by `job_id` (an existing `sent` row short-circuits with the recorded key and makes **no** provider call) → claim `pending → sending` in guarded SQL → exactly one provider call → record. The completion lookup runs *before* each seam's own idempotence guard, because the ledger, not the item, is the authority on whether this job's write happened.
- **Ambiguity as a terminal state.** A failure is *clean* only when the provider provably did not accept it; a timeout or 5xx is *ambiguous* and nothing ever re-opens it — the honest resolution is a human checking the tracker and approving a fresh job with a new id. "At-least-once intent, at-most-one un-verified attempt" (`README.md`).
- **A persisted FSM with a lease instead of a lock.** `sync_state` is an explicit state machine (`ok`/`syncing`/`error`/`auth_required`/`quarantined`) over a stable `binding_key`; `step()` is a pure, property-tested, total transition function with an injected clock, and the interpreter applies each transition in one UPDATE. `SyncStarted` *is* the lease acquisition, so one-writer-per-binding costs no lock and no long transaction (`crates/hq-core/src/store/sync_fsm.rs`).
- **Plan-time gating with cheap auth probes.** Before enqueuing, the fan-out asks each binding's FSM row whether it is worth trying: not-yet-due errors and quarantined bindings are skipped, and an `auth_required` binding gets `Channel::probe_auth` — a *file read only, never a network round-trip* — so restored credentials auto-resume. Direct single-source jobs are never gated, so a human always has an escape hatch out of quarantine.
- **Cooperative deadlines, never hard kills.** Long handlers check a budget *between* units of work and stop with an honest partial result (`deadline_hit` plus a remaining count), banking progress and freeing the worker. Slack cursors advance per-conversation after mining, so a budget cut re-fetches only the unmined ones and sha-dedupe makes the overlap free.
- **Fail-open capability affordances.** The dashboard disables buttons the binding's state does not allow, but treats missing capability rows as "enabled" and treats `capabilities`/`actions`/`state` as open vocabularies to be membership-tested, never exhaustively matched — an advisory refinement, not a second enforcement point (`frontend/src/lib/capabilities.ts`).
- **Approval as an enqueue-site invariant.** Policy lives where jobs are created, not where they run: `Channel::write` only ever executes an already-approved job, and there is no approve tool on MCP at all. The dev-check gate asserts the absence, which turns a design decision into a test.
- **Config-driven engine chains.** `[triage] engines = ["native:ollama", "native"]` makes a local Ollama the primary summarizer and a cloud model the fallback with no code change, and every job result records `engine_used`, `tokens_in`, `tokens_out` and `estimated_cost` per project. Engines that report no usage record `null` rather than a fabricated number (`README.md`).
- **Backlog candidates** (not filed here, per the profiling procedure): *transactional outboxes and the clean/ambiguous failure taxonomy*; *persisted FSMs as a sync-state plane* (lease-as-transition, plan-time gating, backoff/quarantine); and *strangler-fig migrations with live A/B parity proof and a scheduled teardown*.

## State, Gaps and Drift

**Maturity.** Well past prototype: 310 commits over seven weeks, 570 tests, 15
migrations, an ADR, 12 specs and 27 plans, a 931-line end-to-end gate, and a
functioning launchd service. Exactly one literal TODO/FIXME/HACK marker exists in
the whole tree (and it is a `mktemp` template, not a marker). Deferrals are
tracked in beads instead.

**README-vs-code drift.**
- `README.md` says the MCP surface "carries 15 tools" and lists 15, omitting
  `hq_schedule_meeting`. The code registers 16 and `scripts/dev-check.sh` asserts
  `len(n)==16`. `skills/hq/SKILL.md` has it both ways — its heading says 16, its
  ops section says 15 twice.
- `hq corpus search --category` is documented in `README.md` *and*
  `skills/hq/SKILL.md`; the clap enum has only `--project`, `--k`, `--sensitive`.
  The REST and MCP paths do support `category`, so the gap is CLI-only.
- `README.md` has two `## Calendar auth` headings (lines 430 and 564); the second
  is empty and immediately followed by `## Triage`.
- The `[llm]` example model is `gpt-5-mini` in the triage section and
  `gpt-5.5-mini` in the corpus section.
- `skills/hq-dev/references/architecture.md` is the stalest doc in the repo:
  it states the worker pool is "hard-coded `for i in 0..2`" (it is
  `config.work.worker_count`, default 4), that there is "No `XAUTOCLAIM`/
  pending-reclaim" (boot reclaim landed in `hq-2ah`), and that
  `default_handlers` "registers 13" (it registers 14 — `triage.sweep` is
  missing from its table).

**Environment drift.** `~/.local/bin/hq` symlinks to `target/debug/hq`, whose
mtime is 2026-07-29, while the newest commit touching `crates/hq-cli` is
2026-08-17 (`hq-6n0` Phase 2: composite board sort, keyset pagination, search) —
so the CLI a human invokes is a build behind its source. `target/debug/hq-server`
is current (2026-09-02). A built `hq.app` bundle exists, but `tauri/` has had no
functional commit since 2026-07-14 (only the brand icon set).

**Working-tree state at profiling time** (pre-existing, not caused by this
profile): `.beads/issues.jsonl` staged-modified, plus untracked `.playwright-mcp/`
and one `registry.toml.bak.<epoch>` — the latter is the backup the `hq-onboard`
skill's guardrail requires before a destructive registry edit.

**Open issues worth knowing about** (27 open, 1 in progress). Operational:
`hq-31l` the `hq:jobs` Valkey stream is never trimmed (70k+ historical entries)
and `hq-khi` `queue_depth` therefore reports cumulative `XLEN` rather than
pending work; `hq-m31` the launchd plists lack a `PATH`, so the shelled-out
`gh`/`git`/`bd`/`slackcli` binaries are not found under launchd; `hq-261` bd
writes from the hq daemon hang past 300s, degrading the auto-beads lane.
Correctness: `hq-rzn` `mirror_registry`'s prune can cascade-delete real data if
the server boots with a wrong registry — the most dangerous open item, since the
mirror is the ownership axis everything else keys off; `hq-73w` concurrent
`corpus.scan` runs race to a duplicate key; `hq-47p` `corpus.scan --force` has
been a silent no-op since the 2a config-plane cutover. Performance: `hq-5f0`
corpus hybrid search full-scans ~8.5s over 11.7k chunks because the blended score
cannot use an index. Documentation: `hq-cd0` the dev-check triage and calendar
gates are stale since the Web API cutover.

**Specs stop before the code does.** `docs/superpowers/specs/` ends at the
2026-07-29 teardown, while `docs/superpowers/plans/` continues through
2026-09-01. Three later cycles (`hq-pi1`, `hq-svv`, `hq-oze`) shipped with a plan
and a design commit but no file in the specs directory — a break in the otherwise
strict spec → plan → devloop discipline.

### Synthesis (inferred)

hq is best understood as **one idea applied four times**: name the seam, put the
policy at the seam, and make the seam the only path. The `Channel` trait is that
idea for reading sources; the outbox is that idea for writing to them; the
enqueue site is that idea for approval; and the registry is that idea for
identity. Each was built beside the thing it replaced, proven at parity against
real data, then made exclusive by *deleting* the alternative. That is why a
codebase this young has almost no TODOs — the deferrals are either beads issues
or deliberate absences, and the absences are asserted by tests.

The approval gate is the load-bearing design decision, and it is enforced
structurally rather than by convention: there is no MCP approve tool, the
frontend's direct-to-queue route rejects approval-gated job types with a 400, the
generic enqueue path gates `work.write` safe-by-default so a raw `POST /api/jobs`
cannot slip one through, and dev-check asserts the absence of `hq_approve` on
every run. An agent with full MCP access to hq can draft anything and send
nothing. Any future "autopilot" is therefore a matter of flipping named dials one
at a time, not of building new machinery — which is exactly what the spec
predicted in July.

The relationship to mech-crate is worth stating plainly because it inverts the
usual direction: hq is the *upstream* here. mech-crate's techniques corpus — the
thing this profile is being written into — is a port of hq's corpus subsystem,
which was itself a port of blackmast-prospector's. Three generations of the same
doc/chunk/sha256/hybrid-search pattern, each one generalizing further: bid
prospects → company knowledge → engineering techniques. The chunker is the
clearest fossil: mech-crate kept hq's paragraph packer verbatim as an inner
function and wrapped heading-awareness around it.

The one real structural risk is `hq-rzn`. Because `registry.toml` is gitignored
and `mirror_registry` prunes rows absent from it, a server booted against a
truncated or wrong registry can cascade-delete live data — and the registry is
now load-bearing for more than hq itself, since this corpus proposes to key
cross-repo ownership on its project slugs. The cheapest mitigation is a refusal
to prune below a sanity threshold; the more durable one is a checked-in
identifier-only skeleton the real file is merged over.

## Quick Reference
| Task | Command / path |
|---|---|
| Build | `cargo build` (server + CLI); `(cd frontend && npm install && npm run build)` |
| Run the whole stack | `hq up` (idempotent) · `hq up --no-app` headless · `hq down [--valkey]` |
| Tests | `cargo test`; end-to-end gate `scripts/dev-check.sh`; shell tests `cd tauri/src-tauri && cargo test` |
| Health | `hq status` · `curl 127.0.0.1:7717/health` · `curl 127.0.0.1:7717/api/status` |
| Dashboard | `http://127.0.0.1:7718` (Astro SSR) — API at `http://127.0.0.1:7717` |
| MCP endpoint | `POST http://127.0.0.1:7717/mcp` — registered for Claude Code as `hq` |
| Register MCP | `claude mcp add --transport http hq http://127.0.0.1:7717/mcp --scope user` |
| Agent contract | `skills/hq/SKILL.md` (tools, approval policy, corpus gates) |
| Contributor map | `skills/hq-dev/SKILL.md` + `skills/hq-dev/references/architecture.md` (partly stale) |
| Add a client/project | invoke the `hq-onboard` skill — never hand-edit `registry.toml` |
| Describe the world | `registry.toml` (gitignored) — schema in `registry.example.toml` |
| Runtime dials | `config.toml` (gitignored) — schema in `config.example.toml` |
| Service logs | `~/Library/Logs/hq/server.log`, `~/Library/Logs/hq/frontend.log` |
| launchd labels | `co.dev916.hq.server` (:7717), `co.dev916.hq.frontend` (:7718) |
| Install/remove service | `scripts/install-launchd.sh` · `scripts/uninstall-launchd.sh` |
| Rescue a stuck source | `hq sync retry <binding_key>` · `hq sync all --force` |
| Release a draft | `hq approvals` then `hq approve <id>` / `hq approve --all` (human-only) |
| Issue tracker | beads in `.beads/` — `bd ready`, `bd list --status=in_progress` |
| Design spec | `docs/superpowers/specs/2026-07-12-hq-command-center-design.md` |

## Sources

- `README.md` (1,100 lines) — quickstart, launchd service model, job-pool resilience, the channel engine's five phases, the sync-state FSM, the write plane and outbox, registry channels, calendar auth, triage engines and cost accounting, work board, corpus. The densest source, and the origin of most recorded drift.
- `docs/superpowers/specs/2026-07-12-hq-command-center-design.md` — purpose, decomposition order, non-goals, topology, the credential-reuse and secrets rules, the job model, and the planner-agent design that has not shipped.
- `docs/adr/0001-channel-trait.md` — the strangler-fig decision, its amendments, and the teardown that closed it.
- `skills/hq/SKILL.md` — the agent-facing contract (16 tools, CLI verbs, REST surface, push doors, approval policy). `skills/hq-onboard/SKILL.md` — the discover → confirm → merge → validate registry flow. `skills/hq-dev/references/architecture.md` — crate boundaries and boot flow, used for the component table; its worker-pool, XAUTOCLAIM and handler-count claims were checked against code and found stale.
- `crates/hq-cli/src/{main.rs,up.rs}` — the real CLI surface (clap doc comments) and stack control.
- `crates/hq-server/src/{main.rs,api.rs,background.rs,launcher.rs}` and `crates/hq-server/src/mcp/tools_{read,write,triage}.rs` — boot order, the 41 routes, the CORS allow-list, the four schedulers, the profile-aware launcher, and the authoritative 16-tool list.
- `crates/hq-jobs/src/handlers.rs` (`default_handlers`, 14 registered; the `agent.run` body) · `crates/hq-channels/src/port.rs` (the `Channel` trait) · `crates/hq-core/src/{config.rs,store/mirror.rs,store/corpus.rs}` (defaults, the registry → DB upsert, the hybrid-search SQL) · `crates/hq-corpus/src/ingest/chunk.rs` (`chunk_text` and its blackmast origin note).
- `migrations/001..015` — the 17-table schema, `vector` + `pg_trgm`, FSM columns, write outbox, triage sweep.
- `frontend/{astro.config.mjs,package.json,src/pages/api/sync.ts,src/lib/capabilities.ts}` — SSR setup, the direct-to-queue route, the fail-open affordance rules. `tauri/README.md` + `tauri/src-tauri/Cargo.toml` — the shell's scope, the workspace-exclusion rationale, what its tests assert.
- `scripts/dev-check.sh` — the gates, and the tool-count assertion that settles 15-vs-16. `CLAUDE.md` / `AGENTS.md` — the beads workflow and the `BEADS_ACTOR` convention. `.beads/issues.jsonl` — 180 records; the open-issue list above.
- mech-crate: `crates/mx-lib/src/corpus/{mod.rs,chunk.rs,store.rs}` (port comments, shared chunk size, identical hybrid-score expression) and `skills/technique-research/references/source-providers.md` (hq-corpus as a planned research source).
- Repo metadata via `git log`/`git ls-files`/`git status` and `gh api repos/Dev916/hq`; the live project list via the `hq_projects` MCP tool; installed-copy checks against `~/.claude/skills`, `~/.codex/skills` and `~/.claude.json`.
