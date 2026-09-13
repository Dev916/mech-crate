---
title: "devloop: Claude Code skill that executes a plan task-by-task with a verifying subagent (Repo Profile)"
category: repos
languages: [markdown]
complexity: intermediate
use_cases:
  - "understanding what devloop does and where its files live"
  - "finding devloop's skill surface — orchestrator phases, verification toolkits, subagent JSON contract — before extending it"
  - "answering 'which repo executes an implementation plan with live verification'"
  - "resuming work on devloop in a fresh session"
summary: "devloop is a two-skill Claude Code package containing no code at all — eleven Markdown files plus one JSON example — that executes an existing design spec and implementation plan one task at a time. A lean orchestrator in the main session detects the project structure (docker/mx, xcode, standalone), stands up the environment, discovers the target URL / simulator / binary, then dispatches one isolated subagent per plan task; that subagent implements the task and then verifies it with the task's assigned toolkit — Playwright for web UI, XcodeBuildMCP for the iOS simulator, live HTTP for APIs, live run for CLIs — iterating until acceptance criteria pass, with stuck detection at three identical failures and a hard cap of ten iterations, returning a fixed-shape JSON result. Progress is resumable via plan-file checkboxes or optional beads (bd) issues. The public repo nyvorin/devloop (Apache-2.0) has been frozen at three commits since 2026-04-22; the installed copy under ~/.claude/skills/devloop has since gained a techniques-corpus consult and an optional a2a/Codex dispatch path, and is the copy that actually runs."
provenance: researched
researched: 2026-09-02
publish: false
repo: https://github.com/nyvorin/devloop
local_path: ~/dev/devloop
status: dormant
visibility: public
owner: Personal (nyvorin)
sources:
  - README.md (target repo)
  - SKILL.md (target repo)
  - subagent-prompt.md (target repo)
  - writing-devloop-plans/SKILL.md (target repo)
  - references/observe-modes.md (target repo)
  - references/xcode-observe-modes.md (target repo)
  - references/api-observe-modes.md (target repo)
  - references/cli-observe-modes.md (target repo)
  - references/url-discovery.md (target repo)
  - references/mx-cli-reference.md (target repo)
  - references/xcodebuildmcp-reference.md (target repo)
  - examples/sample-task-result.json (target repo)
  - manifest.json (target repo)
  - docs/development/multi-agent-systems-in-practice.md (mech-crate)
  - docs/development/llm-token-cache-efficiency.md (mech-crate)
  - docs/development/mx-app-playbook.md (mech-crate)
---

# devloop

> **"Build with eyes on."** devloop is a Claude Code *skill* — pure Markdown, no
> executable code — that takes a design spec plus an implementation plan and
> executes the plan task by task, dispatching one isolated subagent per task that
> writes the code and then *observes the running result* before declaring the task
> done. Observation is per-task, not per-project: a browser (Playwright), an iOS
> simulator (XcodeBuildMCP), a live HTTP request, or a live CLI run. The
> orchestrator stays in the main session and stays lean — all the screenshots,
> DOM dumps and failed attempts live in the subagent's throwaway context, so the
> user sees clean pass/fail per task. The public repo is frozen at three commits
> from April 2026; the copy installed under `~/.claude/skills/devloop` has kept
> moving and is what actually runs here.

## Identity
| Field | Value |
|---|---|
| Repository | `nyvorin/devloop` (public) — default branch `main` |
| Local path | `~/dev/devloop` (directory name matches the repo name) |
| Owner / org | Personal (nyvorin) — no hq project registered for it |
| Status | dormant — last commit 2026-04-22, 3 commits, profiled at `85d33a8` |
| Languages (by file count) | Markdown 11 · JSON 2 · other 2 (`LICENSE`, `.gitignore`) — 15 files, ~1,140 lines total, zero executable code |
| Build system | none — nothing to compile; `manifest.json` is a skill manifest, not a build file |
| Runtime deps | Claude Code (the skill host). Per-toolkit and optional: Playwright MCP, XcodeBuildMCP, `mx` CLI + Docker, `bd` (beads), `curl`/`bash`, `yq` or `python3`+PyYAML for compose parsing |
| License | Apache-2.0 (`LICENSE`, also declared in `manifest.json` and GitHub metadata) |
| CI / release | none — no `.github/` directory, no tags, no releases (checked via `gh api repos/nyvorin/devloop`) |

