---
title: "mech-crate (mx): AI-native meta-framework for Docker project ecosystems (Repo Profile)"
category: repos
languages: [rust, shell, typescript, markdown]
complexity: intermediate
use_cases:
  - "understanding what mech-crate (mx) does and where each of its subsystems lives"
  - "finding mx's CLI, MCP-tool, recipe or corpus surface before extending it"
  - "answering 'which repo owns the mx router, the techniques corpus, or mechcrate.dev'"
  - "resuming work on mech-crate in a fresh session"
summary: "mech-crate is the repository behind `mx` — a Rust CLI, a 47-tool MCP server, and a body of project templates that impose one folder contract on every Docker-based project so humans and agents both land in familiar terrain. It ships: the `mx` binary (25 subcommands: scaffold, service ops, a global Traefik router, infra credentials, docs compilation, self-update); `mx-mcp`, a stdio JSON-RPC MCP server exposing scaffold/operate/understand/retrieve tool families; seven local recipes plus an optional Unyform remote-blueprint client; the techniques corpus itself (68 markdown docs under docs/development, chunked and embedded into pgvector on Neon and queried via `mx rag` / `rag_context`); a self-growing research subsystem (a weekly cron that runs the technique-research skill and opens a PR); and mechcrate.dev, an Astro/Starlight static site that publishes the corpus. Rust workspace of three crates (~18k lines), a vestigial 6.4k-line bash layer, ~665 commits since 2026-01-07, public under MIT OR Apache-2.0, active."
provenance: researched
researched: 2026-09-02
publish: false
repo: https://github.com/Dev916/mech-crate
local_path: ~/dev/dev916/mech-crate
status: active
visibility: public
owner: PriceLove LLC (Dev916)
sources:
  - README.md
  - AGENTS.md
  - CLAUDE.md
  - Cargo.toml
  - Makefile
  - make/docs.mk
  - install.sh
  - crates/mx-cli/src/main.rs
  - crates/mx-cli/src/commands/
  - crates/mx-lib/src/lib.rs
  - crates/mx-lib/src/corpus/
  - crates/mx-lib/src/recipe/
  - crates/mx-lib/src/router/mod.rs
  - crates/mx-lib/src/template/placeholder.rs
  - crates/mx-lib/src/project.rs
  - crates/mx-lib/src/paths.rs
  - crates/mx-lib/src/config.rs
  - crates/mx-lib/src/unyform/mod.rs
  - crates/mx-mcp-server/src/tools/mod.rs
  - crates/mx-mcp-server/src/mcp/
  - bin/lib/
  - templates/
  - scripts/install.sh
  - scripts/package.sh
  - scripts/research-weekly.sh
  - scripts/coverage-ratchet.sh
  - scripts/test-e2e.sh
  - .github/workflows/
  - site/apps/site/
  - skills/
  - docs/development/INDEX.md
  - docs/development/RESEARCH_BACKLOG.md
  - docs/development/RESEARCH_LOG.md
  - docs/superpowers/specs/
  - tests/KNOWN_BROKEN.md
  - .beads/issues.jsonl
---

# mech-crate (mx)

> **"Consistency is the highest-leverage investment in the agent era."** mech-crate
> is the repository behind `mx`, a scaffolding-and-operations kit that gives every
> Docker-based project the same folder contract, the same `make dev`, the same
> hostname-routed URLs — so a developer who knows one mx project knows all of them,
> and an agent stops re-deriving project shape every session. It is four things in
> one tree: a Rust CLI plus MCP server, a body of production-shaped project
> templates and recipes, the *techniques corpus* those agents retrieve from, and
> the site that publishes it. This profile is the umbrella; the five-plus existing
> `mx-*` technique docs in this same corpus hold the depth (see Notable
> Techniques). Repo is public, active, and pushed the day this profile was written.

