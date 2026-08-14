# mech-crate Test Baseline & Release Gating — Design Spec

**Date:** 2026-08-11
**Status:** Approved
**Repo:** mech-crate

## Overview

Establish a comprehensive, harness-maximal test suite for the mx workspace so every operation is testable, gate every release on it, and lay the TDD groundwork for fixing the ~22 tracked defects. Today: 83 Rust tests (mx-cli 29+9, mx-lib 45, **mx-mcp-server 0**), shell testbed not in CI, and a release workflow that ships signed binaries with **no test gate**.

## Decisions (settled during brainstorming)

| Decision | Choice |
|---|---|
| Tiers | Unit + integration gate CI on every PR/push; E2E (scaffold → router → URL) runs locally (`make test-e2e`) and via a `workflow_dispatch` GitHub Action only. |
| Gate shape | New `ci.yml` on PR + main; `release.yml` gains a `test` job that build/publish jobs `needs:` — a tag cannot ship without the suite passing on that commit. |
| Coverage | Green suite + **ratchet**: `cargo-llvm-cov` measured every run; floor in `.coverage-floor` starts at the baseline value; CI fails only if coverage drops below floor − 0.25; floor bumps ride coverage-raising PRs. No arbitrary target. |
| Known-broken defects | **TDD**: write tests asserting the *fixed* behavior now; annotate `#[ignore = "bd:mech-crate-xxx"]`; a `continue-on-error` CI lane runs them and scoreboards progress. Un-ignoring is each fix's definition of done. Gate stays green. |
| Harness level | **C — harness-maximal**: cargo-nextest runner, proptest where properties are real, wiremock fake embedding server fixture, cargo-mutants on a schedule/dispatch (never a PR gate). |

## Test architecture

**Runner:** cargo-nextest for unit/integration (CI + `make test`); `cargo test --doc` separately (nextest skips doc-tests). Per-test retries allowed ONLY in the E2E tier.

**Shared test infra** (`crates/mx-lib` `tests/common/` + a `test-support` feature where cross-crate reuse is needed):
- **Stub-bin PATH fixture**: temp dir prepended to PATH containing scripted fake `docker`/`make`/`mx` executables that record invocations and return canned outputs — unit-tests code that shells out, no daemons needed.
- **Tempdir project scaffolder**: builds a valid 4-marker mx project skeleton for CLI/upgrade/doctor tests.
- **Wiremock embedding server**: scripted 200s (with batch-order shuffling), 429s (+`retry-after-ms`), 5xx, malformed bodies; shared by embed-path unit tests and future ingest-resilience tests.
- **pgvector helper**: existing pattern preserved — `MX_RAG_TEST_DATABASE_URL` (55433 container locally; service container in CI), skip-if-unset, `db_lock()` serialization.

**Per-crate new surface:**

| Crate | Additions |
|---|---|
| mx-mcp-server | Unit: tool-registry dispatch, JSON-RPC request handling, error mapping, offline-corpus (`corpus: None`) message paths, executor arg-building against `MockMxExecutor`. Integration: `tests/mcp_stdio.rs` — spawn the real `mx-mcp` binary, drive initialize → tools/list → tools/call conversations (incl. rag_health backend reporting and the offline path), with child-process cleanup on drop. |
| mx-lib router | Port allocation (range, stale-cache reallocation), install file layout, ensure_network — via stub-bin `docker`. |
| mx-lib upgrade | Categorization matrix (tooling/config/conditional/skip), backup naming, path remaps — via tempdir template trees. Known-broken lane test: discovery against the real templates layout (bd: broken `mx upgrade`). |
| mx-lib recipe | **Strict validator** `recipe::validate` (unknown fields, unknown transforms, dangling template sources rejected) run against every real `templates/recipes/*/recipe.json`; installer round-trip into a tempdir project (files land per mapping, placeholders resolved); `common://` resolution. Kills the silent astro `npm_install`/`kebab` class. Validator is lib code, reusable by a future `mx recipes lint`. |
| mx-lib corpus | Existing 7 DB tests unchanged; proptest added (below). |
| mx-cli | assert_cmd: every subcommand `--help`, `rag ingest --dry-run` on fixtures, `doctor` on scaffolded tempdir, `rag gaps --help`. Stub-bin PATH for commands shelling to make/docker. |