## What It Does

The problem: an agent that writes code from a plan can only see its own tests. It
cannot tell whether the page renders, whether the endpoint really answers on the
wire, or whether the app crashes on launch — so "tests pass" and "it works" drift
apart, task after task, until the final integration is a surprise.

devloop's answer is to make every task end in an observation. It assumes the
design and planning work already happened (Claude's brainstorming and
writing-plans skills, augmented by the bundled `writing-devloop-plans`), reads
the most recent spec from `docs/superpowers/specs/` and the most recent plan from
`docs/superpowers/plans/`, and then runs a five-phase loop: detect the project
structure, stand the environment up, discover what to point at, work the tasks
one at a time through subagents, and finish with a held-out acceptance pass that
re-verifies every completed task from a fresh context (`SKILL.md`).

Its users are agents, not humans: the orchestrator is a Claude Code session, each
task executor is a subagent, and the human's role is to answer the
skip/retry/abort question when a task gets stuck and to read the summary at the
end. "Done" looks like every plan task checked off, a green final acceptance
pass, and the environment left running with its URL (or simulator, or built
binary) handed back to the user (`SKILL.md`, `README.md`).

## Capabilities

### Skills
- `devloop` — the orchestrator: preconditions, environment standup, target discovery, per-task dispatch, final acceptance (`SKILL.md`)
- `writing-devloop-plans` — companion planner that augments a `superpowers:writing-plans` plan with a per-task **Acceptance Criteria** block and a `**Compatible with:** devloop skill v0.1+` header flag (`writing-devloop-plans/SKILL.md`)
- Both skills are declared for plugin installation in the package manifest (`manifest.json`)

### Orchestrator phases
- Phase 1 — project-structure detection (`xcode` → `docker` → `standalone`, in that order), toolkit availability checks, spec/plan discovery, and beads detection (`SKILL.md`)
- Phase 2 — environment standup: `mx router up` + `mx dev` with restart-loop detection for docker, simulator build for xcode, local build plus optional background server for standalone (`SKILL.md`)
- Phase 3 — target discovery: Traefik URL for docker, `"simulator"` for xcode, `localhost:<port>` or a binary path for standalone, each with a reachability check (`SKILL.md`)
- Phase 4 — the per-task loop: parse the plan into task blocks, classify each task's toolkit, dispatch a subagent, parse its JSON, flip checkboxes on pass, halt and ask on stuck/fail (`SKILL.md`)
- Phase 5 — final acceptance: a single verification-only subagent re-runs every task's checks; one regression-repair subagent is allowed before escalating (`SKILL.md`)
- A DOT flow diagram of the whole lifecycle is embedded in the orchestrator (`SKILL.md`)

### Verification toolkits (classified per task, not per project)
- `playwright` — navigate, DOM accessibility snapshot, console capture, network-failure capture; `interactive` mode additionally drives forms, clicks and waits (`references/observe-modes.md`)
- `xcodebuildmcp` — build-and-run into the simulator, screenshot analysis, tap/swipe automation, crash and build-error handling (`references/xcode-observe-modes.md`)
- `api` — write a persisting integration test in the project's own framework, then make the live HTTP request and compare status/headers/body, saving a request+response evidence JSON (`references/api-observe-modes.md`)
- `cli` — write a persisting integration test, then run the binary for real and capture stdout, stderr, exit code and file effects as a proof snapshot (`references/cli-observe-modes.md`)
- Selection order: an explicit `**Verify via:** <toolkit>` tag in the task block wins; otherwise keyword voting across the four keyword lists; otherwise a per-structure default (docker → `api`, xcode → `xcodebuildmcp`, standalone → `cli`) (`SKILL.md`)

