# mech-crate Test Baseline & Release Gating Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harness-maximal test suite for the mx workspace (nextest, fixtures, proptest, mutants), a known-broken TDD lane linked to bd issues, a coverage ratchet, and CI/release gating so no tag ships untested.

**Architecture:** Shared `test-support` fixtures in mx-lib (stub-bin PATH, tempdir scaffolder, wiremock embedding server) power new unit/integration suites across all three crates — headlined by mx-mcp-server going 0→covered plus a real stdio harness. Known-broken defects get `#[ignore = "bd:<id>"]` tests asserting fixed behavior, scoreboarded by a continue-on-error CI lane. `ci.yml` (lint/test/coverage/known-broken) gates PRs and main; `release.yml` gains a `needs:` test job; `e2e.yml` and `mutants.yml` are dispatch-only. Makefile targets are the single entry points CI itself calls.

**Tech Stack:** cargo-nextest, cargo-llvm-cov, cargo-mutants, proptest, wiremock, assert_cmd/predicates, GitHub Actions (pgvector service container, taiki-e/install-action, Swatinem/rust-cache).

**Spec:** `docs/superpowers/specs/2026-08-11-test-baseline-design.md`

**Compatible with:** devloop skill v0.1+

## Global Constraints

- Runner: `cargo nextest run` for unit/integration + `cargo test --doc` for doc-tests. Retries NEVER in unit/integration profiles (e2e tier only).
- `#[ignore = "bd:mech-crate-…"]` is reserved EXCLUSIVELY for the known-broken lane. DB-gated tests keep the existing env-var early-return pattern.
- Test DB: `MX_RAG_TEST_DATABASE_URL=postgres://postgres@localhost:55433/mx_rag` (container `mx-rag-test`; `docker start mx-rag-test` if exited). DB tests skip when unset and serialize on `db_lock()`.
- No CI job touches Neon or real embedding APIs — wiremock + service container only.
- clippy `-D warnings` workspace-wide; any `#[allow]` needs an inline reason comment.
- Local tooling: `cargo install cargo-nextest cargo-llvm-cov cargo-mutants` (subagents: check `which` first; install if missing).
- Conventional commit per task; `cargo fmt` on touched files (git-restore unrelated fallout); trailer:
  `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>` + `Claude-Session: https://claude.ai/code/session_01QQozAZdFXi7WfRBxLpx3Ru`
- Branch: `feat/test-baseline`.

---

### Task 1: Nextest bootstrap + Makefile test targets

**Acceptance Criteria (observable):**
- `make test` exits 0, visibly running the existing suite via nextest (output contains "Nextest run") plus doc-tests.
- `make test-unit` exits 0 running only lib/bin unit tests (fast path, no DB needed).
- `make test-int` starts/reuses the mx-rag-test container then runs the workspace suite with the env var set, exit 0.
- `.config/nextest.toml` defines `default` and `ci` profiles with `retries = 0`.

**Verify via:** cli

**Files:**
- Create: `.config/nextest.toml`
- Modify: `Makefile` (append a Testing section)
- Modify: `Cargo.toml` (workspace dev-deps: `proptest = "1"`; verify `assert_cmd`/`predicates` present in mx-cli, add to workspace if not)

**Interfaces:**
- Produces: `make test|test-unit|test-int` — the commands every later task and CI job invokes.

- [x] **Step 1: nextest config**

`.config/nextest.toml`:

```toml
[profile.default]
retries = 0
failure-output = "immediate-final"

[profile.ci]
retries = 0
failure-output = "immediate-final"
fail-fast = false
```

- [x] **Step 2: Makefile targets**

Append to the root `Makefile`:

```makefile
## ─── Testing ────────────────────────────────────────────────────────────────
TEST_DB_URL ?= postgres://postgres@localhost:55433/mx_rag

.PHONY: test test-unit test-int test-known-broken coverage test-e2e test-mutants

## Run the full gate suite (what CI runs)
test:
	cargo nextest run --workspace --profile ci
	cargo test --workspace --doc

## Fast unit-only loop (no DB)
test-unit:
	cargo nextest run --workspace --lib --bins

## Integration with the local pgvector container
test-int:
	@docker start mx-rag-test 2>/dev/null || docker run -d --name mx-rag-test -p 55433:5432 -e POSTGRES_DB=mx_rag -e POSTGRES_HOST_AUTH_METHOD=trust pgvector/pgvector:pg17
	@sleep 2
	MX_RAG_TEST_DATABASE_URL=$(TEST_DB_URL) cargo nextest run --workspace --profile ci
	MX_RAG_TEST_DATABASE_URL=$(TEST_DB_URL) cargo test --workspace --doc

## Known-broken TDD lane (expected red; scoreboard)
test-known-broken:
	-MX_RAG_TEST_DATABASE_URL=$(TEST_DB_URL) cargo nextest run --workspace --run-ignored only

## Coverage with ratchet check (BUMP=1 to raise the floor)
coverage:
	./scripts/coverage-ratchet.sh $(if $(BUMP),--bump,)
```

(`test-e2e` and `test-mutants` are added in Tasks 14/15; declare them `.PHONY` now, define later.)

- [x] **Step 3: deps + verify**

Add `proptest = "1"` to `[workspace.dependencies]` and to mx-lib `[dev-dependencies]` (`proptest = { workspace = true }`). Confirm mx-cli dev-deps include `assert_cmd` + `predicates` (they power tests/cli_tests.rs; if referenced only as direct versions, normalize to workspace deps).

Run: `which cargo-nextest || cargo install cargo-nextest` then `make test` / `make test-unit` / `make test-int`.
Expected: all exit 0; nextest banner visible; current counts (≥83 tests) pass.

- [x] **Step 4: Commit**

```bash
git add .config/nextest.toml Makefile Cargo.toml crates/mx-lib/Cargo.toml crates/mx-cli/Cargo.toml Cargo.lock
git commit -m "feat(test): nextest bootstrap + make test targets"
```

