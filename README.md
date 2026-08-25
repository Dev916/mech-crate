<p align="center">
  <img src="assets/mechcrate-logo.png" alt="MechCrate Logo" width="200">
</p>

<h1 align="center">MechCrate (mx)</h1>

<p align="center">
  <strong>An AI-native meta-framework for standing up and operating entire service ecosystems — consistently, locally, and with agents as first-class developers.</strong>
</p>

<p align="center">
  <a href="https://github.com/Dev916/mech-crate/actions/workflows/ci.yml"><img src="https://github.com/Dev916/mech-crate/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue" alt="License: MIT OR Apache-2.0">
  <img src="https://img.shields.io/badge/rust-stable-orange" alt="Rust">
</p>

---

Modern development has two audiences: the people on your team, and the AI agents working alongside them. Both suffer from the same disease — **every project is shaped differently**. Every repo has its own layout, its own env-var loading order, its own half-documented way to start the stack. Humans burn onboarding time relearning it; agents burn tokens re-exploring it, on every single session.

MechCrate's bet is that **consistency is the highest-leverage investment you can make in the agent era.** When every project in every organization you touch has the same folder contract, the same `make dev`, the same compose conventions, and the same URLs, then:

- A developer who knows one mx project knows all of them.
- An agent lands in familiar terrain every time — less exploration before the first useful edit, and one MCP server that understands any mx project instead of per-repo bespoke discovery.
- Your organization's accumulated wisdom travels with the framework: an MCP server and a techniques corpus (RAG over curated engineering docs, stored in a Postgres **you** control) mean agents can look up how your stacks are built instead of guessing — with no SaaS middleman between the agent and your stack.

The Docker/ecosystem machinery below is the foundation that makes that possible. It was built the way real developers actually work — and that story is worth telling in full.

## The problems it exists to solve

### You can't run all twelve services locally

Real ecosystems have 10+ services. Running all of them on a laptop — or in a shared dev environment — is not feasible, and most of them are irrelevant to the feature you're building today.

mx makes the *subset* the unit of work. Every service gets its own **atomic compose file** (`docker/compose/<service>.yml` + `<service>.dev.yml` override) — nothing forces all-or-nothing:

```bash
make dev                 # the whole stack
make dev s=api           # just the service you're working on
```

Baseline files describe production shape; dev overrides add hot-reload mounts, debug ports, and relaxed health checks. Nothing about "what runs where" lives in tribal knowledge.

### Switching organizations shouldn't mean tearing down your stack

If you work across multiple organizations — or just multiple products — the usual routine is misery: spin down ecosystem A's containers so ecosystem B's ports don't collide, lose your state, repeat tomorrow.

The **mx router** is a single global Traefik instance that every mx project registers with. Application services publish **no HTTP host ports** — Traefik routes to them by hostname, so two ecosystems' web apps never collide. (Dev overrides do expose backing services like Postgres and Redis on their conventional ports for local GUI tools; that's per-compose-file and easy to change.) Each service is reachable at a stable hostname:

```
http://api.localhost        http://admin.localhost        http://docs.localhost
```

Multiple ecosystems coexist on one machine. Switching contexts is `cd` — not teardown. One router, installed once (`mx router install && mx router up`), serves everything.

### Scaffolding should carry wisdom, not boilerplate

Starting a new service usually means cobbling: copy a Dockerfile from the last project, hand-port the test harness, rediscover the admin-panel setup, re-solve deployment. Piecemeal, every time.

mx **recipes** are production-shaped service definitions that carry the accumulated decisions of the stacks they came from — dependency choices, testing setup, admin tooling, dev overrides, deploy configuration — applied in one motion:

```bash
mx new my-platform            # a complete project skeleton
mx add api --recipe rust-api --domain api.localhost   # a service with its wisdom included
mx upgrade --diff             # review what newer scaffolding would change
mx upgrade                    # adopt it
```

Projects don't fork away from the framework: `mx upgrade` keeps existing projects current with the templates as they improve. (`mx upgrade` is currently mid-repair against the shipped template layout — it's [mech-crate-z5i](tests/KNOWN_BROKEN.md), with a red test asserting the fixed behavior. More on that lane below.)

### The cobbled-together everything else

The concerns every team solves badly, differently, per-project — mx standardizes once:

- **Environment config**: a single documented loading order — `.env.shared` → `.env.secrets` (gitignored) → `.env.<service>`, all under `docker/.config/`.
- **A consistent CLI**: `make dev / up / down / logs / sh / ps / build / doctor` behave identically in every project. (`mx` mirrors the same verbs globally.)
- **Filesystem discipline**: 1:1 host-to-container mounts under `docker/system/` — what you see is what the container sees.
- **Documentation**: `mx docs` compiles project Markdown to PDF/HTML.
- **Health**: `mx doctor` and `make doctor` check dependencies before they ruin your afternoon.

### AI that helps without leaking or wasting

Most "AI-enabled" development bolts a chatbot onto chaos. mx inverts it — make the terrain legible, then arm the agent:

- **An MCP server** (`mx-mcp-server`) exposing 40+ tools: scaffolding (`mx_new`, `mx_add_service`), operations (`make_dev`, `make_logs`, router controls), project analysis (`project_analyze`, `service_info`), and retrieval (`rag_*`).
- **A techniques corpus** — RAG over curated engineering docs on pgvector (vector retrieval; a trigram lexical arm exists and is currently weak — tracked as [mech-crate-4jw](tests/KNOWN_BROKEN.md)). Agents call `rag_context` while planning and implementing, and consult your organization's patterns instead of guessing. The corpus is stored in a Postgres **you** control — a Neon project or a local container — never a third-party AI vendor's store. Embeddings are computed through any OpenAI-compatible `/embeddings` endpoint: OpenAI by default; point `embedding_base_url` at Ollama or LM Studio to keep ingestion and queries fully local.
- **A corpus that grows on schedule** — this repo's own corpus is maintained by a research skill ([`skills/technique-research/`](skills/technique-research/)) run weekly: it researches a topic, corroborates claims against primary sources, and opens a PR with provenance frontmatter and a full source list. A human merges. Audit trail: [RESEARCH_LOG.md](docs/development/RESEARCH_LOG.md).
- **Less exploration by construction** — identical layouts and stable conventions mean agents don't re-learn project shape per session, and one MCP server understands every mx project.

## How it works in 60 seconds

Every mx project honors one **folder contract**:

```
project-root/
├── Makefile                 # the CLI: make dev / up / down / logs / sh ...
├── make/                    # make modules (common.mk, dev.mk, ...)
├── scripts/                 # shell scripts
├── apps/                    # application source code
│   └── <service>/
└── docker/
    ├── .config/             # env files: .env.shared, .env.secrets, .env.<service>
    ├── compose/             # atomic per-service compose: <service>.yml + <service>.dev.yml
    ├── system/              # 1:1 host-to-container mounts
    └── dockerfiles/         # one directory per service
```

That contract is non-negotiable — and it's the point. It's what lets one `make dev` work everywhere, one router serve everything, and one MCP server understand any project.

### Install

```bash
git clone https://github.com/Dev916/mech-crate.git
cd mech-crate
make install-local    # installs mx to ~/.local/bin (no sudo)

mx --version
mx doctor
```

### First project

```bash
mx router install && mx router up   # once per machine

mx new my-app
cd my-app
mx add api --recipe rust-api --domain api.localhost
make doctor                          # check dependencies
make init                            # initialize environment
make dev                             # develop at http://api.localhost
```

## Recipes

| Recipe | Status | What it carries |
|--------|--------|-----------------|
| `astro` | ✅ | Full-stack Astro 5 with Vue 3 islands, SSR, shadcn-vue, PrimeVue, global state, Cloudflare deployment |
| `laravel` | ⚠️ | Laravel 12 + Octane (Swoole) with Filament admin & Inertia.js SSR frontend |
| `nuxt` | ✅ | Nuxt 3 SSR/SSG application with Nitro server, Tailwind CSS |
| `rust-api` | ✅ | Rust API service with Actix-web, SQLx, and hexagonal architecture |
| `rust-leptos` | ✅ | Leptos SSR + Actix-web with shadcn-ui, actor model, PostgreSQL, and Redis |
| `rust-worker` | ⚠️ | High-performance job worker with Redis pub/sub, PostgreSQL, and local LLM evaluation |
| `zola` | ⚠️ | Zola static site generator — single binary, no dependencies |