### Subagent contract
- A single fill-in-the-blanks prompt template covers all four toolkits, with double-brace placeholders the orchestrator substitutes (spec excerpt, task block, project root, structure, toolkit, target URL, binary path, observe mode, scratch dir) (`subagent-prompt.md`)
- The subagent must first *derive* acceptance criteria from spec + task if the plan did not state them, and must return `fail` rather than invent criteria it cannot ground in the spec (`subagent-prompt.md`)
- Stuck detection: the same observation or failed assertion three iterations running returns `status: "stuck"`; a hard cap of ten iterations returns `status: "fail"` (`subagent-prompt.md`)
- The last message must be one fenced JSON object — `status`, `iterations`, `summary`, `evidence.{screenshots, console_errors, network_failures, logs_excerpts}`, `files_changed`, `next_step_hint` — and unparseable output is treated as a failure (`subagent-prompt.md`, `examples/sample-task-result.json`)

### Environment and target discovery
- Traefik host-label parsing from the compose file with a real YAML parser (explicitly "never regex YAML"), TLS-label-driven scheme selection, and an abort-with-candidates rule when more than one URL matches (`references/url-discovery.md`)
- The exact `mx` command set devloop relies on, including the note that `mx` has no `--tail` flag so log dumps pipe through `tail` (`references/mx-cli-reference.md`)
- XcodeBuildMCP tool categories, the instruction to re-discover live tool names with ToolSearch, and the optional `.xcodebuildmcp/config.yaml` keys (`references/xcodebuildmcp-reference.md`)

### Task tracking
- Default: plan-file checkboxes are the source of truth; re-running resumes at the first task with an unchecked step (`SKILL.md`)
- Optional beads: auto-detected from `bd` on PATH plus a `.beads/` directory, then one issue per task with a linear dependency chain, `--claim` before dispatch, notes plus close on pass, notes only on stuck, and `bd ready`/`bd list` driving resume — with bd authoritative over checkboxes when present (`SKILL.md`, `README.md`)

### Not (yet) implemented
- `--from-scratch` (running without a pre-existing plan) is explicitly aborted as a v0.2 feature (`SKILL.md`)
- Multiple discovered URLs abort rather than prompt; disambiguation is deferred to v0.3 (`references/url-discovery.md`)
- Spec excerpting: the full spec is passed to every subagent, with per-section splitting named as a future optimization (`subagent-prompt.md`)
- No tests, linter, schema file or CI of any kind ship with the repo — the JSON contract is enforced only by prose plus one example (`examples/sample-task-result.json`)

## Architecture

**Stack.** Markdown and JSON only. The "runtime" is Claude Code's skill loader:
`SKILL.md` frontmatter (`name`, `description`) makes the skill discoverable, and
the body is executed by an agent reading it as a checklist. Version 0.1.0 per
`manifest.json`.

**Component map.** One orchestrator document, one subagent prompt template, seven
on-demand reference documents, one example result, one companion planning skill.
The split is deliberate: the orchestrator loads always, the references load only
when the matching branch is taken.

**Data flow.**

```
plan file + spec  ──▶  orchestrator (main session)
                          │  detect structure ─ stand up env ─ discover target
                          │
                          ├──▶ per task: classify toolkit ─▶ substitute prompt
                          │        └──▶ subagent (isolated context)
                          │               code work ─▶ observe ─▶ compare ─▶ fix
                          │               (≤10 iterations, 3 repeats = stuck)
                          │        ◀── one JSON object (status/evidence/files)
                          │
                          ├──  pass: flip checkboxes (+ bd note/close)
                          └──  stuck|fail: halt, surface evidence, ask user
                                     │
                          final acceptance subagent (re-verify only) ─▶ pass/repair
```