---

### Task 2: Shared test-support fixtures in mx-lib

**Acceptance Criteria (observable):**
- `cargo nextest run -p mx-lib test_support` exits 0 with self-tests proving: the stub-bin fixture makes a fake `docker` first-on-PATH, records argv lines to a log file, and returns scripted stdout/exit codes; the scaffolder produces a directory passing the strict 4-marker project detection; the wiremock embedding fixture serves scripted 200/429/500 responses.
- `cargo check -p mx-cli --features mx-lib/test-support` and `-p mx-mcp-server` equivalents exit 0 (fixtures consumable cross-crate).

**Verify via:** cli

**Apply:** docs/development/appendix-rust.md — A6: "Contract tests for core/ports against adapter fakes" — the stub-bin and wiremock fixtures ARE the adapter fakes; keep them under a feature gate so they never ship in release binaries.

**Files:**
- Create: `crates/mx-lib/src/test_support/mod.rs` (+ `stub_bin.rs`, `scaffold.rs`, `embed_server.rs`)
- Modify: `crates/mx-lib/src/lib.rs` (`#[cfg(feature = "test-support")] pub mod test_support;`)
- Modify: `crates/mx-lib/Cargo.toml` (feature `test-support = []`; move `wiremock`/`tempfile` from dev-deps to optional deps activated by the feature, keeping them in dev-deps too)
- Modify: `crates/mx-cli/Cargo.toml`, `crates/mx-mcp-server/Cargo.toml` (dev-dep on `mx-lib` with `features = ["test-support"]`)

**Interfaces:**
- Produces:
  ```rust
  pub struct StubBin { /* tempdir with fake executables */ }
  impl StubBin {
      pub fn new() -> Self;
      pub fn stub(&self, name: &str, exit_code: i32, stdout: &str) -> &Self;  // writes a sh script
      pub fn path_env(&self) -> String;          // "<stubdir>:<original PATH>"
      pub fn invocations(&self, name: &str) -> Vec<String>; // recorded "argv..." lines
  }
  pub fn scaffold_project(root: &std::path::Path);  // Makefile, docker/{compose,.config,system,dockerfiles}, make/, scripts/, tmp/up
  pub struct EmbedServer { pub uri: String, /* wiremock server */ }
  impl EmbedServer {
      pub async fn ok(dims: usize) -> Self;              // 200s, per-input vectors, index-shuffled
      pub async fn rate_limited(retry_after_ms: u64) -> Self;  // always 429 + retry-after-ms header
      pub async fn failing(status: u16) -> Self;
  }
  ```

- [x] **Step 1: Write the failing self-tests**