**Proptest scope** (fixed `cases: 256`, `proptest-regressions/` committed): chunker (size bound incl. prefix, content preservation across chunks, heading-path correctness; fence-atomicity property added when that fix lands), frontmatter round-trip + never-panics-on-junk, placeholder transforms (idempotence, charset invariants), gaps query normalization equivalence.

**Mutation testing:** `cargo-mutants --package mx-lib`, timeout-capped, in `mutants.yml` on `workflow_dispatch` + weekly cron; report uploaded as artifact; missed mutants become test-backlog bd entries. Never a PR gate. `make test-mutants` locally.

## Known-broken TDD lane

- Each testable tracked defect (~14 of the 22; doc-drift/process items excluded — the plan enumerates the mapping) gets a test asserting fixed behavior, annotated `#[ignore = "bd:mech-crate-xxx"]`.
- `ci.yml` job `known-broken` (`continue-on-error: true`): `cargo nextest run --run-ignored only`, emitting a scoreboard: still-failing count, now-passing count with "un-ignore + close bd" instruction. (No collision with DB-gated tests — those skip via env-var early-return, not `#[ignore]`; the ignore attribute is reserved exclusively for the known-broken lane.)
- A known-broken test that PASSES is the signal that a fix landed without bookkeeping — surfaced, not silently green.
- Fix workflow (next initiative): claim bd issue → make the lane test pass → remove ignore → test joins the gate → close issue.

## CI & release wiring

**`ci.yml`** (PR + push to main), jobs:
1. `lint`: `cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings` (one-time cleanup of existing warnings, incl. `McpError::Corpus` dead code, is in scope).
2. `test`: pgvector service container (`pgvector/pgvector:pg17`, health-checked) + `MX_RAG_TEST_DATABASE_URL`; `make test` (= nextest workspace + doc-tests); Swatinem/rust-cache.
3. `coverage`: `cargo llvm-cov nextest` → line %; fail if `< floor − 0.25` (floor in `.coverage-floor`); `scripts/coverage-ratchet.sh [--bump]`.
4. `known-broken`: the scoreboard lane.

**`release.yml`**: new `test` job (lint + test + coverage, no known-broken) between `resolve-version` and the build/publish jobs via `needs:`.

**`e2e.yml`** (`workflow_dispatch`): build mx → testbed scaffold smokes (laravel, rust-api) with real Docker → `mx router install/up` on the runner (owns port 80) → assert `http://<svc>.localhost` reachable (`curl --resolve <host>:80:127.0.0.1`). Retries permitted here only. Mirrors `make test-e2e`.

**Makefile targets** (repo root; CI calls these — zero CI/local drift): `test`, `test-unit`, `test-int` (ensures the 55433 container), `test-e2e`, `test-known-broken`, `test-mutants`, `coverage [BUMP=1]`.

## Error handling & policies

- Local runs without Docker stay green (DB tests skip); CI always provides the service container so the gate never silently skips.
- stdio harness reaps spawned `mx-mcp` children on panic/drop (no CI zombies).
- Retry policy: E2E tier only; a flaky unit/integration test is a bug to fix, not retry.
- clippy `-D warnings` from day one; suppressions require an inline `#[allow]` with a reason comment.
- Coverage epsilon 0.25 prevents refactor false alarms; ratchet bumps are deliberate, reviewed changes.
- Corpus/OpenAI: no CI job ever touches Neon or real embedding APIs — wiremock and the service container only.

## Testing the tests (acceptance for this build)

- `ci.yml` goes green on the feature branch itself (the gate proves itself by gating its own PR); a deliberately-broken commit on a scratch branch demonstrates each gate failing (lint, test, coverage-drop, release-test).
- Known-broken lane shows the expected ~14 red with bd ids; zero unexpected reds in the main suite.
- Coverage baseline recorded in `.coverage-floor` and reported in CI output.
- Devloop executes the plan with cli-toolkit acceptance criteria per task.

## Out of scope

- Fixing the tracked defects themselves (next initiative; unlocked one un-ignore at a time).
- nextest-based flake quarantine automation, CI test-sharding (revisit if suite time exceeds ~5 min).
- Windows/other-OS CI matrices (macOS signing already covered in release.yml's existing jobs).
- Testing the techniques-research/corpus *content* pipeline beyond existing corpus tests (eval harness is tracked separately in bd: corpus retrieval quality).