⚠️ = `mx add` for these recipes currently fails on a known templating defect (the Tera renderer parses `{{ }}` in the recipe's own app sources) — tracked in the issue index with red tests pending; fix in progress.

`mx recipes list` and `mx recipes info <name>` show what's installed; the [Recipe Authoring Guide](docs/development/RECIPE_AUTHORING_GUIDE.md) covers writing your own.

## The AI layer

```bash
mx mcp build      # build the MCP server
mx rag ingest     # ingest docs/development into the techniques corpus
mx rag status     # corpus backend, doc/chunk counts, embedding model
mx rag gaps       # mine weak-scoring queries for research topics
```

`mx rag` needs a Postgres with pgvector (a local container or a Neon project) and an embeddings key (`OPENAI_API_KEY`, or any OpenAI-compatible endpoint via `embedding_base_url` in `~/.mech-crate/config/rag.toml`).

The MCP tools group into four families:

- **Scaffold** — `mx_new`, `mx_add_service`, `mx_recipes_list`, `mx_recipe_info`, `mx_upgrade`
- **Operate** — `make_dev`, `make_up`, `make_down`, `make_logs`, `make_shell`, `mx_router_*`
- **Understand** — `project_analyze`, `project_detect`, `service_info`, `mx_doctor`
- **Retrieve** — `rag_context`, `rag_search`, `rag_find_implementation`, `rag_compare_approaches`, `rag_get_guidance`, `rag_health`, and friends

An agent with these tools can scaffold a service, start the right subset of the stack, tail its logs, and consult the techniques corpus before writing a line — all through one server you run yourself.

## Remote blueprints (optional)

Local recipes are fully self-sufficient — scaffolding, the router, and the CLI require no account. For teams that want organization-connected scaffolding, mx integrates with [unyform](https://unyform.ai): `mx login` links your org, and blueprints generated from your connected repositories become available alongside local recipes. That's the extent of it — an optional cloud source of recipes, not a dependency.

## Quality

The test suite gates every release. The three blocking gates — lint, test, coverage — are **proven, not assumed**: each was demonstrated failing on a deliberately-broken branch before the baseline landed.

| Deliberate break | Job that failed | Evidence |
|---|---|---|
| red unit test | `test` | [run 31819711713](https://github.com/Dev916/mech-crate/actions/runs/31819711713) |
| `clippy::useless_format` warning | `lint` | [run 31819749340](https://github.com/Dev916/mech-crate/actions/runs/31819749340) |
| `.coverage-floor` raised 49.5 → 54.5 | `coverage` | [run 31819764638](https://github.com/Dev916/mech-crate/actions/runs/31819764638) |

- `make test` — the unit + integration gate (cargo-nextest + doc-tests), on every PR and push to main
- `make coverage` — line coverage against a ratcheting floor (currently 49.5%; `BUMP=1` raises it, drops fail CI)
- `make test-known-broken` — a TDD lane (a report, not a gate) where every open, testable defect has a red test asserting its *fixed* behavior; un-ignoring the test is the fix's definition of done ([tests/KNOWN_BROKEN.md](tests/KNOWN_BROKEN.md))
- `make test-e2e` — scaffold → router → live URL with real Docker (dispatched workflow)
- `make test-mutants` — scheduled mutation testing on the core library (weekly report)

A release tag cannot ship unless its commit passes lint, tests, and coverage — the publish jobs depend on them. We'd rather publicly index our own defects than pretend there aren't any: that's what the known-broken lane is.

## Documentation

| Guide | Description |
|-------|-------------|
| [Router Guide](docs/router.md) | Global Traefik reverse proxy setup |
| [Recipe Authoring](docs/development/RECIPE_AUTHORING_GUIDE.md) | Create custom service recipes |
| [Rust CLI Development](docs/development/RUST_CLI_DEVELOPMENT.md) | Develop the mx CLI |
| [Quick Reference](docs/development/QUICK_REFERENCE.md) | Common commands cheatsheet |
| [Known-Broken Lane](tests/KNOWN_BROKEN.md) | Open defects and their red tests |

The `docs/development/` directory also holds the techniques corpus itself — 60+ curated engineering docs spanning architecture, concurrency, databases, security, and process. It's readable as documentation and queryable as RAG.

## Development

```
mech-crate/
├── crates/
│   ├── mx-lib/             # core logic
│   ├── mx-cli/             # the `mx` binary
│   └── mx-mcp-server/      # MCP server for AI agents
├── templates/              # project skeleton, recipes, router
└── docs/                   # documentation + techniques corpus
```

```bash
make build            # debug build
make test             # the gate suite
make lint             # clippy (-D warnings)
make check            # fmt-check + lint + test
make install-local    # install to ~/.local/bin
```

Database-backed tests skip when `MX_RAG_TEST_DATABASE_URL` is unset, so a laptop without Docker still runs green; CI always supplies the container.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

---

<p align="center">🦝 <em>Crate Raccoon says: happy building.</em></p>