## Identity
| Field | Value |
|---|---|
| Repository | `Dev916/mech-crate` (public) — default branch `main` |
| Local path | `~/dev/dev916/mech-crate` (directory name matches the repo name) |
| Owner / org | PriceLove LLC (Dev916) — GitHub description: "MechCrate is the project scaffolding kit for Docker-based development" |
| Status | active — last commit 2026-09-01, 664 commits, profiled at `004624a` (GitHub `pushed_at` 2026-09-02) |
| Languages (by file count) | outside `templates/`: Markdown 141 · Rust 69 · Shell 49 · TypeScript 35 · SVG 26 · Make 13 · Astro 13. Inside `templates/` (596 of 1,015 tracked files): Vue 165 · Rust 65 · PHP 55 · TypeScript 50 · YAML 42 — recipe payloads, not first-party code |
| Build system | Cargo workspace (3 crates, `resolver = "2"`, version `0.1.1`) + GNU Make (`Makefile`, one optional include `make/docs.mk`) + npm for `site/apps/site` and `scripts/docs` |
| Runtime deps | Docker + Compose (services, router, test DB); Postgres with pgvector for `mx rag`; an OpenAI-compatible `/embeddings` endpoint; Node 22 for the site and `mx docs`; optional `pandoc`/XeLaTeX for PDF output; `bd` (beads, Dolt-backed) for issue tracking |
| License | MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`; GitHub reports Apache-2.0) |
| CI / release | 5 GitHub Actions workflows (`ci`, `e2e`, `mutants`, `release`, `site`). Zero git tags, zero releases — the release pipeline has never fired |

## What It Does

**The problem.** Real ecosystems have ten-plus services and no two repositories are
shaped alike: each has its own layout, its own env-loading order, its own
half-documented way to start the stack. Humans burn onboarding time; agents burn
tokens re-exploring, every session. And two ecosystems on one laptop fight over
ports 80, 5432 and 6379 (`README.md`).

**The answer, in four moves.** (1) A non-negotiable folder contract — `Makefile`,
`make/`, `scripts/`, `apps/`, `docker/{.config,compose,system,dockerfiles}` — that
`mx new` writes and `ProjectDetector` recognizes (`crates/mx-lib/src/project.rs`).
(2) One global Traefik instance, the *mx router*, on the shared Docker network
`devmesh-traefik`; application services publish no HTTP host ports and are reached
at stable hostnames like `api.localhost`, so ecosystems coexist and switching
context is `cd` (`crates/mx-lib/src/router/mod.rs`, `templates/router/`).
(3) *Recipes* — production-shaped service definitions that carry the accumulated
decisions of the stacks they came from, applied in one `mx add`
(`templates/recipes/`). (4) An agent layer: an MCP server that understands any mx
project, plus a RAG corpus of curated engineering docs stored in a Postgres the
owner controls (`crates/mx-mcp-server/`, `crates/mx-lib/src/corpus/`).

**Who uses it.** Humans through `mx` and the per-project `make` verbs; agents
through the `mx-mcp` server (this machine has it wired as the `mx` MCP server);
and the repo itself, which dogfoods every part — `site/` is an mx project
scaffolded from the astro recipe, and the corpus that agents query *is* this
repo's `docs/development/`. "Done" for a user is: `mx router install && mx router
up` once per machine, then `mx new` → `mx add` → `make dev` → a working URL.

## Capabilities

### CLI — `mx` (25 top-level subcommands, clap-derive)
- `init` · `new` · `add` — install templates to `~/.mech-crate`, scaffold a project, add a service from a recipe (`crates/mx-cli/src/commands/{init,new,add}.rs`)
- `recipes list|ls|info|pull|apply|versions|cache` — local recipe catalogue plus the Unyform remote arm (`crates/mx-cli/src/commands/recipes.rs`)
- `dev` · `up` · `down` · `logs` · `restart` · `sh` · `ps` · `build` — service verbs that shell out to the *project's* Makefile with `s=<service>` / `f=1` (`crates/mx-cli/src/commands/{dev,build}.rs`)
- `router install|up|start|down|stop|restart|status|ps|logs|inspect|info|network|uninstall|remove` — global Traefik lifecycle, dashboard port auto-allocated in 7680–7799 (`crates/mx-cli/src/commands/router.rs`)
- `infra setup|list|ls|inspect|link|unlink|remove` — credential config for cloudflare / digitalocean / aws / hetzner with project→linked→global resolution (`crates/mx-cli/src/commands/infra.rs`, `crates/mx-lib/src/infra/config.rs`)
- `rag ingest|status|gaps` — corpus ingestion (with `--dry-run`, `--clear`, `--force`, `--reembed`), backend/count/model report, and weak-query gap mining (`crates/mx-cli/src/commands/rag.rs`)
- `mcp build|status|ps|config|run|info|test` — MCP server lifecycle and client config emission (`crates/mx-cli/src/commands/mcp.rs`)
- `docs` — compile project Markdown to PDF/HTML through a Node/tsx pipeline (`crates/mx-cli/src/commands/docs.rs`, `scripts/docs/compile.ts`)
- `doctor` · `upgrade` (project scaffolding, `--diff`/`--dry-run`) · `self-update` (rebuilds the CLI from source and re-asserts `/usr/local/bin` symlinks) (`crates/mx-cli/src/commands/{doctor,upgrade,self_update}.rs`)
- `login` · `logout` · `whoami` · `unyform login|logout|whoami` — Unyform auth only; there is **no** `mx unyform recipes …` (`crates/mx-cli/src/commands/unyform.rs`)
- `cc-plugin install|uninstall|status|session|stop` — installs Unyform SessionStart/Stop hooks into the Claude Code settings file, resolves blueprints from the Unyform SaaS into a system-reminder block, and reports per-session token accounting; both hook handlers fail soft so a misconfigured gateway never breaks the host (`crates/mx-cli/src/commands/cc_plugin.rs`)

### MCP tools — `mx-mcp` (47 registered tools, stdio JSON-RPC, protocol `2024-11-05`)
All defined in `crates/mx-mcp-server/src/tools/mod.rs`; transport in `crates/mx-mcp-server/src/mcp/{transport,protocol,server}.rs`.
- **Scaffold (7)** — `mx_new`, `mx_add_service`, `mx_recipes_list`, `mx_recipe_info`, `mx_upgrade`, `mx_build`, `mx_help`
- **Operate (14)** — `mx_router_{install,up,down,status,inspect}`, `mx_infra_{setup,list,link}`, `make_{dev,up,down,logs,restart,shell,ps,help,key}`, `mx_doctor`
- **Understand (4)** — `project_analyze`, `project_list`, `project_detect`, `service_info`
- **Retrieve (8)** — `rag_context`, `rag_search`, `rag_search_category`, `rag_find_implementation`, `rag_get_guidance`, `rag_compare_approaches`, `rag_find_related`, `rag_health`
- **Docs (2)** — `mx_docs_compile`, `mx_docs_list`
- **Unyform (8)** — `unyform_login`, `unyform_logout`, `unyform_whoami`, `unyform_recipes_{list,pull,apply,versions,cache}` — the only place the `unyform recipes` verbs exist
- Server root detection walks cwd / `~/dev/mech-crate` / `~/.mech-crate` looking for `bin/mx`; `--no-rag` disables the corpus arm (`crates/mx-mcp-server/src/mcp/server.rs`)

### Templates, recipes and the router
- 7 recipes, each a directory with a `recipe.json` manifest plus payload: `laravel` (308 files), `astro` (48), `rust-leptos` (39), `rust-worker` (36), `zola` (32), `rust-api` (25), `nuxt` (19) (`templates/recipes/`)
- Manifest schema — `name`, `title`, `version`, `features`, `services`, `options`, `placeholders` (`source` + `transform` ∈ slug/upper/rust_crate/ssr_bool), `init_app`, `directories`, `templates` (with a `common://` namespace for shared payload), `post_install` (create_files/rename/chmod/gitkeep/run/gitignore), `next_steps`, `notes` (`crates/mx-lib/src/recipe/parser.rs`)
- A strict key-allowlist validator catches fields serde would silently drop, enforced across every shipped recipe by a conformance test (`crates/mx-lib/src/recipe/validate.rs`, `crates/mx-lib/tests/recipes_conformance.rs`)
- Substitution is a deliberately narrow custom parser, not a general template engine: only known bare identifiers inside doubled curly braces are expanded, Tera whitespace-control markers honoured, everything else (filters, statement blocks, unknown names) copied through byte-for-byte — written after Tera ate Blade/Vue/Zola sources (`crates/mx-lib/src/template/placeholder.rs`)
- The router ships as four files copied verbatim (no interpolation) into `~/.mech-crate/router/`: Traefik v3.6.1 compose bound to 80/443, static config with docker + file providers, dynamic middlewares, and an acme.json seed chmod'd 0600 (`templates/router/`, `crates/mx-lib/src/router/mod.rs`)
- Per-project scaffolding: `templates/Makefile.template` (wildcard-includes `make/*.mk`), 13 make modules, 29 scripts including an `md2pdf/` toolchain, 19 generic docker files, a Cloudflare Wrangler worker under `templates/infra/cloudflare/` (`templates/`)