**Storage.** Three places, all outside the repo: the plan file's `- [ ]`
checkboxes (progress), `/tmp/devloop-run-<ISO-timestamp>/` (screenshots, API
evidence JSON, CLI proof snapshots — per run, disposable), and optionally
`.beads/` in the target project (issue history and evidence notes).

**External integrations.** Playwright MCP browser tools, XcodeBuildMCP simulator
and UI-automation tools, the `mx` CLI (router, ps, dev, logs) and Traefik labels
for docker projects, `bd` for beads, plus plain `curl`/`bash` for the API and CLI
toolkits — which is why those two paths need no MCP at all.

**Process / concurrency model.** Strictly sequential and single-level. One
orchestrator, one subagent alive at a time, tasks in plan order, no peer
messaging and no nesting. Concurrency is deliberately absent because every task
mutates one shared working tree and one plan file.

**Security model.** The repo declares no configuration and no secrets — there is
nothing to hold. The only security-adjacent decisions are that reachability
checks pass `curl -k` to accept the self-signed certificates typical of local
Traefik setups (`references/url-discovery.md`), and that 401/403 count as
"reachable" so auth-gated apps do not abort a run (`SKILL.md`). Toolkits execute
project code and project test suites by design, so devloop inherits whatever
trust boundary the target project has.

## Repository Layout

```
SKILL.md                              orchestrator — the devloop skill itself (entry point)
subagent-prompt.md                    per-task subagent prompt template (all four toolkits)
README.md                             human-facing overview, install, toolkit and beads docs
manifest.json                         skill package manifest (name, version, both skills)
LICENSE                               Apache-2.0
examples/
  sample-task-result.json             the JSON shape every subagent must return
references/
  observe-modes.md                    Playwright static vs interactive observation
  xcode-observe-modes.md              simulator static vs interactive observation
  api-observe-modes.md                API keyword list, live-hit procedure, evidence shape
  cli-observe-modes.md                CLI keyword list, live-run procedure, proof shape
  url-discovery.md                    Traefik label parsing and reachability rules
  mx-cli-reference.md                 the mx commands devloop uses
  xcodebuildmcp-reference.md          XcodeBuildMCP tool categories and detection
writing-devloop-plans/
  SKILL.md                            companion planning skill (entry point #2)
```

Entry points: `SKILL.md` (invoked by `/devloop` or "run the dev loop") and
`writing-devloop-plans/SKILL.md` (invoked when writing a devloop-compatible
plan). Everything else is read on demand by one of those two.

## How It Was Built

**Toolchain.** None. There is no compiler, no package manager, no lockfile and no
test runner in the repository; authoring is editing Markdown.

**Build / run / test / lint — as they really are.** No build. No test suite. No
linter. "Running" it means installing the skill and invoking `/devloop` inside a
project that already has a spec and a plan. The README's install path is a clone
straight into the skills directory plus a symlink for the companion skill, or a
`claude plugin install nyvorin/devloop` (README claims, not verified — the repo
carries a root `manifest.json` and has no `.claude-plugin/` directory, so whether
the plugin form works is undetermined).

**Dev loop.** Edit the Markdown, reload the skill, run it against a real project
and watch what the orchestrator does. Because the repo's product *is* a prompt,
the only meaningful test is an end-to-end run.

**CI/CD and deploy path.** None. No workflows, no tags, no releases; distribution
is `git clone` (or the unverified plugin install).

**Configuration and env vars.** The repo declares none. It reads configuration
only out of the *target* project: `mx.toml` or a compose file with a `# mx-managed`
marker (docker detection), `Cargo.toml` / `go.mod` / `package.json` /
`pyproject.toml` (standalone detection), and an optional `.xcodebuildmcp/config.yaml`
whose relevant keys are the default scheme, project path, simulator device name and
enabled workflows.

**Provenance.** Three commits in two days by web-mech: an initial v0.1.0 release
(2026-04-21) and two commits adding optional beads tracking plus its README
section (2026-04-22). No design spec ships with the repo, and no devloop design
spec exists in mech-crate's `docs/superpowers/specs/` — the design lives in the
skill document itself. The repo has been untouched since; local `main` is level
with `origin/main`.

