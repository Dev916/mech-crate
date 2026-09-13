---
title: "a2a: Claude-orchestrated Codex workers behind a crash-safe file layer (Repo Profile)"
category: repos
languages: [rust, markdown, json, sql]
complexity: advanced
use_cases:
  - "understanding what a2a does and where its code lives"
  - "finding a2a's CLI surface — spawn/wait/verify/mesh — before extending it"
  - "answering 'which repo dispatches Codex workers from a Claude session'"
  - "resuming work on a2a in a fresh session"
summary: "a2a is a single Rust binary (~57k lines of src, ~41k of tests) that lets a Claude Code session dispatch OpenAI Codex CLI workers, watch them, steer them at turn boundaries and reap them — from a CLI whose exit code is the answer, so an orchestrator never burns context polling. Worker state is a per-worker write-ahead log (events.jsonl) plus an atomic snapshot (state.json) under ~/.a2a, so nothing is lost when the CLI, the shell or the machine dies mid-turn. Two transports share one CLI: a broker daemon owning a private `codex app-server` child (mid-turn interrupt, live tail, admission caps, circuit breaker, turn budgets) and a detached `codex exec` process group with no daemon. Four lanes fix sandbox/model/effort; briefs carry acceptance_criteria/verify/files_in_scope/non_goals front matter; impl workers run in their own git worktree on their own branch and `a2a verify` re-runs the brief's commands — never the worker's — behind an execution-bearing-diff refusal. On top of that sit a same-machine agent mesh (presence, feed, directed notes), a cross-machine mesh fleet over a Postgres registry with pinned-TLS gateways, and a signed work-handoff layer. Private (Dev916), Rust, v0.3.0, actively developed."
provenance: researched
researched: 2026-09-02
publish: false
repo: https://github.com/Dev916/a2a
local_path: ~/dev/a2a
status: active
visibility: private
owner: PriceLove LLC (Dev916)
sources:
  - README.md (target repo)
  - CLAUDE.md, AGENTS.md (target repo)
  - src/cli.rs (target repo)
  - src/commands/gen_skills.rs (target repo)
  - src/commands/verify.rs (target repo)
  - src/commands/doctor.rs (target repo)
  - src/config.rs (target repo)
  - src/brief.rs (target repo)
  - src/home.rs (target repo)
  - skills/orchestrate-protocol.md (target repo)
  - skills/worker-protocol.md (target repo)
  - templates/impl-brief.md, templates/impl-result.schema.json (target repo)
  - docs/machines/worker.md (target repo)
  - docs/superpowers/specs/2026-08-14-a2a-claude-codex-design.md (target repo)
  - docs/superpowers/plans/*.md (target repo)
  - docs/mesh-work.md (target repo)
  - schema/v2-snapshot/REGEN.md (target repo)
  - docs/development/multi-agent-systems-in-practice.md (mech-crate)
---

# a2a

> **"The exit code is the answer."** a2a is a single Rust binary that lets one
> agent run another: a Claude Code session (or a script, or a Makefile) spawns
> OpenAI Codex CLI workers, waits on them, answers them when they block, and
> verifies what they produced — without ever reading a transcript. Every command
> answers in one call and encodes its verdict in the process exit code, because
> the resource being protected is the orchestrator's context window. Worker state
> is a write-ahead log plus an atomic snapshot on disk, so a killed CLI, a dead
> broker or a rebooted machine loses nothing. Around that core it has grown a
> broker daemon with admission control and a circuit breaker, git-worktree
> isolation with an independent verification gate, a one-way capability mirror
> into the Codex home, and a three-layer agent mesh that now carries signed work
> handoffs between machines.

## Identity
| Field | Value |
|---|---|
| Repository | `Dev916/a2a` (private) — default branch `main` |
| Local path | `~/dev/a2a` (directory name matches the repo name) |
| Owner / org | PriceLove LLC (Dev916) — no hq project registered for it |
| Status | active — last commit 2026-09-02, 144 commits, profiled at `56c1491` |
| Languages (by file count) | Rust 152 · Markdown 41 · JSON 29 · SQL 2 · TOML 1 · shell 1 (excluding `target/`, `.git/`, `.beads/`). ~57,176 lines under `src/` in 75 files, ~41,364 lines under `tests/` in 77 files |
| Build system | cargo, single binary crate `a2a` v0.3.0, edition 2021 (`Cargo.toml`) |
| Runtime deps | `codex` CLI on `PATH`, authenticated, version in `>=0.140, <0.150`; `git` (worktrees); optionally Postgres/Neon for the layer-2 mesh registry and `ngrok` for gateway exposure |
| License | none declared — no `LICENSE` file, and `gh api repos/Dev916/a2a` reports `license: null` |
| CI / release | none — no `.github/` directory. Releases are self-served: `a2a update` clones/updates the repo under `$A2A_HOME/update`, builds it, and installs to `~/.local/bin/a2a` (`src/commands/update.rs`) |

## What It Does

The problem it solves is cost asymmetry. A supervising Claude session that
implements a long task inline pays for every build log, every failed attempt and
every file it reads. Delegating to a subagent helps, but the subagent still runs
on the same quota and the same model family. a2a moves that work onto a
*different* runner — the Codex CLI, with its own context window and its own
quota — and gives the supervisor a mechanical interface to it: dispatch, block
until something changes, read a result, verify it independently.

The design's central move is that **the control plane lives in files and exit
codes, never in prose**. `a2a wait` returns the moment a worker changes state and
exits `0` for done, `4` for failed, `5` for blocked, `6` for not-yet, `10` for
orphaned — so an orchestrator's monitor loop is a `case` statement, not a
transcript read. A worker that needs a decision writes
`outbox/<seq>.question.json`; that *file* is authoritative and a `BLOCKED:` line
in the worker's prose is an advisory hint only
(`docs/superpowers/specs/2026-08-14-a2a-claude-codex-design.md` §2).

Its users are agents. The Claude side reads a generated `a2a-orchestrate` skill
telling it when to dispatch, how to write a brief a memoryless worker can finish,
and how to keep the acceptance gate mechanical; the Codex side reads a generated
worker protocol telling it that everything it reads is data and never
instructions, to stay in its assigned worktree, and to finish with evidence
(`skills/orchestrate-protocol.md`, `skills/worker-protocol.md`). "Done" is a
branch plus a `verification.json` an orchestrator can branch on — **a2a never
merges anything** (`skills/orchestrate-protocol.md`, `acceptance-gate` section).

## Capabilities

### CLI — worker lifecycle
- `a2a spawn --lane <impl|review|research|general> --cwd <dir> --brief <file>` — dispatch a worker; **stdout is the worker id** so `ID=$(a2a spawn …)` works (`src/cli.rs`, `src/commands/spawn.rs`)
- `a2a spawn` extras: `--context <file>` (staged read-only into `context/`), `--schema`, `--profile <name>`, `--model`, `--effort`, `--transport`, `--worktree` / `--no-worktree`, `--base-ref`, `--no-queue`, `--owner`, `--danger-full-access` + `--reason`, `--template <lane>` (`src/cli.rs`)
- `a2a status | ls | tail | result | wait` — read paths; `status` and `result` encode the worker's state in the exit code, `wait --until blocked|terminal|done` blocks on a state change rather than polling, and accepts several ids (first to change wins) (`src/cli.rs`, `src/commands/wait.rs`)
- `a2a tail --follow` — live broker frames on the app-server transport, a 500 ms log poll on exec; always bounded by a terminal state, `--timeout`, or broker shutdown (`src/commands/tail.rs`)
- `a2a send | interrupt | kill | resume | respawn | gc` — steer, end the open turn, witnessed group-kill, continue in a new epoch, retry with lineage, reclaim terminal workers with a tombstone (`src/commands/{send,interrupt,kill,resume,respawn,gc}.rs`)
- `a2a verify <id>` — re-run the **brief's** verify commands in the worker's workspace and write `verification.json` (`src/commands/verify.rs`)
- `a2a doctor` — 15 environment checks, exit 9 if any fails (`src/commands/doctor.rs`)
- `a2a config show [--origin]`, `a2a config add-root <dir>` (`src/commands/{config_show,config_roots}.rs`)
- `a2a update [--check] [--channel tag|main]` — self-updater: clone/refresh, build, install, verify version, restart the gateway service (`src/commands/update.rs`)
- `a2a broker run|status|shutdown|restart|reset-breaker` (`src/commands/broker_cmd.rs`)
- `a2a sync skills|mcp [--dry-run]` (`src/commands/sync_cmd.rs`, `src/sync/`)
- Hidden verbs: `__gen-skills`, `__exec-shim` (the detached exec child that holds `run.lock` and lands the exit code), `__broker-call` (one raw request over the broker socket), `__selftest-error` (`src/cli.rs`)

### CLI — mesh
- Layer 1 (same machine): `mesh register|heartbeat|retire|status|post|feed|note|inbox|roster|inject|hooks` (`src/cli.rs`, `src/commands/mesh_cmd.rs`)
- Layer 2 (cross machine): `mesh fleet init|ls|migrate|revoke|enroll|rotate-root`, `mesh gateway run|status|stop|install-service|uninstall-service|identity|install-token`, `mesh links` (`src/cli.rs`, `src/mesh/registry.rs`, `src/mesh/keys.rs`)
- Layer 3 (work handoff): `mesh work send|offer|list|show|approve|decline|cancel|status|ack|retry-push` (`src/cli.rs`, `src/commands/work_cmd.rs`, `src/mesh/work.rs`)

### Lanes, briefs and result schemas
- Four lanes fix sandbox, network and default effort: `impl` = workspace-write / network off / effort high / worktree by default; `review` = outbox-only / off / medium; `research` = outbox-only / **network on** / medium; `general` = workspace-write / off / medium (`src/config.rs`)
- Per-lane turn budgets — impl 40, general 30, research 20, review 15; a brief may only *lower* its own, and asking for more is clamped loudly at spawn (`src/config.rs`, `README.md`)
- Brief front matter keys: `lane`, `acceptance_criteria`, `verify`, `files_in_scope`, `non_goals`, `result_schema`, `max_turns`; `impl` requires `acceptance_criteria` + `verify`, `review` requires `acceptance_criteria` (`src/brief.rs`)
- Brief scaffolds are compiled into the binary — `a2a spawn --template <lane>` prints one and creates nothing, so every machine answers identically (`src/cli.rs`, `templates/*-brief.md`)
- The impl lane's result schema **is** devloop's `sample-task-result.json` shape — `{status, iterations, summary, evidence, files_changed, next_step_hint}` — materialized into each impl worker as `result-schema.json` and enforced at the turn boundary (`templates/impl-result.schema.json`)

### Generated agent skills
- `a2a __gen-skills --out <dir> [--check]` renders exactly three artifacts and **writes nowhere else** — no symlink into `~/.claude/skills`, no copy into `~/.codex/skills`, no edit to `~/.codex/AGENTS.md`; installation is a separate, consent-bearing step, pinned by a test that runs the command with `HOME` and `CODEX_HOME` at an empty tempdir (`src/commands/gen_skills.rs`, `tests/skills_gen.rs`)
- `skills/worker-protocol.md` → `codex/a2a-worker/SKILL.md` and `codex/AGENTS-section.md`; `skills/orchestrate-protocol.md` → `claude/a2a-orchestrate/SKILL.md` (`src/commands/gen_skills.rs`, constant `ARTIFACTS`)
- Sources are sliced on `<!-- a2a:section id -->` … `<!-- a2a:end id -->` HTML markers — invisible in every renderer, greppable, and immune to a heading inside a fenced code block. Section id lists are pinned (7 worker, 11 orchestrator); a missing, renamed or reordered section is a source error (`src/commands/gen_skills.rs`)
- `--check` is a byte-comparison drift gate where a missing file counts as drift (`src/commands/gen_skills.rs`)

### Fleet survival
- Admission control: `[admission] global_cap`, `queue_cap`, per-lane caps; over-cap spawns **queue** with a `queue_position` rather than failing, and every cap is a `u64` where `0` means "admit nothing". A slot is released only on `WorkerSettled`, unclaimed reservations expire after 60 s, and **exec spawns are deliberately uncapped** — the broker never learns an exec worker finished, so a slot spent on one could never be freed (`src/config.rs`, `src/broker/core.rs`)
- Circuit breaker: `breaker_failures` settlements inside `breaker_window_secs` stops admitting new work (`BREAKER_OPEN`, exit 9); adoption of already-running workers is never refused; `breaker_failures = 0` disables it — the one cap where `0` does not mean "stop everything" (`README.md`)
- Respawn lineage: `parent_id` / `attempt` / `max_attempts` (default 3), `ATTEMPTS_EXHAUSTED` on the fourth try, `PARENT_ACTIVE` while the parent still runs (`src/commands/respawn.rs`)
- `auto_review` (impl lane, off by default): a finished impl worktree is diffed and handed to a review-lane worker whose findings land as `review.json`; reviewers are never themselves reviewed (`README.md`, `tests/auto_review.rs`)
- `events.jsonl` rotation at `[broker] events_max_bytes`, one generation kept, with a `discarded_prior_generation` marker on the second rotation (`README.md`, `tests/events_rotation.rs`)

### Not (yet) implemented
- Guardian-denial parking and `a2a approve` — `guardianWarning` is recorded as a lifecycle outbox notice with no state change; the parking behaviour is still deferred (`src/events.rs`, `docs/superpowers/plans/2026-08-17-a2a-phase2-broker-app-server.md`)
- Turn budgets are counted on both transports but **enforced broker-side only**: an exec worker's `turns_this_epoch` is reported and nothing acts on it (`README.md`)
- Multi-generation `events.jsonl` retention is deliberately deferred (`README.md`)
- `--strict-config` as the post-write MCP validator was abandoned as nondeterministic on codex 0.140.0; `codex mcp list --json` replaced it, at the cost of tolerating unknown fields (`README.md`)

## Architecture

**Stack.** Rust 2021, one binary crate. tokio (multi-thread rt, process, net,
io-util, sync, time, signal) with `tokio-util`'s `TaskTracker`; clap 4 derive for
the CLI; serde/serde_json for the file layer and JSON-RPC; `toml` for reading and
`toml_edit` 0.25 for comment-preserving writes into somebody's hand-maintained
`~/.codex/config.toml`; `nix` for `LOCAL_PEERCRED`/`SO_PEERCRED` peer-uid checks
and process signalling; `ed25519-dalek` + `rcgen` + `rustls`/`tokio-rustls` +
`tokio-tungstenite` for the mesh gateway; `sqlx` (postgres) for the fleet
registry; `ulid` for ids (`Cargo.toml`).

**Component map.** `src/` splits into the file layer (`events.rs`, `state.rs`,
`store.rs`, `fold.rs`, `fsm.rs`, `reconcile.rs`, `home.rs`, `inbox.rs`,
`brief.rs`, `redact.rs`, `ids.rs`), the two transports (`exec.rs`; `broker/`
with `core.rs` at ~4,260 lines, `worker_actor.rs`, `child.rs`, `sockserv.rs`,
`wire.rs`, `shutdown.rs`; plus `brokerclient.rs`, `rpc.rs`, `protocol.rs`,
`adapter.rs`, `responder.rs`), the mesh (`mesh/` — `presence.rs`, `feed.rs`,
`notes.rs`, `routing.rs`, `keys.rs`, `registry.rs`, `wire.rs`, `work.rs`,
`work_store.rs`, `work_exec.rs`, plus `mesh/gateway/` — `mod.rs`, `link.rs`,
`replicate.rs`, `expose.rs`, `notes_wire.rs`, `presence_wire.rs`), the sync
mirrors (`sync/` — `skills.rs`, `mcp.rs`, `manifest.rs`), and one file per verb
under `commands/` (`spawn.rs` ~2,421 lines and `doctor.rs` ~1,756 are the
largest). Under `src/broker/`, `Arc<Mutex<…>>` and `Arc<RwLock<…>>` are
*forbidden* — state two tasks need is reached by message, and a source-scan test
enforces it; every channel is bounded, and the one stream that cannot
backpressure (the trace tail) drops the oldest and emits a synthetic
`EventsDropped` saying how many.

**Data flow.**

```
orchestrator (Claude/script)
   │  a2a spawn --lane impl --brief b.md
   ▼
CLI ── brief parse + secret scan + allowed-root check ── worktree create
   │
   ├── transport auto ──▶ broker daemon (one per $A2A_HOME)
   │                        └── one private `codex app-server` child
   │                              └── one thread per worker (thread id == codex session id)
   └── fallback ────────▶ detached `codex exec --json` process group (exec-shim holds run.lock)
                                   │
   every notification ─▶ events.jsonl (append + fsync, redacted at append time)
                                   │  fold()
                                   ▼
                             state.json  (temp → fsync → rename → fsync-dir)
                                   │
   a2a wait / status / ls ◀────────┘   exit code == state
   a2a verify  ──▶ re-run BRIEF's verify: commands ──▶ verification.json
```

**Storage.** Everything lives under `$A2A_HOME` (default `~/.a2a`, created 0700):
`workers/<id>/` (`brief.md`, `context/`, `outbox/`, `events.jsonl` [+ `.1`],
`state.json`, `state.lock`, `run.lock`, `prompt.md`, `result.md`,
`result-schema.json`, `verification.json`, `review.json`, `exit_code`),
`worktrees/<id>/`, `work/`, `mesh/`, `index.jsonl` (tombstones, so an id is never
reused), `config.toml`, `sync-manifest.json`, `audit.jsonl`, `LAYOUT_VERSION`,
and the broker's `broker.sock` / `.lock` / `.pid` / `.log` / `.jsonl` /
`broker-health.json` (`src/home.rs`, `README.md`).

**Worker state machine.** `docs/machines/worker.md` is declared the single source
of truth and mirrors `src/fsm.rs` 1:1, with `tests/fsm_table.rs` as the merge
gate. States: `spawned`, `queued`, `running`, `awaiting-input`, `escalated`,
`orphaned`, `done`, `failed`, `cancelled`, plus `unknown` as an open-enum landing
pad for a `state.json` written by a newer binary. `step()` is pure — no IO, no
clock, no randomness — time enters only as `Timeout { kind }` from persisted
absolute deadlines, and the match has **no catch-all arm**, enforced by a
source-scanning test. `FailureClass` has thirteen variants including
`budget_exhausted`, `schema_violation`, `sandbox_denied` and `session_missing`;
`cancelled` never carries one, so no retry policy can auto-retry a human stop.

**Protocol.** The app-server transport speaks codex's v2 JSON-RPC
(`thread/start`, `thread/resume`, `turn/start`, `turn/steer`, `turn/interrupt`
and the matching notifications). Because that protocol has **no version
handshake**, `schema/v2-snapshot/` holds a checked-in subset of
`codex app-server generate-json-schema` output and `tests/protocol_canary.rs`
byte-diffs a fresh regeneration against it — a token-free, auth-free drift alarm
(`schema/v2-snapshot/REGEN.md`).

**Mesh planes.** The mesh data plane is **not** the database: gateways talk to
each other over WebSocket-on-TLS (`tokio-tungstenite` over `tokio-rustls`) with
a serde-tagged frame enum (`Challenge`/`Hello`/`Feed`/`Note`/`Presence`/
`Digest`/`Want`/`Ack`/`Err`) and TLS pinned by SPKI fingerprint rather than PKI,
plus a fleet-root-signed proof-of-possession token in the handshake
(`src/mesh/wire.rs`, `src/mesh/keys.rs`). Postgres is only the **control
plane** — two tables, `mesh_gateways` (endpoint URL, key and cert fingerprints,
heartbeat, `revoked_at`) and `mesh_work_items` (addressing, status, a
compare-and-swap `claimed_by` claim, `result_ref`) — and it never stores feed
content, note text or presence (`migrations/*.sql`, `src/mesh/registry.rs`).

**Security model.** Briefs and context files are scanned for secrets on ingress
(spawn fails on a hit); events are redacted at append time so the WAL itself is
clean; every printed stream is redacted on egress (`src/redact.rs`).
`OPENAI_API_KEY`, `AWS_*` and `DEPOT_TOKEN` are scrubbed from every worker
environment, and the broker's app-server child gets an **allowlist** — `PATH`
(pinned at broker start), `HOME`, `USER`, `TERM`, `LANG`, `TMPDIR`, `CODEX_HOME`
— and nothing else. `broker.sock` is 0600, accepts only a peer whose uid equals
the daemon's, and the broker re-validates every wire-supplied `cwd` against the
allowed roots it was started with. No brief or message text ever appears in
argv — it travels through a 0600 prompt file on the child's stdin — and there is
no shell anywhere: every codex invocation is an argv array. `--ephemeral` is
never passed (it would kill resumability) and `danger-full-access` needs a lane
permission plus `--reason`, forces the exec transport, and appends to
`$A2A_HOME/audit.jsonl` (`README.md`, `src/cli.rs`).

## Repository Layout

```
src/
  main.rs cli.rs lib.rs        entry point, clap surface, crate root
  home.rs store.rs state.rs    $A2A_HOME layout, WAL append, snapshot write
  events.rs fold.rs fsm.rs     event mapping, projection, pure state machine
  reconcile.rs                 crash recovery, schema gate, outbox drain
  brief.rs inbox.rs redact.rs  brief front matter, message queue, redaction
  exec.rs                      detached `codex exec` transport
  broker/                      daemon: core, worker actors, app-server child, socket, wire
  brokerclient.rs rpc.rs       CLI-side client and JSON-RPC plumbing
  protocol.rs adapter.rs       codex v2 wire types and steer/interrupt adaptation
  worktree.rs                  git worktree per worker, branch a2a/<lane>/<id>
  mesh/                        presence, feed, notes, keys, registry, wire, work handoff
  sync/                        skills mirror, MCP translation, ownership manifest
  commands/                    one module per CLI verb (23 files)
skills/
  worker-protocol.md           SOURCE of the Codex worker protocol
  orchestrate-protocol.md      SOURCE of the Claude a2a-orchestrate skill
  generated/                   checked-in renderings (drift-gated)
templates/                     brief scaffolds + result/finding JSON schemas
schema/v2-snapshot/            checked-in codex app-server JSON Schema subset
docs/machines/worker.md        the FSM table (merge gate)
docs/superpowers/specs/        4 design specs (core + three mesh layers)
docs/superpowers/plans/        8 implementation plans (phases 1-5, mesh 1-3)
docs/mesh-join.md mesh-work.md operator runbooks
docs/development/              two corpus-appendix fragments (see Relationships)
migrations/                    mesh_l2_0001.sql, mesh_l3_0001.sql (fleet registry)
tests/                         77 integration test files + fixtures
```

Entry point: `src/main.rs` → `src/cli.rs` (clap) → one module per verb under
`src/commands/`. The daemon entry point is the same binary: `a2a broker run`.

## How It Was Built

**Toolchain.** Stable Rust with cargo; no `rust-toolchain.toml`, no vendored
deps, no lockfile pinning beyond `Cargo.lock`. Codex CLI 0.140.0 is the
protocol reference (`schema/v2-snapshot/REGEN.md`).

**Build / run / test — as they really are.**

- `cargo build` / `cargo install --path . --root ~/.local` → `~/.local/bin/a2a` (installed binary reports `a2a 0.3.0`, matching `Cargo.toml`)
- `cargo test` is hermetic: temp `$A2A_HOME` per test, `tests/fixtures/stub-codex` for the exec transport and `tests/fixtures/fake-app-server` for the live one, so it is green from a clean checkout with **no codex on PATH and no network**
- Four gated tiers, all `#[ignore]`d and all requiring an env switch: `A2A_E2E=1` for `e2e_exec` / `e2e_appserver` (real codex, real tokens), `A2A_E2E=1 A2A_EVAL=1` for the statistical prompt-contract tier (k=3 runs against `tests/fixtures/adversarial-briefs/`), `A2A_SYNC_LIVE=1` for the read-only sync proof, and `A2A_MESH_LIVE=1` / `A2A_MESH_L2_LIVE=1` / `A2A_MESH_L3_LIVE=1` for the mesh tiers
- The sync suite carries a runtime guard that refuses the real `$HOME` with `SYNC_REFUSES_REAL_HOME` (exit 9) unless `A2A_SYNC_ALLOW_REAL_HOME=1`

**Dev loop.** `bd` (beads) for issue tracking, wired into the repo's own
`.claude/settings.json` as a `SessionStart` and `PreCompact` hook running
`bd prime`. Design spec → phase plan → task-by-task execution, with commit
subjects carrying the bead id (`feat(mesh): work store + carriage + verified
ingest (L3 T4, a2a-54n.4)`).

**CI/CD and deploy path.** None. There is no `.github/` directory and no release
workflow; distribution is `a2a update`, which resolves the newest release tag (or
`origin/main` with `--channel main`), builds from a checkout under
`$A2A_HOME/update`, installs, verifies the resulting `--version`, and restarts an
installed gateway service (`src/commands/update.rs`).

**Configuration.** Layered `defaults < $A2A_HOME/config.toml < environment`, with
`a2a config show --origin` annotating each value with its layer. Key **names**
only: `allowed_roots`, `codex_version_range`; `[lanes.<lane>]` `model`, `effort`,
`sandbox`, `task_timeout_secs`, `max_turns`, `auto_review`, `allow_elevation`;
`[profiles.<name>]` (the `quick` / `standard` / `max` routing profiles);
`[admission]` `global_cap`, `queue_cap`, `breaker_failures`,
`breaker_window_secs`, `max_attempts`; `[broker]` `idle_timeout_secs`,
`events_max_bytes`; `[mesh]` `feed_max_bytes`, `stale_after_secs`,
`gc_after_days`; `[mesh.gateway]` `expose` and retry/backoff keys;
`[mesh.work]` `max_directed_bytes`, `max_pool_bytes`, `result_note_max_bytes`;
`[sync]` `allow_skills`, `allow_http_header_servers`, `claude_config_dir`. Every
key has an `A2A_`-prefixed environment override; `A2A_HOME`, `A2A_OWNER` and
`A2A_BROKER_DISABLE` are the three that change behaviour rather than a value.
MCP bearer credentials are never copied — the translated table names an env var
(`bearer_token_env_var`) whose value the operator exports (`src/config.rs`,
`src/sync/mcp.rs`, `README.md`).

**Provenance.** 144 commits, all authored by `web-mech`, 2026-08-14 → 2026-09-02
(129 in August, 15 in September) — a fully agent-built repository, executed
task-by-task from checked-in plans. The design spec is explicit that it was
"hardened after a 9-agent corpus-research + adversarial gap review" and names
nine mech-crate corpus documents whose techniques it applies. `.beads/` carries
61 issues: 57 closed, 3 open, 1 in progress.

## Relationships

- **Depends on (third-party):** the OpenAI Codex CLI — a2a is a supervisor for `codex app-server` and `codex exec`, pinned to `>=0.140, <0.150`; beads (`bd`) for its own issue tracking; Postgres/Neon for the layer-2 fleet registry.
- **Depends on (ours):** nothing at build time. a2a is self-contained Rust and does not link, shell out to, or import any of our other repositories.
- **Consumes the mech-crate corpus.** The design spec names nine `docs/development/` documents it applied — `appendix-actor-model.md`, `rust-async-cancellation-graceful-shutdown.md`, `appendix-fsm.md`, `appendix-streams.md`, `appendix-frp-rust.md`, `appendix-rust.md`, `appendix-api-design.md`, `appendix-rag.md`, `appendix-shell-scripting.md` — and `docs/machines/worker.md` cites `appendix-fsm.md` C1/C2/C6/C8 as the reason its FSM table is a checked-in merge gate. See `docs/development/repos/mech-crate.md`.
- **Owes the corpus two fragments.** `docs/development/appendix-api-design.md` (10 lines, "Mesh Injection Digest") and `docs/development/appendix-streams.md` (49 lines, "Mesh Anti-Entropy") sit in the a2a repo under corpus filenames and are **not** present in mech-crate's canonical documents of the same name — verified by grepping both headings in mech-crate. They read as contributions staged for merge and never merged.
- **Pairs with devloop.** `skills/orchestrate-protocol.md`'s `devloop-dispatch` section is a2a's half of the contract: devloop tasks get `--no-worktree` and concurrency 1 (devloop mutates one working tree, the plan file and `.beads/` between tasks), only the `api` and `cli` toolkits are dispatchable, and the impl lane's result schema *is* devloop's `sample-task-result.json` shape so devloop step 4.3.4 parses a worker result with no adapter. See `docs/development/repos/devloop.md`.
- **The `a2a-dispatch.md` question, answered.** The devloop profile flagged that the live `~/.claude/skills/devloop/references/a2a-dispatch.md` exists in no devloop repo copy. **a2a does not generate or own it.** There are zero occurrences of the string `a2a-dispatch` anywhere in `~/dev/a2a`, including its whole git history; `__gen-skills` emits exactly three artifacts and no `references/` directory of any kind. The file is hand-written and committed to a *fourth* repository — `~/.claude/skills` is itself a git checkout of `nyvorin/claude-skills`, where `a2a-dispatch.md` was created on 2026-08-20 in commit `e3a76c8` ("devloop: optional codex dispatch for api/cli tasks via a2a") and last touched 2026-09-02 in `683626a` (model/effort routing). It restates a2a's rules in devloop's vocabulary and adds devloop-only material (devloop's own step numbers, a brief template, `NO_DIFF_BASE`, `ATTEMPTS_EXHAUSTED`), then points back at the `a2a-orchestrate` skill for the full protocol. See `docs/development/repos/claude-skills.md`.
- **Canonical-copy note.** `~/.claude/skills/a2a-orchestrate` is a **hand-made symlink** into `~/dev/a2a/skills/generated/claude/a2a-orchestrate` — it shows as untracked in the claude-skills repo, and a2a never created it. So the Claude-side orchestrator skill is canonical *in this repo* and merely referenced from the skills tree, which is the opposite of devloop's situation.
- **Feeds the Codex home.** `a2a sync skills` mirrors `~/.claude/skills` into `$CODEX_HOME/skills/<name>/` and `a2a sync mcp` translates `~/.claude.json` `mcpServers` into `$CODEX_HOME/config.toml`, both governed by one ownership ledger. See `docs/development/repos/codex-skills.md` and `docs/development/repos/codex-config.md` for the destinations' own stories.

## Notable Techniques

- **Exit code as the API.** A versioned, test-pinned exit-code table (0 success, 1 internal, 2 usage, 3 no such worker, 4 failed, 5 blocked, 6 not terminal, 7 broker unavailable, 8 internal timeout, 9 precondition, 10 orphaned) turns a monitor loop into a `case` statement and keeps the orchestrator's context spend at a few hundred tokens per dispatch. This is the concrete answer to the "delegation is a contract" finding in `docs/development/multi-agent-systems-in-practice.md`.
- **WAL + rebuildable projection.** `events.jsonl` append + `sync_data` per line with torn-tail repair on open; `state.json` written temp → fsync → rename → fsync-dir; the fold is pure so the snapshot is always reconstructible. The corpus already has the general pattern in `appendix-streams.md`; a2a is the worked example.
- **A pure FSM with no catch-all arm, gated by a checked-in table.** `docs/machines/worker.md` and `tests/fsm_table.rs` make totality a build failure rather than a silent default — `appendix-fsm.md` C1/C6/C8 in practice.
- **Held-out, execution-based verification.** `a2a verify` re-runs the *brief's* commands, never the worker's, in a fresh process, and records per command the command string, exit code, `stdout_sha256`, timeout flag and `ran_by: "a2a-verify"`. It also reports `scope.outside_scope` — changed paths no `files_in_scope` entry claimed — plus `files_in_scope_declared`, so an empty `outside_scope` cannot be misread as "stayed in its lane" when no lane was drawn. Exactly the held-out verification the corpus says beats self-report.
- **The execution-bearing-diff gate.** Before re-running anything, verify checks whether the worker's diff touched a path that *executes on somebody else's machine* — `.github/workflows/**`, `build.rs`, `package.json`, `Makefile`, `justfile`, `.cargo/config.toml`, `conftest.py`, `pyproject.toml`, dependency manifests and lockfiles, `.gitmodules`, `.gitattributes` — and refuses with `EXEC_BEARING_DIFF` rather than running the brief's commands against a tree that may have been armed. The sentence names at most ten paths; the full list travels in `data.paths` (`src/commands/verify.rs`).
- **One source, many renderings, drift-gated.** A security protocol that must exist in three places is written once with HTML section markers and rendered; `--check` byte-compares, and a missing artifact counts as drift. The renderer deliberately refuses to install anything.
- **Ownership ledgers over reconciliation.** `sync-manifest.json` records every destination a2a created and a2a only ever updates or removes one the ledger claims; losing the ledger costs the ability to *update*, never content. Every value read from a source config is printed as `[REDACTED:<len>]` in every mode regardless of what the classifier decided, so a misclassification cannot leak.
- **Deny-first capability classification.** Skills are mirrored to Codex unless a seed list or a word-boundary match on `credential|send|post|deploy|password|token|aws` in the directory name or YAML front matter says otherwise — front matter only, because scanning bodies flagged nearly the whole corpus on a real machine.
- **Backlog candidates** (listed here, not filed): a technique doc on *exit codes and file-based control planes as an agent-to-agent API*; one on *execution-bearing diffs* as a general "do not run the untrusted tree's build" gate; and one on *generated-from-one-source agent protocols* (marker slicing, drift gates, and why the renderer must not install).

## State, Gaps and Drift

**Maturity.** v0.3.0, 144 commits over 20 days, ~98k lines of Rust of which 42%
is tests, 77 integration test files, zero `TODO`/`FIXME`/`HACK` markers in
`src/`. All eight implementation plans are complete except the newest
(`2026-09-01-a2a-mesh-work-handoff.md`, 26 unchecked boxes and none checked) —
yet its code shipped: `src/commands/work_cmd.rs`, `src/mesh/work*.rs`,
`migrations/mesh_l3_0001.sql`, `docs/mesh-work.md` and twelve `L3 T…` commits all
exist. The plan file was simply never ticked; beads carried the tracking instead.

**README-vs-code drift (the repo's largest gap).** `README.md` was last modified
2026-08-26 and has not kept up with the 15 commits since:

- Line 22 still says **"This is Phase 2 (broker + app-server transport). Worktrees, guardian-denial parking and `a2a approve` arrive later."** Phases 3, 4 and 5 all shipped (sync, worktrees + `a2a verify` + generated skills, fleet survival), as did three mesh layers. Only the guardian-denial half of that sentence is still true.
- `a2a update` (the self-updater), `a2a config add-root`, `a2a mesh work …` (layer 3) and `spawn --profile quick|standard|max` are entirely absent from the README.
- The `doctor` sample output shows **13 checks**; the code defines **15** — `allowed_roots` and `gateway_enrollment` are missing from the README's list (`src/commands/doctor.rs`).

**Other observations.**

- `CLAUDE.md` is a bd-generated stub: its "Build & Test", "Architecture Overview" and "Conventions & Patterns" sections still read "_Add your … here_", so an agent that reads only `CLAUDE.md` learns nothing about the project. The real orientation is in `README.md` and the specs. `AGENTS.md` is similarly generic (bd + non-interactive shell flags) and is not imported by `CLAUDE.md` — which is the very thing `doctor`'s `context_canon` check exists to warn about.
- No `LICENSE` file and no license metadata on GitHub. For a private repo that is a choice, but it is undeclared rather than deliberate-and-recorded.
- No CI. Every gate that exists — the FSM totality scan, the skills drift check, the protocol canary, the exit-code contract test — is a `cargo test` somebody has to remember to run. `__gen-skills --check` is described in its own module doc as "a CI gate" for a repository that has no CI.
- The checkout is **live during profiling**, which is itself a fact about this repo: at the start of this pass `git status --porcelain` showed only a staged `.beads/issues.jsonl`; twenty minutes later a concurrent session (a running broker, an `impl-…` worker being polled, and a `cargo test` compile) had also modified `src/protocol.rs`, `src/responder.rs`, `tests/rpc_fake.rs` and the whole `schema/v2-snapshot/` tree. Nothing in this profiling pass wrote to the repo; a2a is simply a repository that is usually being worked on by one of its own workers.

### Synthesis (inferred)

a2a reads as an argument that **the interesting part of multi-agent orchestration
is the boundary, not the agents**. Almost every design decision is about making
one agent's report unnecessary: the exit code replaces "how did it go", the
outbox question file replaces "it said it was blocked", `verification.json`
replaces "it says the tests pass", and `scope.outside_scope` replaces "it says it
only touched what it was asked to". The worker is treated as an untrusted
executor throughout — its diff can arm the build, its prose can carry injected
instructions, its self-report is not evidence — and the code says so explicitly
rather than hoping.

The second theme is that **state belongs on disk, not in a process**. The broker
is called a "daemon-lite" for a reason: it can be `kill -9`d at any moment, its
workers become `orphaned`, and the next daemon adopts them by re-opening threads
on a fresh epoch. That is only possible because the WAL, not the daemon, is the
truth. It is also why the exec transport can stay a first-class fallback instead
of a legacy path, and why `a2a resume` can continue an app-server worker after
its broker is gone — the thread id *is* the codex session id.

Where the repo is thin is exactly where a fully agent-built project tends to be
thin: the artifacts a human maintains by habit. Tests, specs, plans and inline
module documentation are unusually strong (module doc comments routinely explain
*why*, including recorded deviations and the bug ids that forced them). The
README, the `CLAUDE.md` stub, the missing license and the absent CI are the
outliers, and they drifted in the same 15 commits — which suggests the working
loop is "spec → plan → beads → code → tests", with docs updated only when a phase
plan says to. The cheapest correction is to make `README.md` a phase-agnostic
document and to move the "what phase are we in" claim into something generated,
since the repo already knows how to generate documents and gate them for drift.

## Quick Reference
| Task | Command / path |
|---|---|
| Build | `cargo build` (repo root — one package, no workspace) |
| Install | `cargo install --path . --root ~/.local` → `~/.local/bin/a2a` |
| Update | `a2a update` (`--check` to report only, `--channel main` for tip) |
| Tests | `cargo test` — hermetic, no codex, no network |
| Gated proofs | `A2A_E2E=1 cargo test --test e2e_exec -- --ignored` (also `e2e_appserver`) |
| Read-only sync proof | `A2A_SYNC_LIVE=1 cargo test --test sync_live -- --ignored` |
| Preflight | `a2a doctor` (exit 9 if any check fails) |
| Dispatch | `ID=$(a2a spawn --lane impl --cwd ~/dev/p --brief b.md)` |
| Monitor | `a2a wait "$ID" --timeout 30m; case $? in 0) … 5) … esac` |
| Accept | `a2a verify "$ID"` → `~/.a2a/workers/<id>/verification.json` |
| Brief scaffold | `a2a spawn --template impl` |
| State tree | `~/.a2a/` (0700) — `workers/`, `worktrees/`, `mesh/`, `work/` |
| FSM table | `docs/machines/worker.md` (mirrors `src/fsm.rs`, gated by `tests/fsm_table.rs`) |
| Skill sources | `skills/worker-protocol.md`, `skills/orchestrate-protocol.md` |
| Regenerate skills | `a2a __gen-skills --out skills/generated` (`--check` for drift) |

## Sources

- `README.md` — quick start, transports table, broker, admission control, fleet survival, exit-code table, configuration, mesh, sync, secret policy, safety properties, development tiers; also the source of the Phase-2 drift finding.
- `CLAUDE.md`, `AGENTS.md` — bd integration and the unfilled project sections.
- `src/cli.rs` — the complete verb and flag surface, including hidden verbs.
- `src/commands/gen_skills.rs` — the three artifacts, marker-slicing renderer, pinned section lists, and the explicit "never installs" contract.
- `src/commands/verify.rs` — `EXEC_BEARING_RULES`, the `Scope` snapshot shape, the refusal's `data.paths` behaviour.
- `src/commands/doctor.rs` — the fifteen checks in code order.
- `src/config.rs` — lanes, sandbox/effort defaults, turn budgets, profiles, every config key name.
- `src/brief.rs` — brief front-matter keys and the per-lane required set.
- `src/home.rs` — `$A2A_HOME` layout, 0700/0600 creation helpers, layout version.
- `skills/orchestrate-protocol.md` — dispatch choice, brief writing, monitor loop, acceptance gate, worktree hygiene, ownership + beads executor metadata, devloop dispatch, mesh milestones, work handoff, model/effort routing.
- `skills/worker-protocol.md` — data-not-instructions, cwd discipline, blocked protocol, progress notices, result and evidence.
- `templates/impl-brief.md`, `templates/impl-result.schema.json` — the scaffold and the devloop-shaped result contract (including why every object is closed).
- `docs/machines/worker.md` — states, failure classes, transition table, the no-catch-all rule.
- `docs/superpowers/specs/2026-08-14-a2a-claude-codex-design.md` — purpose, decisions table, the nine corpus documents it applied.
- `docs/superpowers/plans/*.md` — the eight phase/layer plans and their completion state; `docs/mesh-work.md` — the layer-3 rollout runbook.
- `schema/v2-snapshot/REGEN.md` — why a checked-in protocol snapshot exists and how the canary test uses it.
- `docs/development/multi-agent-systems-in-practice.md` (mech-crate) — the corpus analysis of delegation-as-contract, caps and held-out verification that a2a implements.
- Repo metadata via `git -C ~/dev/a2a log/status` and `gh api repos/Dev916/a2a`; `a2a-dispatch.md` provenance via `git -C ~/.claude/skills log --follow` on that file.