### Techniques corpus (`mx rag`)
- 68 markdown docs, flat, in `docs/development/`; canonical frontmatter schema (title/category/languages/complexity/use_cases/summary + optional provenance/researched/sources) is defined at the bottom of `docs/development/INDEX.md`
- Heading-aware chunker splits on `##` and prefixes every chunk with its `Doc Title > Heading` path so chunks self-contextualize when retrieved alone; oversize sections sub-split on paragraph boundaries, cap 1200 chars (`crates/mx-lib/src/corpus/chunk.rs`)
- Ingest walks `*.md` recursively, skips `INDEX.md` by filename, hashes each doc with SHA-256 for idempotency, and never aborts on one bad doc — malformed frontmatter degrades to path heuristics plus a warning (`crates/mx-lib/src/corpus/{ingest,frontmatter}.rs`)
- Store: pgvector on Neon with a local-Postgres fallback (5 s connect timeout), migrations run on connect; hybrid ranking is `0.85 × (1 − cosine) + 0.15 × pg_trgm similarity`, degrading to trigram-only when no embedder is configured; queries are logged and mined by `gaps()` into research themes (`crates/mx-lib/src/corpus/store.rs`)
- Live state at profiling time (`rag_health`): backend `neon`, 66 docs / 2,148 chunks, model `text-embedding-3-small`, 221 logged queries, last ingest 2026-08-18