## Relationships

**Four copies exist, and they are not identical.** This profile documents
`~/dev/devloop` (the named repo, at `85d33a8`). Diffed against it:

| Copy | Divergence from the repo |
|---|---|
| `~/dev/devloop` (this repo) | baseline — the only copy with `README.md`, `LICENSE`, `manifest.json` and the `writing-devloop-plans/` subdirectory |
| `~/.claude/skills/devloop` | **ahead**: adds `references/a2a-dispatch.md` (162 lines, exists in no other copy); adds orchestrator steps 1.8 (Codex dispatch detection) and 1.9 (agent-mesh milestone posts), rewrites 4.3.2c to record `executor_provider` / `executor_session` / `a2a_worker` metadata on the bd issue, adds a Codex-worker exception to 4.3.3, and adds mesh posts to 4.3.6 and 5.3. Its `subagent-prompt.md` inserts a mandatory `mcp__mx__rag_context` techniques consult as step 2 and renumbers the rest |
| `mech-crate/skills/devloop/` | a **one-file overlay** — only `subagent-prompt.md`, byte-identical to the live copy's; committed as `b293e9c` "devloop subagents consult techniques corpus per task" (2026-07-16) |
| `~/.codex/skills/devloop` | a Codex port: reference paths rewritten to `~/.codex/...`, the Playwright check made tool-name-agnostic, `superpowers:executing-plans` replaced with plain instructions, dispatch generalised to "the available Codex multi-agent tool", plus `agents/openai.yaml` (display name, short description, default prompt). No `writing-devloop-plans/` |

`writing-devloop-plans` diverges the same way: `~/.claude/skills/writing-devloop-plans/SKILL.md`
and `mech-crate/skills/writing-devloop-plans/SKILL.md` are byte-identical to each
other and equal to the repo's copy plus one inserted paragraph requiring an
`mcp__mx__rag_context` consult while drafting, and an `**Apply:** <doc path>`
line on tasks that use a technique. It is installed as a plain directory, not the
symlink into `devloop/writing-devloop-plans` that the README prescribes.

**Canonical copy:** `~/.claude/skills/devloop` is canonical in practice — it is
what Claude Code loads, and it is the only copy carrying both the corpus consult
and the a2a dispatch path. The public repo is the *published* artifact and is two
feature generations behind it. mech-crate's `skills/` tree is neither: it is a
version-controlled record of the corpus-integration edits (two files), not an
installable copy.

- **Depends on (ours):** `mx` for every docker-structure run (router, ps, dev, logs, Traefik URLs) — see `docs/development/repos/mech-crate.md`; the live copy additionally depends on `a2a` (`Dev916/a2a`, `~/dev/a2a`) for the optional Codex dispatch path — see `docs/development/repos/a2a.md`.
- **Depends on (third-party):** beads (`gastownhall/beads` — link verified) for optional tracking; Playwright MCP; XcodeBuildMCP.
- **Used by:** mech-crate's own development loop; the `a2a-orchestrate` skill documents a "Dispatching devloop tasks to Codex workers" protocol and notes that a2a's impl-lane result schema *is* devloop's `sample-task-result.json` shape, so devloop step 4.3.4 parses a worker result with no adapter.
- **Pairs with:** `writing-devloop-plans` (upstream of it) and the mech-crate techniques corpus (consulted by both, in the live copies only).
- **Broken outbound link:** the README points at `https://github.com/mech-crate/mx` for the mx CLI; that repo returns 404. mx actually lives in `Dev916/mech-crate`. Both facts recorded; the README's claim is not endorsed.

## Notable Techniques