`crates/mx-lib/src/test_support/mod.rs` bottom (tests compile only with the feature; `cargo nextest run -p mx-lib --features test-support` — note nextest passes features through):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn stub_bin_intercepts_and_records() {
        let sb = StubBin::new();
        sb.stub("docker", 0, "STUBBED-DOCKER-OK");
        let out = Command::new("docker")
            .args(["network", "inspect", "devmesh-traefik"])
            .env("PATH", sb.path_env())
            .output()
            .unwrap();
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "STUBBED-DOCKER-OK");
        let calls = sb.invocations("docker");
        assert_eq!(calls.len(), 1);
        assert!(calls[0].contains("network inspect devmesh-traefik"));
    }

    #[test]
    fn stub_bin_scripts_exit_codes() {
        let sb = StubBin::new();
        sb.stub("docker", 3, "");
        let out = Command::new("docker").env("PATH", sb.path_env()).output().unwrap();
        assert_eq!(out.status.code(), Some(3));
    }

    #[test]
    fn scaffold_passes_strict_detection() {
        let dir = tempfile::tempdir().unwrap();
        scaffold_project(dir.path());
        let det = crate::project::ProjectDetector::strict();
        assert!(det.is_project(dir.path()));
        assert!(dir.path().join("docker/compose").is_dir());
    }

    #[tokio::test]
    async fn embed_server_modes() {
        let ok = EmbedServer::ok(4).await;
        let resp = reqwest::Client::new()
            .post(format!("{}/embeddings", ok.uri))
            .json(&serde_json::json!({"model":"m","input":["a","b"]}))
            .send().await.unwrap();
        assert!(resp.status().is_success());
        let v: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(v["data"].as_array().unwrap().len(), 2);

        let rl = EmbedServer::rate_limited(1200).await;
        let resp = reqwest::Client::new()
            .post(format!("{}/embeddings", rl.uri))
            .json(&serde_json::json!({"model":"m","input":"a"}))
            .send().await.unwrap();
        assert_eq!(resp.status().as_u16(), 429);
        assert_eq!(resp.headers()["retry-after-ms"], "1200");
    }
}
```

- [x] **Step 2: Run to verify failure** — `cargo nextest run -p mx-lib --features test-support test_support` → compile error (types missing).

- [x] **Step 3: Implement**

`stub_bin.rs` — each `stub()` writes `<dir>/<name>`:

```sh
#!/bin/sh
echo "$@" >> "<dir>/.calls-<name>"
printf '%s\n' '<stdout>'
exit <exit_code>
```

chmod 0755; `invocations()` reads the calls file; `path_env()` = `format!("{}:{}", dir, std::env::var("PATH").unwrap_or_default())`. `scaffold.rs` creates: `Makefile` (one `help:` target), `docker/{compose,.config,system,dockerfiles}`, `make/`, `scripts/`, `tmp/up`. `embed_server.rs` wraps wiremock: `ok(dims)` responds per-input `{index, embedding:[0.0; dims]}` with data array REVERSED (exercises order-restoring clients); `rate_limited` sets the `retry-after-ms` header; `failing(status)` returns the status empty-bodied.

Wire the feature/deps per the Files list.

- [x] **Step 4: Run to verify pass** — the four self-tests green; cross-crate `cargo check`s green.

- [x] **Step 5: Commit** — `git add -A && git commit -m "feat(test): shared test-support fixtures (stub-bin, scaffolder, embed server)"`

---

### Task 3: Recipe validator + real-recipe conformance tests

**Acceptance Criteria (observable):**
- `cargo nextest run -p mx-lib recipe::validate` exits 0, including a test that validates EVERY `templates/recipes/*/recipe.json` on disk with zero findings.
- Running the validator against a fixture containing `post_install.npm_install` and `transform: "kebab"` reports BOTH as unknown-field/unknown-transform findings (the silent-astro class is now loud).
- `templates/recipes/astro/recipe.json` no longer contains `npm_install` or `"kebab"` (data fixed so the on-disk sweep is clean).

**Verify via:** cli

**Files:**
- Create: `crates/mx-lib/src/recipe/validate.rs`
- Modify: `crates/mx-lib/src/recipe/mod.rs` (`pub mod validate;`)
- Modify: `templates/recipes/astro/recipe.json` (remove `npm_install`; change `"kebab"` → `"slug"`)
- Test: inline `#[cfg(test)]` in validate.rs + `crates/mx-lib/tests/recipes_conformance.rs`

**Interfaces:**
- Produces: `pub fn validate_recipe_json(raw: &serde_json::Value) -> Vec<Finding>` and `pub struct Finding { pub path: String, pub message: String }`. Allowed-key sets mirror `parser.rs` structs exactly (Recipe, options, placeholders {source, transform ∈ slug|upper|rust_crate|ssr_bool}, init_app, templates {from,to,condition}, post_install {create_files, rename, chmod, gitkeep, run, gitignore}, next_steps).

- [x] **Step 1: failing tests** — validate.rs unit tests: clean minimal recipe → `[]`; recipe with `post_install.npm_install` → finding mentioning `npm_install`; placeholder `transform: "kebab"` → finding; template `from` referencing `common://nope` → finding when the resolved path is absent (validator takes an optional recipes-root for that check). `recipes_conformance.rs`:

```rust
#[test]
fn every_shipped_recipe_validates_clean() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/recipes");
    let mut checked = 0;
    for entry in std::fs::read_dir(&root).unwrap().flatten() {
        let rj = entry.path().join("recipe.json");
        if !rj.exists() { continue; }
        let raw: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&rj).unwrap()).unwrap();
        let findings = mx_lib::recipe::validate::validate_recipe_json_with_root(&raw, Some(&root));
        assert!(findings.is_empty(), "{}: {:?}", rj.display(), findings);
        checked += 1;
    }
    assert!(checked >= 7, "expected >=7 recipes, found {checked}");
}
```

- [x] **Step 2: red** — compile failure. Also run the conformance test AFTER implementing but BEFORE fixing astro to watch it fail on the real defect (evidence for the commit message), then fix astro.

- [x] **Step 3: implement** — walk the JSON with per-object allowlists; unknown key → Finding; placeholder transform not in the four → Finding; `templates[].from` with `common://` prefix and root provided → check existence. Keep it pure (no fs unless root given).

- [x] **Step 4: green** — validator units + conformance sweep pass with astro fixed.

- [x] **Step 5: Commit** — `"feat(recipe): strict validator + on-disk conformance sweep (fix astro recipe)"`

---

### Task 4: mx-mcp-server unit tests (registry, dispatch, offline paths)

**Acceptance Criteria (observable):**
- `cargo nextest run -p mx-mcp-server --lib --bins` exits 0 with ≥12 new tests proving: `ToolRegistry::list_all()` returns the full tool set including all 8 `rag_*` names; `execute()` with an unknown tool name errors `ToolNotFound`; missing required args on representative tools (`mx_new` without `name`, `rag_context` without `working_on`) error `InvalidArguments` WITHOUT any subprocess spawned; every rag handler with `corpus: None` returns text containing "offline" and the rag.toml hint.
- Zero real `mx`/`docker` processes spawned during these tests (stub-bin PATH set defensively; its invocation log stays empty).

**Verify via:** cli

**Apply:** docs/development/appendix-rust.md — A5/A6: domain errors as meaningful variants + contract tests; assert on `McpError` variants, not strings, except for the user-facing offline text.

**Files:**
- Modify: `crates/mx-mcp-server/src/tools/mod.rs` (append `#[cfg(test)] mod tests`)
- Modify: `crates/mx-mcp-server/Cargo.toml` (dev-deps: tokio-test present; add `mx-lib` test-support feature)

- [x] **Step 1: failing tests** — representative code (full set enumerated below):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mx::MxExecutor;
    use crate::project::ProjectDetector;
    use mx_lib::test_support::StubBin;

    fn reg() -> ToolRegistry { ToolRegistry::new() }
    fn mx_exec() -> MxExecutor { MxExecutor::new(std::path::PathBuf::from("/nonexistent-root")) }

    #[test]
    fn registry_lists_all_rag_tools() {
        let names: Vec<String> = reg().list_all().into_iter().map(|t| t.name).collect();
        for n in ["rag_context","rag_search","rag_search_category","rag_find_implementation",
                  "rag_get_guidance","rag_compare_approaches","rag_find_related","rag_health"] {
            assert!(names.contains(&n.to_string()), "missing {n}");
        }
        assert!(names.len() >= 40, "tool surface shrank: {}", names.len());
    }

    #[tokio::test]
    async fn unknown_tool_is_tool_not_found() {
        let sb = StubBin::new(); std::env::set_var("PATH", sb.path_env());
        let err = reg().execute("nope", serde_json::json!({}), &mx_exec(), &ProjectDetector::new(), None)
            .await.unwrap_err();
        assert!(matches!(err, crate::error::McpError::ToolNotFound(_)));
    }

    #[tokio::test]
    async fn missing_required_arg_is_invalid_arguments_and_spawns_nothing() {
        let sb = StubBin::new(); sb.stub("mx", 0, ""); std::env::set_var("PATH", sb.path_env());
        let err = reg().execute("mx_new", serde_json::json!({}), &mx_exec(), &ProjectDetector::new(), None)
            .await.unwrap_err();
        assert!(matches!(err, crate::error::McpError::InvalidArguments(_)));
        assert!(sb.invocations("mx").is_empty());
    }

    #[tokio::test]
    async fn rag_tools_offline_message() {
        for (tool, args) in [
            ("rag_context", serde_json::json!({"working_on":"x"})),
            ("rag_search", serde_json::json!({"query":"x"})),
            ("rag_health", serde_json::json!({})),
        ] {
            let res = reg().execute(tool, args, &mx_exec(), &ProjectDetector::new(), None).await.unwrap();
            let text = format!("{:?}", res);
            assert!(text.contains("offline") && text.contains("rag.toml"), "{tool}: {text}");
        }
    }
}
```

Full enumerated set (each its own test fn, same patterns): offline for all 8 rag tools; `rag_context` missing `working_on`; `rag_search_category` missing `category`; `mx_recipe_info` missing `recipe`; `mx_build` missing `service`; registry has no duplicate names; every tool's `input_schema.schema_type == "object"`.

NOTE: env-var PATH mutation in tests → keep all PATH-mutating tests in ONE `#[tokio::test]`-per-case module and set PATH inside each test through `Command` env instead where the code allows; where `execute()` spawns via inherited env, serialize with a module-level `std::sync::Mutex` guard (same pattern as corpus `db_lock`).

- [x] **Step 2: red** → **Step 3: implement seams if needed** (expected: none — these paths validate before spawning; if a path DOES spawn pre-validation, fix the ordering as part of this task and note it) → **Step 4: green** → **Step 5: Commit** `"test(mcp): registry/dispatch/offline coverage for the tool surface"`

---

### Task 5: MCP stdio integration harness

**Acceptance Criteria (observable):**
- `MX_RAG_TEST_DATABASE_URL=… cargo nextest run -p mx-mcp-server --test mcp_stdio` exits 0: spawns the built `mx-mcp` binary, completes initialize → tools/list (contains `"rag_context"`) → `rag_health` call reporting `"backend": "local"` against the test container.
- The offline test (fallback URL forced to `postgres://postgres@localhost:1/nope`, no env DB) gets the offline message and the child exits cleanly.
- Without `MX_RAG_TEST_DATABASE_URL`, the online test skips (early return), suite still exit 0; no orphaned `mx-mcp` processes remain after the run (`pgrep` in the test's Drop guard).

**Verify via:** cli

**Files:**
- Create: `crates/mx-mcp-server/tests/mcp_stdio.rs`

**Interfaces:**
- Produces: `struct McpChild` harness (spawn via `assert_cmd`-located binary or `env!("CARGO_BIN_EXE_mx-mcp")`, line-delimited JSON-RPC write/read with timeout, `Drop` = kill + wait).

- [x] **Step 1: failing test skeleton** (representative — the harness struct + three tests):

```rust
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

struct McpChild { child: Child, reader: BufReader<std::process::ChildStdout> }

impl McpChild {
    fn spawn(envs: &[(&str, &str)]) -> Self {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_mx-mcp"));
        cmd.arg("--mech-crate-root").arg(env!("CARGO_MANIFEST_DIR").to_string() + "/../..")
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
        for (k, v) in envs { cmd.env(k, v); }
        let mut child = cmd.spawn().expect("spawn mx-mcp");
        let reader = BufReader::new(child.stdout.take().unwrap());
        Self { child, reader }
    }
    fn send(&mut self, v: serde_json::Value) {
        let stdin = self.child.stdin.as_mut().unwrap();
        writeln!(stdin, "{}", v).unwrap();
    }
    fn recv_id(&mut self, id: u64) -> serde_json::Value {
        // read lines until matching id or 30s deadline; panic with context on timeout
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        let mut line = String::new();
        loop {
            assert!(std::time::Instant::now() < deadline, "timeout waiting for id {id}");
            line.clear();
            if self.reader.read_line(&mut line).unwrap() == 0 { panic!("eof before id {id}"); }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                if v["id"] == serde_json::json!(id) { return v; }
            }
        }
    }
    fn init(&mut self) {
        self.send(serde_json::json!({"jsonrpc":"2.0","id":0,"method":"initialize",
            "params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}));
        self.recv_id(0);
    }
}
impl Drop for McpChild { fn drop(&mut self) { let _ = self.child.kill(); let _ = self.child.wait(); } }

#[test]
fn tools_list_includes_rag_context() {
    let mut c = McpChild::spawn(&[("MX_RAG_FALLBACK_DATABASE_URL", "postgres://postgres@localhost:1/nope"),
                                  ("MX_RAG_DATABASE_URL", "postgres://postgres@localhost:1/nope")]);
    c.init();
    c.send(serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}));
    let v = c.recv_id(1);
    assert!(v.to_string().contains("rag_context"));
}

#[test]
fn rag_health_reports_local_backend() {
    let Ok(url) = std::env::var("MX_RAG_TEST_DATABASE_URL") else { return }; // skip
    let mut c = McpChild::spawn(&[("MX_RAG_DATABASE_URL", &url)]);
    c.init();
    c.send(serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/call",
        "params":{"name":"rag_health","arguments":{}}}));
    let v = c.recv_id(2).to_string();
    assert!(v.contains("backend"), "{v}");
    assert!(v.contains("local") || v.contains("neon"), "{v}");
}

#[test]
fn offline_rag_health_is_graceful() {
    let mut c = McpChild::spawn(&[("MX_RAG_DATABASE_URL", "postgres://postgres@localhost:1/nope"),
                                  ("MX_RAG_FALLBACK_DATABASE_URL", "postgres://postgres@localhost:1/nope")]);
    c.init();
    c.send(serde_json::json!({"jsonrpc":"2.0","id":3,"method":"tools/call",
        "params":{"name":"rag_health","arguments":{}}}));
    let v = c.recv_id(3).to_string();
    assert!(v.contains("offline") || v.contains("rag.toml"), "{v}");
}
```

CAVEAT for the implementer: `MX_RAG_DATABASE_URL` env overrides the user's rag.toml (config precedence from Task 5 of the corpus build) — the harness always pins BOTH URLs so a dev machine's Neon config can't leak in. `rag_health` with a reachable test-DB URL as PRIMARY reports backend "neon" per BackendKind labeling — accept either label, assert on reachability not label.

- [x] **Step 2: red** (file compiles, binary path resolves, initial run may hang → fix protocol assumptions) → **Step 3: adjust harness until green with and without env** → **Step 4: `pgrep -f mx-mcp` clean after run** → **Step 5: Commit** `"test(mcp): stdio integration harness (handshake, health, offline)"`

---

### Task 6: Router module unit tests (stub-docker)

**Acceptance Criteria (observable):**
- `cargo nextest run -p mx-lib router` exits 0 with tests proving: dashboard-port allocation stays within 7680–7799 and re-allocates when the cached value is out of range; `is_installed()` keys on `docker-compose.yml` presence under an overridden home; `ensure_network`/`start` invoke `docker` with the expected subcommands (recorded by stub-bin) and never touch the real daemon.

**Verify via:** cli

**Files:**
- Modify: `crates/mx-lib/src/router/mod.rs` (append tests; introduce a small `home_override` seam ONLY if the config path resolution cannot be redirected via `HOME`/env in tests — prefer env redirection)

- [x] Steps follow the standard TDD cycle. Key mechanics: tests create a tempdir fake `~/.mech-crate/router` (write `docker-compose.yml`, `.dashboard-port` with an out-of-range value like `9999`), point the router's home resolution at it (set `HOME` to the tempdir parent — serialize env-mutating tests with a module mutex), stub `docker` with recorded invocations. Assert: port re-allocation warns+reallocates into range; `docker compose -p mx-router up -d`-shaped invocation recorded on start; network ensure invokes `docker network` commands. Commit: `"test(router): port allocation, install detection, docker invocation contracts"`

---

### Task 7: Upgrade engine tests + first known-broken lane test

**Acceptance Criteria (observable):**
- `cargo nextest run -p mx-lib upgrade` exits 0: categorization matrix verified (tooling→prompt-update, docker config→add-only, `recipes/`→skip, cloudflare→conditional), `.bak` backup naming, `Makefile.template`→`Makefile` and `docker/config/env.*`→`docker/.config/.env.*` remaps — all against synthetic tempdir template trees.
- One `#[ignore = "bd:<real-id>"]` test exists asserting `discover_upgrades()` SUCCEEDS against the repo's real `templates/` layout (the currently-broken path); `cargo nextest run -p mx-lib --run-ignored only upgrade` shows it FAILING (red = correctly encoding the bug).

**Verify via:** cli

**Files:**
- Modify: `crates/mx-lib/src/upgrade/mod.rs` (append tests)

- [x] Steps: standard TDD for the green set (synthetic `templates/project/` trees in tempdirs exercise categorize/backup/remap logic as pure-ish units); the known-broken test uses the real templates dir via `CARGO_MANIFEST_DIR` and asserts `Ok` + non-empty upgrade set — currently `Err("Project templates not found…")`. Fetch the real bd id: `bd list | grep -i "mx upgrade"` → use its id in the ignore reason. Commit: `"test(upgrade): categorization matrix + known-broken discovery test (bd)"`

---

### Task 8: mx-cli assert_cmd expansion

**Acceptance Criteria (observable):**
- `cargo nextest run -p mx-cli --test cli_surface` exits 0: every subcommand in the enumerated list responds to `--help` with exit 0 and non-empty usage text; `mx rag ingest --dry-run --path <fixture>` on a 2-doc fixture prints `2 docs` and `0 warnings`; `mx doctor` on a scaffolded tempdir project exits 0; `mx recipes list` exits 0 listing ≥7 recipes.

**Verify via:** cli

**Apply:** docs/development/MX_RUST_CLI_AND_MCP_SERVER.md — Testing section's assert_cmd pattern (Command::cargo_bin + predicates) is the house style; follow it.

**Files:**
- Create: `crates/mx-cli/tests/cli_surface.rs`
- Create: `crates/mx-cli/tests/fixtures/ragdocs/{a.md,b.md}` (valid frontmatter, tiny bodies)

- [x] **Step 1: the loop-driven help sweep + targeted tests:**

```rust
use assert_cmd::Command;
use predicates::str::is_empty;

const SUBCOMMANDS: &[&str] = &[
    "init","new","add","recipes","dev","up","down","logs","restart","sh","ps",
    "build","docs","router","infra","mcp","doctor","unyform","rag","upgrade","self-update",
];

#[test]
fn every_subcommand_has_help() {
    for sub in SUBCOMMANDS {
        let mut cmd = Command::cargo_bin("mx").unwrap();
        cmd.args([sub, "--help"]).assert().success().stdout(is_empty().not());
    }
}

#[test]
fn rag_dry_run_on_fixture() {
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ragdocs");
    Command::cargo_bin("mx").unwrap()
        .args(["rag", "ingest", "--dry-run", "--path", fixture])
        .assert().success()
        .stdout(predicates::str::contains("2 docs").and(predicates::str::contains("0 warnings")));
}

#[test]
fn doctor_on_scaffolded_project() {
    let dir = tempfile::tempdir().unwrap();
    mx_lib::test_support::scaffold_project(dir.path());
    Command::cargo_bin("mx").unwrap()
        .current_dir(dir.path()).arg("doctor").assert().success();
}
```

(Adjust the SUBCOMMANDS list to the actual enum in main.rs — read it first; login/logout/whoami/cc-plugin included if present. `doctor` may need the stub-bin PATH for docker checks — wire `sb.path_env()` if it probes the daemon.)

- [x] Steps 2-4 standard. Commit: `"test(cli): full subcommand surface + dry-run/doctor fixtures"`

---

### Task 9: Proptest suite

**Acceptance Criteria (observable):**
- `cargo nextest run -p mx-lib prop_` exits 0 running property tests (256 cases each) for: chunker size bound (`content.len() <= max + heading prefix + 2`), chunker content preservation (every non-heading input paragraph substring-present in the concatenated chunks), frontmatter round-trip (`parse(render(meta)) == meta`) and never-panics-on-arbitrary-input, placeholder transforms (slug idempotent, output charset `[a-z0-9-]`; upper_snake charset; rust_crate valid identifier).
- `proptest-regressions/` directory is committed (empty seed files created on first failures, if any).

**Verify via:** cli

**Apply:** docs/development/appendix-rust.md — A6: "Property tests for invariants and decoders/encoders" — this is that line, made real.

**Files:**
- Create: `crates/mx-lib/tests/prop_chunker.rs`, `crates/mx-lib/tests/prop_frontmatter.rs`, `crates/mx-lib/tests/prop_transforms.rs`

- [x] **Step 1: representative property (chunker bound; others follow the same shape):**

```rust
use proptest::prelude::*;
use mx_lib::corpus::chunk::{chunk_markdown, DEFAULT_CHUNK_CHARS};

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    #[test]
    fn chunks_respect_budget(title in "[A-Za-z ]{1,40}", body in "(##? [A-Za-z ]{1,30}\n|[A-Za-z ,.]{0,200}\n){0,40}") {
        let max = DEFAULT_CHUNK_CHARS;
        for c in chunk_markdown(&title, &body, max) {
            let prefix_len = c.heading_path.len() + 2;
            prop_assert!(c.content.len() <= max + prefix_len,
                "chunk {} > budget {} (+prefix {})", c.content.len(), max, prefix_len);
        }
    }
}
```

Frontmatter round-trip renders a `TechniqueMeta` to YAML between `---` fences and reparses; never-panics feeds `any::<String>()` through `parse_frontmatter` asserting no panic (result irrelevant). Transforms call the parser's transform fns (make them `pub(crate)`-visible to tests or test through placeholder resolution).

Spec deviation, noted: the spec lists "gaps query normalization equivalence" under proptest — that normalization lives in SQL (`lower + regexp_replace`), so a 256-case property against the DB is disproportionate; it is already covered by the existing `gaps_aggregates_weak_queries` DB test's case/whitespace variants. No new proptest file for it.

- [x] Steps 2-4 standard (red on compile, implement visibility seams minimally, green). Commit: `"test(prop): chunker/frontmatter/transform property suites"`

---

### Task 10: Known-broken TDD lane (bd-linked ignored tests)

**Acceptance Criteria (observable):**
- `make test-known-broken` runs the ignored lane; the scoreboard output shows ≥12 known-broken tests, each named `kb_<slug>` and carrying `#[ignore = "bd:mech-crate-<id>"]` with a REAL id from `bd list`; the currently-expected state is: all red except any that accidentally already pass (none expected).
- `make test` (the gate) remains fully green — no ignored test runs in it.
- A `docs/development/RESEARCH_LOG.md`-style mapping table exists at `tests/KNOWN_BROKEN.md` (repo root `tests/` dir): bd id ↔ test path ↔ asserted fixed behavior ↔ tier.

**Verify via:** cli

**Files:**
- Create: `tests/KNOWN_BROKEN.md`
- Modify: test files across crates (each lane test lives beside its subject module/suite)

**The lane tests** (implementer: `bd list` first, map titles→ids; write each test asserting the FIXED behavior; ~1-6 lines of arrange each given Task 2's fixtures):

| # | bd title (grep key) | Test asserts (when fixed) | Where |
|---|---|---|---|
| 1 | mx upgrade reads non-existent | (already written in Task 7) | mx-lib upgrade |
| 2 | CLOUDFLARE_ACCOUNT_ID vs CF_ACCOUNT_ID | `mx infra setup cloudflare`-written global file contains `CF_ACCOUNT_ID=` (or the mk reads the new name — assert the CONTRACT: the var name written == the var name `cloudflare.mk` includes; test greps both sides and asserts equality) | mx-cli tests |
| 3 | mx infra link/unlink stubs | `mx infra link cloudflare` in a scaffolded project creates the agreed marker AND `mx infra inspect` resolves to global | mx-cli tests |
| 4 | phantom mx cf | `mx cf --help` exits 0 (implement-path assumption; if docs-fix chosen instead, retire this test when closing the issue) | mx-cli tests |
| 5 | apply --fix dead flag | with wiremock unyform stub: `mx recipes apply x --fix` output differs from without `--fix` (fix performs dependency comparison) | mx-cli tests (wiremock) |
| 6 | org id vs slug | both Unyform clients hit `/v1/orgs/<SAME segment>/recipes` against a wiremock recording server | mx-lib unyform |
| 7 | mx build --platform dropped | stub-bin `make` records `platform=linux/arm64` forwarded from `mx build svc --platform linux/arm64` | mx-cli tests |
| 8 | no make rebuild | scaffolded project: `make -n rebuild` exits 0 | mx-cli tests |
| 9 | recipe apps missing release scripts | nuxt recipe's rendered `package.json` contains a `release` script | mx-lib recipe |
| 10 | source-root vs source marker | `self_update`'s source-dir resolution reads the SAME path `paths::source_root_file()` writes (unit: resolution fn returns the recorded root) | mx-lib paths/self_update seam |
| 11 | recipe version dead metadata | `mx recipes info nuxt` output contains `3.15` | mx-cli tests |
| 12 | router inspect parity | Rust `mx router inspect` output lists connected services (stub-docker returns a network-inspect JSON with 2 containers → both names appear) | mx-cli tests |
| 13 | doctor router checks | `mx doctor` output mentions router status (stub-docker) | mx-cli tests |
| 14 | corpus lexical arm inert | DB test: relevant-vs-irrelevant lexical separation ≥ 5× on two seeded 1.2KB chunks (passes once strict_word_similarity/tsvector/BM25 lands) | mx-lib corpus (env-gated INSIDE the ignored test) |

Representative shape (test 8):

```rust
#[test]
#[ignore = "bd:mech-crate-XXX no make rebuild target"]
fn kb_make_rebuild_exists() {
    let dir = tempfile::tempdir().unwrap();
    mx_lib::test_support::scaffold_project(dir.path());
    let out = std::process::Command::new("make").args(["-n", "rebuild"])
        .current_dir(dir.path()).output().unwrap();
    assert!(out.status.success(), "make rebuild should exist: {}", String::from_utf8_lossy(&out.stderr));
}
```

- [x] Steps: write all lane tests (they must COMPILE and FAIL when run with `--run-ignored only`; a lane test that errors on missing fixtures instead of failing on the assertion is a bug — arrange must succeed); author `tests/KNOWN_BROKEN.md`; verify gate stays green; commit `"test(kb): known-broken TDD lane — 14 bd-linked ignored tests"`

---

### Task 11: Coverage ratchet

**Acceptance Criteria (observable):**
- `make coverage` exits 0, printing `coverage: NN.N% (floor: NN.N%)`; `.coverage-floor` exists containing the recorded baseline number.
- Editing `.coverage-floor` upward by 5 makes `make coverage` FAIL with a drop message (demonstrated, then reverted).
- `make coverage BUMP=1` rewrites the floor to current and exits 0.

**Verify via:** cli

**Files:**
- Create: `scripts/coverage-ratchet.sh`
- Create: `.coverage-floor`

- [x] **Step 1: the script:**

```bash
#!/usr/bin/env bash
# Coverage ratchet: fail if line coverage drops >0.25 below the recorded floor.
set -euo pipefail
FLOOR_FILE="$(dirname "$0")/../.coverage-floor"
EPSILON=0.25
current=$(cargo llvm-cov nextest --workspace --summary-only 2>/dev/null \
  | awk '/^TOTAL/ {print $(NF-2)}' | tr -d '%')
[ -n "$current" ] || { echo "could not parse coverage"; exit 2; }
if [ "${1:-}" = "--bump" ]; then
  echo "$current" > "$FLOOR_FILE"; echo "floor bumped to $current%"; exit 0
fi
floor=$(cat "$FLOOR_FILE")
ok=$(echo "$current >= $floor - $EPSILON" | bc -l)
echo "coverage: ${current}% (floor: ${floor}%)"
[ "$ok" = "1" ] || { echo "COVERAGE DROP: ${current}% < floor ${floor}% - ${EPSILON}"; exit 1; }
```

(Column position of the TOTAL line percentage varies by llvm-cov version — implementer verifies against actual output and adjusts the awk; that's part of Step 2's red→green.)

- [x] Steps: install llvm-cov if absent; run `--bump` once to record the true baseline; demonstrate the failure path; commit `.coverage-floor` with the real number. Commit: `"feat(test): coverage ratchet script + baseline floor"`

---

### Task 12: ci.yml

**Acceptance Criteria (observable):**
- `.github/workflows/ci.yml` exists with jobs `lint`, `test` (pgvector service container + `make test` + env), `coverage` (ratchet), `known-broken` (`continue-on-error: true`).
- Pushed to the feature branch, `gh run watch` shows lint/test/coverage all SUCCESS and known-broken completing (red contents allowed) — the workflow proves itself on its own PR.
- `cargo clippy --workspace --all-targets -- -D warnings` passes locally (pre-existing warnings fixed in this task, incl. `McpError::Corpus` dead code).

**Verify via:** cli

**Files:**
- Create: `.github/workflows/ci.yml`
- Modify: whatever clippy flags (small, mechanical fixes; each `#[allow]` needs a reason comment)

- [x] **Step 1: the workflow:**

```yaml
name: CI
on:
  pull_request:
  push:
    branches: [main]
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: "rustfmt, clippy" }
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
  test:
    runs-on: ubuntu-latest
    services:
      pgvector:
        image: pgvector/pgvector:pg17
        env: { POSTGRES_DB: mx_rag, POSTGRES_HOST_AUTH_METHOD: trust }
        ports: ["55433:5432"]
        options: >-
          --health-cmd "pg_isready -U postgres" --health-interval 5s
          --health-timeout 5s --health-retries 10
    env:
      MX_RAG_TEST_DATABASE_URL: postgres://postgres@localhost:55433/mx_rag
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@v2
        with: { tool: cargo-nextest }
      - run: make test
  coverage:
    runs-on: ubuntu-latest
    services:
      pgvector:
        image: pgvector/pgvector:pg17
        env: { POSTGRES_DB: mx_rag, POSTGRES_HOST_AUTH_METHOD: trust }
        ports: ["55433:5432"]
        options: >-
          --health-cmd "pg_isready -U postgres" --health-interval 5s
          --health-timeout 5s --health-retries 10
    env:
      MX_RAG_TEST_DATABASE_URL: postgres://postgres@localhost:55433/mx_rag
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: "llvm-tools-preview" }
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@v2
        with: { tool: "cargo-nextest,cargo-llvm-cov" }
      - run: make coverage
  known-broken:
    runs-on: ubuntu-latest
    continue-on-error: true
    services:
      pgvector:
        image: pgvector/pgvector:pg17
        env: { POSTGRES_DB: mx_rag, POSTGRES_HOST_AUTH_METHOD: trust }
        ports: ["55433:5432"]
        options: >-
          --health-cmd "pg_isready -U postgres" --health-interval 5s
          --health-timeout 5s --health-retries 10
    env:
      MX_RAG_TEST_DATABASE_URL: postgres://postgres@localhost:55433/mx_rag
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@v2
        with: { tool: cargo-nextest }
      - name: Known-broken scoreboard
        run: |
          make test-known-broken | tee kb.log || true
          echo "== SCOREBOARD =="
          grep -E "PASS|FAIL" kb.log | sort | uniq -c || true
          echo "Passing known-broken tests should be un-ignored and their bd issues closed."
```

- [x] Steps: clippy cleanup first (own commit ok), workflow, push, `gh run watch` until green. Commit(s): `"fix: clippy -D warnings cleanup"` + `"ci: lint/test/coverage/known-broken gate"`

---

### Task 13: release.yml test gate

**Acceptance Criteria (observable):**
- `release.yml` contains a `test` job (checkout, toolchain, nextest, pgvector service, `make test` + clippy/fmt) and every build/publish job's `needs:` includes it (transitively — the first job in each chain).
- `gh workflow run release.yml -f version=0.0.0-testgate` (or an act dry-run if dispatch is risky) demonstrates the test job runs BEFORE build jobs; cancel the run after the ordering is proven — do NOT let a test release publish.
- `actionlint` (or `gh workflow view` parse) reports no syntax errors.

**Verify via:** cli

**Files:**
- Modify: `.github/workflows/release.yml`

- [x] Steps: READ the full existing workflow first (jobs + needs graph), insert the `test` job mirroring ci.yml's test+lint (no known-broken/coverage), rewire `needs:`. For verification prefer `actionlint` + a dispatch run cancelled immediately after the test job starts the dependency chain (`gh run cancel`); note in the commit that no artifact was published. Commit: `"ci(release): gate tag builds on the test suite"`

---

### Task 14: e2e.yml + make test-e2e

**Acceptance Criteria (observable):**
- `make test-e2e` locally: builds mx, runs the rust-api scaffold smoke (testbed) end-to-end — scaffold, `mx add`, `make dev`, router up, `curl --resolve <svc>.localhost:80:127.0.0.1 http://<svc>.localhost` returns 2xx/3xx — then tears down. Exit 0. (`E2E_RECIPES` env selects recipes; local default `rust-api`.)
- `.github/workflows/e2e.yml` exists with `on: workflow_dispatch` only, running the same make target with `E2E_RECIPES="rust-api laravel"` on ubuntu (docker preinstalled, runner owns port 80); one dispatched run shown completing via `gh run watch` (success required — laravel included per spec).

**Verify via:** cli

**Files:**
- Create: `.github/workflows/e2e.yml`
- Create: `scripts/test-e2e.sh` (wraps/cleans up the existing testbed scripts: temp workspace, PATH to built binaries, router up/down, per-step logging, trap-based teardown)
- Modify: `Makefile` (`test-e2e: ; ./scripts/test-e2e.sh`)
- Modify (likely): `tests/testbed/*.sh` — parameterize hardcoded paths if any; keep changes minimal

- [x] Steps: read `tests/testbed/testbed.sh` + the rust-api smoke first; wrap rather than rewrite; local green run is the hard part (router owns port 80 — the script must detect an already-running mx-router and REUSE it locally, only installing on CI); e2e profile in nextest unused here (shell script), retries = one scripted re-curl loop (30s). Commit: `"test(e2e): scaffold->router->URL smoke, local + dispatch workflow"`

---

### Task 15: mutants.yml + make test-mutants

**Acceptance Criteria (observable):**
- `make test-mutants` runs `cargo mutants --package mx-lib --timeout 300 --in-place false` (bounded) and produces `mutants.out/` with a summary; command exits 0 even with missed mutants (report, not gate) — demonstrated on a SMALL scope first (`--file src/corpus/chunk.rs`) to keep the proof fast.
- `.github/workflows/mutants.yml` exists: `workflow_dispatch` + `schedule: cron '0 6 * * 6'`, runs the make target, uploads `mutants.out` as an artifact, never fails the workflow on missed mutants.

**Verify via:** cli

**Files:**
- Create: `.github/workflows/mutants.yml`
- Modify: `Makefile` (define `test-mutants`)
- Create: `.cargo/mutants.toml` (exclude test_support + generated code from mutation)

- [x] Steps standard; local proof on the chunker file only (minutes, not hours); full-package left to the scheduled job. Commit: `"test(mutants): scheduled mutation testing on mx-lib"`

---

### Task 16: Prove the gates + finalize

**Acceptance Criteria (observable):**
- A scratch branch with a deliberately broken unit test shows ci.yml's `test` job FAILING; a scratch branch with a clippy warning shows `lint` FAILING; a scratch branch with `.coverage-floor` set 5 points high shows `coverage` FAILING. All three scratch branches deleted after evidence capture (`gh run view` URLs recorded in the PR body).
- The feature branch's final CI run: lint/test/coverage SUCCESS, known-broken completing with the expected red scoreboard (~13 failing, Task 7's + others; count recorded).
- `tests/KNOWN_BROKEN.md` counts match the lane's actual test count; `make test` green locally; `cargo nextest run --workspace -E 'ignored()'`... (verification permutations documented in the PR body).

**Verify via:** cli

- [x] Steps: three scratch-branch demonstrations (commit → push → `gh run watch` → capture URL → delete branch), final green run, then open the PR (base main) with the evidence table. Do NOT merge — the user merges. Commit: `"docs(test): gate-proof evidence + KNOWN_BROKEN index"`