### Skills (`skills/`, 5 files)
- `skills/techniques/SKILL.md` — the consume side: query the corpus before choosing an approach; appends discovered gaps to `docs/development/RESEARCH_BACKLOG.md`
- `skills/technique-research/SKILL.md` + `references/source-providers.md` — the produce side: 6 phases (locate/pick → assess coverage → research via providers → author → verify with `mx rag ingest --dry-run` at 0 warnings → ship a PR); autonomous topic ladder is backlog → stalest doc → `mx rag gaps` → tech-radar sweep; verdicts NEW / IMPROVE / FRESH; 3 active providers (web, x, hackernews) and 6 planned
- `skills/writing-devloop-plans/SKILL.md` — plan format with per-task Acceptance Criteria
- `skills/devloop/subagent-prompt.md` — a **one-file overlay** of the `nyvorin/devloop` repo (commit `b293e9c`, "devloop subagents consult techniques corpus per task"); no SKILL.md, no references — it points at `~/.claude/skills/devloop/` for both
- A `repo-profile` skill (this document's procedure) exists in the working checkout but is not yet committed on `main`

### Site — mechcrate.dev (`site/`)
- Astro 5 + Starlight 0.37 + Pagefind, static output, no adapter; the Astro app is nested at `site/apps/site` because `site/` itself is an mx project scaffolded from the astro recipe (`site/apps/site/astro.config.mjs`, `site/Makefile`)
- **Flat-scan contract**: the corpus loader reads only top-level `*.md` in `docs/development` (non-recursive) plus a three-entry allowlist of root guides; `publish: false` — and only an explicit boolean false — holds a doc back; unparseable frontmatter, route collisions, dangling `.md` links and secret-lint hits are all hard build failures (`site/apps/site/src/loaders/lib/{sources,frontmatter,pipeline,links,secrets}.ts`)
- 110 prerendered pages: 26 authored Starlight docs, 65 published corpus docs + 3 root guides, per-category indexes, a landing page, `/llms.txt` and `/llms-full.txt`, and a prerendered `/api/health`
- Mermaid diagrams are rendered to committed light/dark SVGs by `site/apps/site/scripts/render-diagrams.mjs`, with a `diagrams:check` drift gate, so `astro build` never launches a browser
- Deployed as a Cloudflare **Workers static-assets** bundle (not Pages) via `wrangler deploy` (`site/apps/site/wrangler.jsonc`, `.github/workflows/site.yml`)

### Research automation
- `scripts/research-weekly.sh` — a *user crontab* entry (`3 9 * * 1`, Mondays 09:03), not a GitHub workflow: `timeout 7200 claude -p "…technique-research… autonomous mode"` with an explicit `--allowedTools` allowlist and deliberately **no** skip-permissions flag, because the run ingests untrusted web content; output is PR-gated, log at `~/.mech-crate/research-cron.log`
- Audit trail: `docs/development/RESEARCH_LOG.md` (11 rows, 2026-07-18 → 2026-08-14; verdicts NEW ×8, FRESH ×2, IMPROVE+FRESH ×1) and `docs/development/RESEARCH_BACKLOG.md` (16 entries, 14 open)

### Bash layer (`bin/lib/`, 17 files, ~6,380 lines) — vestigial
The pre-Rust implementation of the entire CLI, still tracked and still shipped in release tarballs, but **nothing sources it at runtime**: no file under `crates/` references `bin/lib`. Largest members are `recipe.sh` (1,137 lines, the original unified recipe engine), `infra.sh` (930), `upgrade.sh` (568), `mcp.sh` (557, still carrying a Weaviate RAG backend that the Rust path replaced with pgvector), `unyform.sh` (471), `new.sh` (476), `router.sh` (327).

### Not (yet) implemented
- `mx cf` — documented but nonexistent (`bd:mech-crate-vxq`, `tests/KNOWN_BROKEN.md`)
- `mx router install --force` — printed in the "use `--force` to reinstall" hint but not a flag (`crates/mx-cli/src/commands/router.rs`)
- `mx router inspect` is an alias of `status`; SIGHUP `reload` survives only in `bin/lib/router.sh`
- `recipe.json` `templates[].condition` is parsed and never evaluated (`crates/mx-lib/src/recipe/installer.rs`)
- `mx recipes apply --fix` accepts and discards the flag; remote "apply" writes cursor rules from `recipe.patterns` and scaffolds nothing (`bd:mech-crate-9be`)
- Linux release builds, the Homebrew tap-bump job, and a draft→publish release step are TODOs at the bottom of `.github/workflows/release.yml`
- `mx-ingest` — a third binary referenced by the release workflow, `scripts/install.sh`, `scripts/package.sh`, `Makefile` and `self_update.rs`, but dropped from the workspace (commit `f9b1e15`); no `[[bin]]` target defines it

## Architecture

**Stack.** Rust 2021, Cargo workspace at `0.1.1`, tokio full runtime, clap 4.5 derive, sqlx 0.8 + pgvector 0.4.1, tera 1.19 (retained only for binary-file detection), serde/serde_yaml/toml, reqwest 0.12, tracing. Release profile is `lto = true`, `codegen-units = 1`, `strip`, `panic = "abort"`. TypeScript/Astro 5 for the site on Node 22; Bash for the legacy layer, the testbed and CI helpers.

**Component map.** `mx-lib` (8,085 lines) is the shared core — `project`, `paths`, `config`, `env`, `docker`, `recipe/{parser,validate,installer}`, `template/{placeholder,engine}`, `router`, `infra`, `upgrade`, `unyform`, `mcp`, `corpus/{chunk,frontmatter,ingest,embed,store,config}`, plus a `test-support` feature gating shared fixtures (stub-bin PATH, project scaffolder, embedding server) so neither wiremock nor tempfile links into a release binary. `mx-cli` (6,755 lines) is one module per subcommand over that core. `mx-mcp-server` (5,351 lines) has its own JSON-RPC/stdio stack and — notably — its *own* Unyform client rather than reusing `mx-lib`'s.

**Data flow.**

```
mx new ──▶ templates/ ──(placeholder expand)──▶ project skeleton
mx add ──▶ recipe.json ──(parse ▸ validate ▸ install)──▶ apps/ + docker/compose/<svc>.yml
                                                              │ traefik labels
make dev ──▶ docker compose ──▶ container ──┐                 ▼
                                            └──▶ mx-router (Traefik, net devmesh-traefik)
                                                       └──▶ http://<svc>.localhost

docs/development/*.md ──▶ chunk (## split, Title > Heading prefix)
                            ├──▶ embed (OpenAI-compatible /embeddings)
                            │      └──▶ pgvector on Neon ──▶ rag_context / rag_search
                            └──▶ site loader (flat scan, publish gate) ──▶ mechcrate.dev
```

**Storage.** `~/.mech-crate/` is the single machine-level home: `templates/`, `recipes/` (Unyform cache, org/name/version with a `latest` symlink), `router/` (installed compose + `.dashboard-port`), `mcp/` (server state + wrapper), `config/` (`rag.toml`, `unyform/{credentials,session}.json` at 0600, `infra/<provider>.env`, `source-root`). Corpus rows live in Postgres (`technique_docs` / chunks with SHA-256 idempotency). Issue state lives in `.beads/` on a Dolt backend.

**External integrations.** Docker/Compose and Traefik; Neon Postgres with pgvector; any OpenAI-compatible embeddings endpoint (OpenAI by default, Ollama/LM Studio by changing one config key); the Unyform SaaS API (`/v1/auth/me`, `/v1/orgs/<org>/recipes[…]`); Cloudflare Workers for the site and the Wrangler container-worker template; GitHub Actions; Apple notarization services in the release job.

**Process / concurrency model.** Everything is a short-lived process. The CLI is `#[tokio::main]` but linear; the MCP server runs a stdio reader task, a writer task and bounded 100-slot mpsc channels between them. The one background task in the library is a spawned query-log insert in the corpus store, with an explicit `flush_query_log()` so tests can await it. Concurrency lives in Docker, not in mx.

**Security model.** No server, no inbound network surface except the local Traefik. Credentials are per-provider files under `~/.mech-crate/config/` written 0600; nothing is committed — `.env.secrets` and `.env*.local` are gitignored, and the site build runs a secret lint over both body and frontmatter that fails the build on a hit. Config is name-addressed: `MECH_CRATE_ROOT`, `MX_RAG_DATABASE_URL`, `MX_RAG_FALLBACK_DATABASE_URL`, `MX_RAG_EMBEDDING_BASE_URL`, `MX_RAG_EMBEDDING_MODEL`, `MX_RAG_EMBEDDING_API_KEY` (or `OPENAI_API_KEY`), `MX_RAG_TEST_DATABASE_URL`, `MX_ROUTER_DASHBOARD_PORT`, `MECHCRATE_REPO_ROOT` (site container), plus CI secrets `CLOUDFLARE_API_KEY`, `CLOUDFLARE_ACCOUNT_ID`, `APPLE_CERT_P12_BASE64`, `APPLE_CERT_PASSWORD`, `APPLE_API_KEY_BASE64`, `APPLE_API_KEY_ID`, `APPLE_API_ISSUER_ID`, `RELEASES_REPO_TOKEN`. The weekly research cron is the sharpest edge and is handled explicitly: allowlisted tools, no skip-permissions, human-merged PRs.

## Repository Layout

```
crates/
  mx-lib/          shared core: project, recipe, template, router, infra, upgrade, unyform, corpus
  mx-cli/          the `mx` binary — src/main.rs (entry), one module per subcommand
  mx-mcp-server/   the `mx-mcp` binary — src/main.rs (entry), JSON-RPC/stdio + 47-tool registry
bin/
  mx, mx-mcp       prebuilt arm64 Mach-O binaries, committed to the repo
  lib/*.sh         17-file legacy bash CLI (no longer sourced by anything)
templates/         596 files: recipes/ (7 stacks + common/), router/ (Traefik), docker/, make/,
                   scripts/, infra/cloudflare/, Makefile.template
docs/
  development/     the techniques corpus (68 flat .md) + INDEX.md, RESEARCH_{BACKLOG,LOG}.md
  superpowers/     specs/ (5) and plans/ (5) — the design record
  unyform/         Unyform product/GTM material (whitepaper, PRD, pricing, pitch)
  router.md, cloudflare.md, docs-command.md, architecture-review-*.md, product-structure.md
site/              nested mx project; the Astro/Starlight site lives at site/apps/site/
skills/            techniques, technique-research, writing-devloop-plans, devloop (overlay)
scripts/           install.sh, coverage-ratchet.sh, test-e2e.sh, research-weekly.sh, docs/ (tsx)
tests/             KNOWN_BROKEN.md + testbed/ (bash recipe harness); Rust tests live in crates/
make/docs.mk       the single Makefile fragment (document compilation targets)
assets/            brand art only — MechCrate and Unyform logos (nothing in code reads it)
.beads/            beads issue tracker (Dolt backend, issues.jsonl, 5 git hooks)
target/, artifacts/, mutants.out/, workbench/   build and scratch output, all gitignored
```

Entry points: `crates/mx-cli/src/main.rs` (the `mx` binary), `crates/mx-mcp-server/src/main.rs` (`mx-mcp`), `Makefile` (developer verbs), `install.sh` (symlink installer), `scripts/install.sh` (the real installer), `site/apps/site/astro.config.mjs`, and the four `skills/*/SKILL.md` files.

## How It Was Built

**Toolchain.** Stable Rust with `cargo-nextest`, `cargo-llvm-cov`, `cargo-mutants` (config in `.cargo/mutants.toml`, `test_tool = "nextest"`); Node 22 for the site and the docs compiler; Docker for the test database, e2e and the router.

**Build / run / test / lint, as they really are** (`Makefile`, 218 lines):
`make build` (debug) · `make build-release` · `make install-local` (→ `~/.local/bin` via `scripts/install.sh --local --skip-build`) · `make init` (installs templates to `~/.mech-crate`) · `make test` (nextest workspace `ci` profile + doc-tests) · `make test-unit` (no DB) · `make test-int` (spins a `pgvector/pgvector:pg17` container and points `MX_RAG_TEST_DATABASE_URL` at it) · `make test-known-broken` (ignored-only, never fails) · `make coverage` (`scripts/coverage-ratchet.sh` against `.coverage-floor`, currently 49.5, `BUMP=1` raises it) · `make test-e2e` · `make test-mutants` · `make test-smoke` (`tests/testbed/testbed.sh`) · `make lint` (clippy `-D warnings`) · `make fmt-check` · `make check` = fmt-check + lint + test. Database-backed tests self-skip when `MX_RAG_TEST_DATABASE_URL` is unset, so a laptop without Docker still runs green.

**Dev loop.** Edit Rust → `make build` → `make check`; for scaffolding changes, `make init` then scaffold into `workbench/` and `make dev`. Project URLs always come from the router (Traefik `Host()` labels), never from a localhost port — that rule is the subject of `docs/development/mx-app-playbook.md`. Work is tracked in beads (`bd ready` / `--claim` / `close`), and `AGENTS.md` + `CLAUDE.md` make that mandatory for agents.

**CI/CD.** `ci.yml` runs lint, test (with a pgvector service), coverage, and a non-blocking `known-broken` job that parses the nextest summary into a job-summary table. `e2e.yml` is dispatch-only because it takes the host's port 80. `mutants.yml` runs Saturdays 06:00 UTC and passes its dispatch input through `env` rather than interpolating it (workflow-injection mitigation). `site.yml` gates on diagram drift, vitest, an offline lychee link check and a served-content-type assertion, then deploys to Cloudflare on main — skipping with a notice, never failing, when the secrets are absent. `release.yml` fires on `v*` tags: a test gate every publish job must list in `needs:`, then a macOS universal build (`lipo`), Developer ID codesign, notarization, `scripts/package.sh` tarball + SHA-256, and `gh release create/upload` into a separate releases repo.

**Distribution, honestly.** The only path that works today is `git clone` + `make install-local`. The root `install.sh` is a thinner variant that symlinks exactly one path, `/usr/local/bin/mx`, into the checkout. `mx self-update` is a *source rebuild*, not a download. There are no tags, no releases, and both distribution repos (`unyform-ai/mech-crate-releases`, `unyform-ai/homebrew-tap`) are empty — verified via `gh api`, which reports "This repository is empty" for each.

**Configuration.** There is no project manifest: no `mx.toml` exists anywhere. A project is identified structurally (`Makefile` + `docker/`, strictly also `make/` + `scripts/`) and its services, compose files and make targets are *discovered* by scanning. The only TOML the codebase parses is `~/.mech-crate/config/rag.toml` (keys: `database_url`, `fallback_database_url`, `embedding_base_url`, `embedding_model`, `embedding_api_key`; precedence env > file > default).

**Provenance.** 664 commits since 2026-01-07, 640 by web-mech, 22 by Nyvorin, 2 by a third contributor; 202 carry a Claude co-author trailer. Cadence: a 437-commit January burst, 103 in February, a quiet spring, then 60 in July (the RAG library and research mode) and 51 in August (the test baseline and the site). Five design specs and five plans in `docs/superpowers/` are the written record: recipe update (2026-04), techniques RAG (2026-07-15), self-growing techniques (2026-07-18), test baseline (2026-08-11), mechcrate.dev (2026-08-20).

## Relationships

- **Depends on (ours):** none at build time — mx is a leaf. At runtime it assumes the Docker network `devmesh-traefik`, the same name as the **devmesh-traefik** repo (checked out locally as `~/dev/dev916/stack`); see `docs/development/repos/devmesh-traefik.md`.
- **Ported from (ours):** the corpus subsystem is an explicit port of the **hq** corpus pattern — `crates/mx-lib/src/corpus/mod.rs` says so in its module doc ("Ported from the hq corpus pattern (~/dev/hq): doc/chunk tables with sha256 idempotency, hybrid (cosine + trigram) search"), and `chunk.rs` names its `pack_paragraphs` a port of hq's `chunk_text`. See `docs/development/repos/hq.md`.
- **Overlays (theirs):** `skills/devloop/subagent-prompt.md` is a one-file overlay of **nyvorin/devloop** (`~/dev/devloop`), adding a mandatory corpus consult; the full skill lives at `~/.claude/skills/devloop/`. See `docs/development/repos/devloop.md`, whose Relationships section documents the four-copy divergence from the other side.
- **Used by:** every mx-scaffolded project (this repo's own `site/` included); the `devloop` skill's docker path, which drives `mx router up`, `mx dev`, `mx ps` and `mx logs` and derives URLs from Traefik labels; and every agent session on this machine that calls `mcp__mx__rag_context`. The `techniques` and `technique-research` skills are wholly about this repo's corpus.
- **Integrates with (external, optional):** **Unyform** (`https://api.unyform.ai`) for organization blueprints and the Claude Code plugin hooks. mx is fully usable without an account — local recipes, the router and the CLI require no login. This repo also *carries* Unyform's product documentation in `docs/unyform/`, which is unusual: the framework repo is the home of a separate product's GTM material.
- **Third-party runtime peers:** Traefik v3.6.1, Neon Postgres + pgvector, Cloudflare Workers, beads (Dolt-backed), Claude Code (the MCP client and the research cron's runner).
- **Canonical-copy note:** `bin/mx` and `bin/mx-mcp` are committed *build outputs* of `crates/`, refreshed by hand (`make upgrade`, or `mx self-update`). The source is canonical; the committed binaries are a convenience copy that can silently lag it.

## Notable Techniques

The corpus already documents mx in depth — **link these rather than re-deriving them**:

- `docs/development/mx-app-playbook.md` — the operating manual for building apps *with* mx: project anatomy, scaffolding flow, migration, and the always-use-the-router URL rule.
- `docs/development/MX_RUST_CLI_AND_MCP_SERVER.md` — workspace architecture, adding a CLI command or an MCP tool, build/run workflow.
- `docs/development/mx-recipes-and-build.md` — full recipe lifecycle (local *and* Unyform), how updates reach consumers, and the three image-build paths with tag semantics.
- `docs/development/mx-cloudflare-deploy.md` — as-implemented infra flow, credential resolution and known traps.
- `docs/development/mx-mcp~usage.md` — the MCP server from a consumer's point of view.
- Also mx-specific: `docs/development/RUST_CLI_DEVELOPMENT.md`, `docs/development/RECIPE_AUTHORING_GUIDE.md`, `docs/development/INFRA_CONFIG.md`, `docs/development/MX_QUICK_REFERENCE.md`, `docs/development/QUICK_REFERENCE.md`. Adjacent corpus work that came *out* of this repo: `docs/development/pgvector-rust-batch-embedding.md`, `docs/development/rag-retrieval-fusion-and-chunking.md`, `docs/development/multi-agent-systems-in-practice.md`, `docs/development/llm-token-cache-efficiency.md`.

Patterns worth extracting that no doc yet covers:

- **The narrow template parser.** A general template engine is the wrong tool for scaffolding source files that *themselves* contain template syntax. Expanding only known identifiers and copying everything else byte-for-byte is the fix, and the code carries the bug ids that forced it (`crates/mx-lib/src/template/placeholder.rs`).
- **Schema allowlist validation over permissive serde.** `#[serde(default)]` silently swallows typos; a hand-written key allowlist that reports JSONPath-ish findings turned two real drifts into test failures (`crates/mx-lib/src/recipe/validate.rs`).
- **The known-broken lane.** Every open defect owns a red test asserting its *fixed* behaviour, `#[ignore]`-reserved to that lane, with a scoreboard that must sum with the gate suite. It converts a bug list into executable specification (`tests/KNOWN_BROKEN.md`).
- **Proving a gate before trusting it.** Each of lint/test/coverage was demonstrated failing on a deliberately broken branch, with the CI run ids recorded in `README.md`.
- **A corpus that feeds itself.** Query logging → `rag gaps` → research backlog → weekly autonomous run → PR → ingest is a closed loop with a human only at the merge.
- **Backlog candidates** (report only; `RESEARCH_BACKLOG.md` is not edited here): *scaffolding template engines and the escaping problem*; *committed build artifacts — when a binary in git is worth its cost*; *self-growing documentation corpora: query-gap mining and the human-merge gate*.

## State, Gaps and Drift

**Maturity.** Version `0.1.1`, no tags, no releases. 189 gate tests passing plus 14 red known-broken tests; coverage floor 49.5% and ratcheting; mutation testing scheduled weekly on `mx-lib`. Documentation is unusually strong — five design specs, five plans, a published site, and a corpus about its own internals.

**Open work.** 66 beads issues, **35 open / 31 closed** (39 tasks, 23 bugs, 4 features; p1 ×18). 10 open GitHub issues on top of that — two trackers, no stated reconciliation. Zero literal TODO/FIXME markers in the Rust sources; deferrals are prose TODOs in `release.yml` and rows in `tests/KNOWN_BROKEN.md`.

**Drift, concrete:**
- **`mx-ingest` is a ghost.** Commit `f9b1e15` dropped it, yet `release.yml` (the `lipo` loop), `scripts/package.sh` (a hard existence check), `scripts/install.sh`, `Makefile` (`upgrade`, `uninstall`) and `self_update.rs` all still name it. `scripts/package.sh` additionally requires a `LICENSE.txt` the repo does not have (it ships `LICENSE-MIT` and `LICENSE-APACHE`), so the packaging step would fail its own asset gate on two counts.
- **The release channel is empty.** Both `unyform-ai/mech-crate-releases` and `unyform-ai/homebrew-tap` return "This repository is empty" from the GitHub API; there are no tags and no releases anywhere. A notarized-tarball pipeline exists and has never run.
- **Committed binaries.** `bin/mx` (~8.6 MB) and `bin/mx-mcp` (~5.3 MB) are arm64 Mach-O executables tracked in git, refreshed by hand ("chore(bin): refresh installed binaries…"). They are macOS-arm64-only and carry no provenance link to the commit that built them.
- **The corpus is behind the repo.** A local `mx rag ingest --dry-run` sees 67 docs / 2,217 chunks; the live Neon store holds 66 docs / 2,148 chunks with `last_ingest` 2026-08-18. Ingestion is manual and has drifted about two weeks.
- **Retrieval is running lexical-only.** `rag_search` through the MCP server announces "lexical-only search (no embedding key configured); results may be weaker", so the 0.85-weighted vector arm is inert in that path even though embeddings exist in the store. The 0.15 trigram arm is separately known-weak: measured 2.18× separation against a ≥5× target (`bd:mech-crate-4jw`).
- **`bin/lib/` is 6,380 lines of dead bash** still shipped in tarballs, with a `scripts/package.sh` comment claiming it is "sourced by mx-mcp" — nothing sources it. `bin/lib/mcp.sh` still manages a Weaviate backend the Rust path replaced.
- **README-vs-code:** the README advertises "40+ tools" (actual: 47) and "60+ curated docs" (actual: 68); it flags `mx upgrade` as mid-repair, which matches `bd:mech-crate-z5i`. `make help` says `make upgrade` reinstalls to `~/.local/bin`; the target actually copies into `bin/`. `site/apps/site/README.md` is still the astro-recipe scaffold README and describes Vue, PrimeVue, Tailwind, Pinia, Drizzle, Postgres and Redis — none of which the site uses.
- **Two Traefiks.** `templates/docker/compose/traefik.yml` is a per-project Traefik on network `traefik` binding 8080; it would fight the global `mx-router` for 80/443 if used.
- **Two Unyform clients.** `crates/mx-lib/src/unyform/` and `crates/mx-mcp-server/src/unyform/` are separate implementations that disagree on the org path segment (id vs slug) — `bd:mech-crate-rnj` holds the red test.
- **`docs/development/repos/` is invisible to the site.** The loader's flat, non-recursive scan means repo profiles are ingested into the corpus but never published — which matches their `publish: false`, but by two independent mechanisms rather than one.

### Synthesis (inferred)

mech-crate's real product is not the CLI — it is *the contract*. `mx` is one of several
things that implement it (the Rust CLI, the dead bash CLI, the MCP server, the
templates' own Makefile), and the fact that all four can implement the same folder
contract is exactly what makes the bet pay: an agent that learns `apps/`, `docker/compose/<svc>.yml`
and `make dev` once has learned every project the org will ever create. Read that way,
the 596 template files are the source of truth and the 18k lines of Rust are an
enforcement mechanism.

The repo's distinguishing move in 2026 was closing the loop between the framework and
the *knowledge* about the framework. Most tooling repos have docs; this one has a
retrieval store, a query log, a gap miner, an autonomous weekly researcher and a
human-merge gate, and the corpus it serves is checked into the same tree that the
tooling is. That is why a repo profile belongs here at all — the corpus is
infrastructure, not documentation.

The consistent failure mode is the **last mile**. Every subsystem is built to a high
standard right up to the point where it must leave the machine: the release pipeline
is signed, notarized and gated — and has never fired into two empty repositories;
embeddings are computed, stored and hybrid-ranked — and the MCP path queries without
a key; `bin/lib/` was superseded — and never deleted; `mx-ingest` was removed — and
never unwired. None of these are hard problems; all of them are bookkeeping that no
gate covers. The known-broken lane is the one place where that bookkeeping *was*
made mechanical, and it is, tellingly, the healthiest part of the repo. The cheapest
high-value work available is to extend that instinct outward: a packaging smoke test
would have caught `mx-ingest` and `LICENSE.txt` on the day they broke, and an ingest
step in CI would keep the corpus from drifting from the repo that defines it.

## Quick Reference
| Task | Command / path |
|---|---|
| Build | `make build` (debug) · `make build-release` |
| Install | `make install-local` (→ `~/.local/bin`); or `./install.sh` to symlink `/usr/local/bin/mx` |
| Initialize templates | `make init` (or `mx init --force`) |
| Tests | `make test` · `make test-unit` (no DB) · `make test-int` (starts pgvector) · `make check` |
| Known-broken lane | `make test-known-broken` — expected all red (`tests/KNOWN_BROKEN.md`) |
| Coverage | `make coverage` (floor in `.coverage-floor`; `BUMP=1` raises it) |
| Lint | `make lint` (clippy `-D warnings`) · `make fmt-check` |
| First project | `mx router install && mx router up`, then `mx new <p>` → `mx add api --recipe rust-api --domain api.localhost` → `make dev` |
| Service URL | Traefik `Host()` label from `docker/compose/<svc>.yml` — never a localhost port |
| Corpus ingest | `mx rag ingest` (add `--dry-run` for a no-DB parse/chunk check; must report 0 warnings) |
| Corpus status | `mx rag status` · `mx rag gaps --days 30 --min-count 2` · MCP `rag_health` |
| MCP server | `mx mcp build` · `mx mcp config` · binary at `bin/mx-mcp` |
| Site | `cd site/apps/site && npm run dev` (4321) · `npm run build` · `npm test` · `npm run diagrams:check` |
| Research run | `scripts/research-weekly.sh` (cron Mondays 09:03); log `~/.mech-crate/research-cron.log` |
| Issue tracker | `bd ready` · `bd show <id>` · `bd update <id> --claim` (`.beads/`) |
| Machine state | `~/.mech-crate/{templates,recipes,router,mcp,config}` |

## Sources

- `README.md` — the framework's own argument, the folder contract, the recipe table, the proven-gates table, and the tool-count and doc-count claims checked against code here.
- `AGENTS.md`, `CLAUDE.md` — beads-mandatory workflow and the non-interactive shell rules agents must follow in this repo.
- `Cargo.toml`, `crates/*/Cargo.toml` — workspace members, dependency set, release profile, feature gating of `test-support`, and the two `[[bin]]` targets (which is how `mx-ingest`'s absence was established).
- `crates/mx-cli/src/main.rs` and `crates/mx-cli/src/commands/` — the 25-subcommand surface and each command's flags; `cc_plugin.rs` for the Claude Code hook subsystem; `self_update.rs` and `upgrade.rs` for the two different "upgrade" meanings.
- `crates/mx-lib/src/` — `project.rs` (structural detection), `paths.rs`/`config.rs` (the `~/.mech-crate` layout), `recipe/` (manifest schema, validator, installer), `template/placeholder.rs` (the narrow parser), `router/mod.rs` (network name, port range, compose project pinning), `unyform/mod.rs`, `upgrade/mod.rs`, `corpus/` (chunker, frontmatter, ingest, store, config).
- `crates/mx-mcp-server/src/tools/mod.rs` — the 47 tool definitions counted and grouped; `src/mcp/{server,transport,protocol}.rs` for the stdio JSON-RPC stack and protocol version.
- `Makefile`, `make/docs.mk`, `install.sh`, `scripts/{install,coverage-ratchet,test-e2e,research-weekly,package}.sh` — the real build/test/install/release commands, the coverage ratchet, the e2e flow, the cron allowlist, and the packaging asset gate.
- `.github/workflows/{ci,e2e,mutants,release,site}.yml` — job graphs, the "every publish job needs the test gate" rule, secret names, and the release TODOs.
- `templates/` — recipe manifests and payloads, the router compose and Traefik config, the generated-project Makefile and make modules.
- `site/apps/site/{package.json,astro.config.mjs,wrangler.jsonc,src/loaders/}` — the site stack, the flat-scan corpus contract, the publish gate and the build-failure rules, and the Workers static-assets deploy.
- `skills/` — the techniques/technique-research loop, the provider registry, and the devloop overlay.
- `docs/development/INDEX.md` (frontmatter schema), `RESEARCH_BACKLOG.md`, `RESEARCH_LOG.md`; `docs/superpowers/specs/` and `plans/` (the design record); `tests/KNOWN_BROKEN.md` (the lane and its scoreboard).
- `.beads/issues.jsonl` — issue counts by status, type and priority, read with `grep -o`; `bd` was not run.
- Live checks: `git log`/`git ls-files`/`git shortlog` in the worktree; `gh api` for repo metadata, tags, releases and the two distribution repos; `bin/mx rag ingest --dry-run` for local doc/chunk counts; `mcp__mx__rag_health` and `mcp__mx__rag_search` for the deployed corpus state and the lexical-only warning.