- **Context isolation as the point.** Verbose evidence (screenshots, DOM dumps, build logs, failed attempts) never reaches the supervisor — only a ~1 KB JSON summary does. Already analysed in `docs/development/llm-token-cache-efficiency.md`, which names devloop's economics "context-isolation economics" and warns against routing small tasks to cheaper subagent models.
- **Hub-and-spoke, single-level, sequential.** No peer messaging, no nesting, one worker at a time. `docs/development/multi-agent-systems-in-practice.md` evaluates exactly this shape and concludes devloop "already matches the winning shape", while flagging the gap: task prompts should carry output format, tool guidance, boundaries and a call budget — delegation as contract.
- **Held-out verification.** Phase 5 dispatches a *fresh* subagent that re-runs the checks itself rather than trusting the implementer's claims — the held-out-suite pattern the same corpus doc endorses.
- **Traefik-label URL derivation.** Parse compose with a real YAML parser, derive scheme from a sibling `tls=true` label, never assume a port. Already captured in `docs/development/mx-app-playbook.md`, which cites devloop's `references/url-discovery.md` as its source.
- **Structured return over prose.** A fixed JSON schema at the subagent boundary, with "unparseable = fail" as the rule, is what lets the orchestrator stay mechanical.
- **Backlog candidates** (not filed here, per the profiling procedure): a technique doc on *observable acceptance criteria* — deriving user-visible, toolkit-checkable facts from a spec and refusing to invent them; and one on *iteration governors for agent loops* — repeat-detection versus hard caps, and what each failure mode costs.

## State, Gaps and Drift

**Maturity.** v0.1.0, three commits, frozen since 2026-04-22, no tags, no CI, no
tests, one GitHub star. Documentation quality is high and the design is coherent;
the repository is a specification, and the only executable check on it is running
it.

**README-vs-code drift.**
- `writing-devloop-plans/SKILL.md` still ends with "Devloop is web-app only" and tells non-web projects to use plain writing-plans — flatly contradicting `SKILL.md`, `README.md` and the skill descriptions, all of which advertise four toolkits including CLI and API. Its acceptance-criteria heading is likewise hardcoded to "Playwright-observable" even though the body later explains API and CLI phrasing. Highest-value single fix in the repo.
- The README's mx link 404s (see Relationships).
- The README's install instructions prescribe a symlink for the companion skill; the actual local install is a plain copied directory, so the two skills can (and did) drift apart.
- `claude plugin install nyvorin/devloop` is a README claim, not verified; the repo has no `.claude-plugin/` directory.

**Portability leaks.** Phase 1.2 hardcodes `mcp__plugin_playwright_playwright__browser_navigate` — a name that depends on the Playwright MCP being installed as a *plugin* under that exact namespace. The Codex copy already had to patch this line to a tool-discovery check, which is evidence the hardcoding bites. Similarly, `references/xcodebuildmcp-reference.md` documents pseudo-names (`simulator/build-and-run`, `ui-automation/tap`) that do not match the real tool names (`build_run_sim`, `tap`); the file hedges by telling the subagent to re-discover names with ToolSearch, so this is a documentation-cost issue rather than a breakage.

**Minor internal inconsistencies.** Restart-loop detection is specified twice with different windows — poll for up to 60s in `SKILL.md` step 2.3d, poll for 30s in `references/mx-cli-reference.md`. Deferred features are labelled against versions (v0.2 `--from-scratch`, v0.3 multi-URL) that no release plan backs.

**TODO/FIXME density.** Zero literal TODO/FIXME/HACK markers; deferrals are written as prose "v0.x" notes instead. No `.beads/` directory in the repo, so no open-issue count exists for it.

**Risks.** The published artifact is drifting away from the running one with no
sync mechanism: the live copy has gained a2a dispatch, mesh milestone posts, bd
executor metadata and the corpus consult, none of which exist upstream, and
nothing in either tree reconciles them. A second risk is silent toolkit
misclassification — keyword voting with a per-structure fallback can pick the
wrong verifier, and the only escape hatch is a manually added `**Verify via:**`
tag.

### Synthesis (inferred)

devloop is best read as a *contract* rather than a tool: it fixes the boundary
between an orchestrator and its executors (one task in, one JSON object out) and
then makes everything else pluggable behind it. That is why the four toolkits
could be added without restructuring anything, why a2a can substitute a Codex
worker for a Claude subagent at step 4.3.3 with no adapter, and why the whole
thing can be 1,100 lines of Markdown with no code. The interesting engineering
is in what it *refuses*: no parallelism, no nesting, no self-reported success, no
inventing acceptance criteria the spec cannot ground.

The freeze is not neglect so much as a fork that was never merged back. Every
post-April change was made where the skill actually runs, and only the two
corpus-consult files were ever captured in version control (in mech-crate, not in
devloop). If devloop is meant to stay a public artifact, the cheapest correct
move is to make `~/.claude/skills/devloop` a checkout of the repo again and land
the a2a, mesh and corpus deltas upstream; if it is not, the repo should say so.

The `writing-devloop-plans` "web-app only" line is the clearest evidence of how
the fork happened: the planning skill was written first, against a Playwright-only
v0.1, and the three later toolkits were added to the orchestrator without a pass
back over its companion.

## Quick Reference
| Task | Command / path |
|---|---|
| Build | none — Markdown only, nothing to build |
| Run | `/devloop` in a project that has a spec in `docs/superpowers/specs/` and a plan in `docs/superpowers/plans/` |
| Tests | none ship with the repo |
| Install (clone) | `git clone https://github.com/nyvorin/devloop ~/.claude/skills/devloop`, then symlink `writing-devloop-plans` alongside it (`README.md`) |
| Write a compatible plan | invoke the `writing-devloop-plans` skill (`writing-devloop-plans/SKILL.md`) |
| Force a toolkit | add `**Verify via:** playwright\|xcodebuildmcp\|api\|cli` to the task block (`SKILL.md`) |
| Result contract | `examples/sample-task-result.json` |
| Evidence from a run | `/tmp/devloop-run-<ISO-timestamp>/` |
| Resume after abort | re-run `/devloop`; unchecked plan steps (or open bd issues) drive it |
| The copy that actually runs | `~/.claude/skills/devloop` (ahead of this repo) |

## Sources

- `README.md` — install paths, prerequisites table, toolkit keyword lists, beads section, the mx and beads links.
- `SKILL.md` — the five phases, structure detection order, toolkit classification, error-handling table, resumability and post-success behaviour.
- `subagent-prompt.md` — the prompt template, criteria-derivation rules, stuck detection, iteration cap, and the required JSON fields.
- `writing-devloop-plans/SKILL.md` — acceptance-criteria rules, the `**Compatible with:**` header flag, and the "web-app only" contradiction.
- `references/observe-modes.md`, `references/xcode-observe-modes.md` — static vs interactive classification and pass criteria for the two visual toolkits.
- `references/api-observe-modes.md`, `references/cli-observe-modes.md` — keyword lists, test-plus-live-hit procedure, evidence and proof shapes, failure-diagnosis tables.
- `references/url-discovery.md` — Traefik parsing algorithm, scheme resolution, ambiguity handling, reachability rules.
- `references/mx-cli-reference.md` — the mx command surface devloop assumes, including the `tail` note.
- `references/xcodebuildmcp-reference.md` — tool categories, detection, and the ToolSearch hedge on tool names.
- `examples/sample-task-result.json` — the concrete result shape.
- `manifest.json` — version, declared skills, author, license.
- `docs/development/multi-agent-systems-in-practice.md`, `docs/development/llm-token-cache-efficiency.md`, `docs/development/mx-app-playbook.md` (mech-crate) — existing corpus analysis of devloop's topology, token economics, and URL-discovery technique.
- Copy divergence established by `diff -r` against `~/.claude/skills/devloop`, `~/.codex/skills/devloop`, and `mech-crate/skills/{devloop,writing-devloop-plans}`; repo metadata via `git log`/`git status` and `gh api repos/nyvorin/devloop`.
