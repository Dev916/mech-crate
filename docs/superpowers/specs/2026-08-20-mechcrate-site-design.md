# mechcrate.dev — Site Design Spec

**Date:** 2026-08-20
**Status:** Approved (design approved in-session; spec pending user review)
**Repo:** mech-crate

## Overview

A public home for MechCrate at **mechcrate.dev**: a landing page that tells the "why" (mirroring the launch README's problems narrative), full documentation, the published techniques corpus, and LLM-consumable surfaces (`llms.txt` / `llms-full.txt`). Built with **Astro + Starlight**, scaffolded and operated **as an mx project** (the site itself is the dogfood), deployed to **Cloudflare** via the astro recipe's existing deploy path.

## Decisions (settled during brainstorming)

| Decision | Choice |
|---|---|
| Stack | Astro Starlight, scaffolded as an mx app (astro recipe + Starlight integration); dev via mx router at `http://mechcrate.localhost` |
| Hosting | Cloudflare (Pages/Workers static), domain **mechcrate.dev** (user registers) |
| Location | **In-repo at `site/`** — a complete nested mx project root. Docs stay single-source-of-truth; the same PR updates corpus and site. No submodules/sync machinery. |
| Docs scope | **Publish-by-default** with a `publish: false` frontmatter escape hatch. Curation is the exception, not the rule. |
| IA | README "problems it exists to solve" section is the landing page's information architecture |
| unyform | One low-key page under Framework (remote blueprints, optional) — same posture as the README |

## Architecture

```
mech-crate/
├── docs/                      # single source of truth (existing)
│   ├── development/           # corpus: 67 docs, frontmatter-typed
│   └── *.md                   # framework guides (router, cloudflare, docs-command)
├── site/                      # NEW — nested mx project (own folder contract)
│   ├── Makefile, make/, scripts/, docker/   # mx contract (scaffolded)
│   └── apps/site/             # Astro + Starlight app
│       ├── src/content/docs/  # authored MDX (landing sections, guides, AI layer)
│       ├── src/loaders/corpus.ts  # build-time corpus loader (see Content pipeline)
│       └── astro.config.mjs   # Starlight config, sidebar, Pagefind
└── .github/workflows/site.yml # NEW — build + deploy workflow
```

The `site/` directory is created with `mx new site` + `mx add site --recipe astro --domain mechcrate.localhost`, then Starlight is added as an Astro integration. Nesting a complete mx project inside the repo is safe by construction — the folder contract makes projects self-contained (the e2e testbed already does this in throwaway form).

## Information architecture

**Landing page** (custom Astro page, not a Starlight doc):
1. Hero — positioning line + install one-liner + GitHub link
2. Four problem sections (from the README, expanded with diagrams): subset-of-services · cross-org router · scaffold-with-wisdom · the cobbled-together everything else
3. The AI layer — MCP + corpus + research pipeline, with the loop diagram
4. Quality/transparency strip — proven gates, known-broken lane
5. CTA — install + docs

**Docs sidebar (Starlight groups):**
1. **Start** — install, first project, folder contract, CLI reference (mx verbs + make verbs)
2. **Framework** — router, recipes + authoring, compose & env conventions, infra credentials, upgrade, testing, Cloudflare deploy, remote blueprints (unyform)
3. **AI Layer** — MCP server & tool families, techniques corpus / RAG setup (`rag.toml`, local-endpoint option), research pipeline, agent execution rules

Framework and AI Layer pages are **authored MDX**; where a corpus doc covers the topic (e.g. `INFRA_CONFIG.md`, `instructions.md`), the authored page stays short and cross-links the corpus page rather than duplicating it — each corpus doc renders exactly once, in the Corpus section.
4. **Techniques Corpus** — every publishable `docs/development/` doc, auto-grouped by frontmatter `category` (architecture, concurrency, database, theory, patterns, …). Each page carries a banner: *"This doc ships inside mx's agent corpus — agents retrieve it via `rag_context`."*
5. **Project** — research log, research backlog, known-broken lane (rendered from `tests/KNOWN_BROKEN.md`), license

## Content pipeline

A build-time loader (`src/loaders/corpus.ts`, TypeScript, runs inside Astro's content-collection loader API):

- Reads `docs/development/*.md` plus an explicit allowlist of repo-root guides — `docs/router.md`, `docs/cloudflare.md`, `docs/docs-command.md` — from the repo root (a stable relative path from the nested app).
- **Respects `publish: false`** in frontmatter; skips those docs.
- Maps existing frontmatter (`title`, `category`, `summary`, `complexity`, `use_cases`, `provenance`, `researched`, `sources`) onto Starlight page metadata; renders provenance/researched/sources as a page footer ("Researched 2026-08-14 · 24 sources").
- Sanitizes slugs (e.g. `mx-mcp~usage.md` → `mx-mcp-usage`), rewrites intra-corpus relative links to site routes, and rewrites repo-relative links (e.g. `tests/KNOWN_BROKEN.md`) to GitHub URLs.
- **Secret lint**: build fails if any published doc matches secret patterns (`postgres://[^l]`, `sk-`, `AKIA`, bearer tokens). Cheap insurance for a pipeline that publishes by default.
- Generates **`llms.txt`** (index: site map + one-line summaries per doc, per the llms.txt convention) and **`llms-full.txt`** (concatenated full text of all published guides + corpus docs) at the site root.

**Hold-backs** (`publish: false` added to frontmatter in this build — my calls, revisable):
- `docs/development/INDEX.md` — generated index; the site builds its own navigation
- `docs/development/APPLE_DESIGN_GUIDELINES.md`, `APPLE_DESIGN_QUICK_GUIDE.md` — derivative summaries of Apple's HIG; little value beyond the original and closest to republishing someone else's content

**Never published** (loader scope, not frontmatter): `docs/superpowers/` (specs/plans), `docs/unyform/` (internal integration notes), `docs/architecture-review-2026-03-07.md` and `docs/product-structure.md` (internal point-in-time thinking), `docs/research-source.txt`, and **subdirectories of `docs/development/`** — notably `docs/development/repos/` (internal repo profiles; see `2026-09-01-repo-profiles-corpus-design.md` §3 D3). The flat scan in `sources.ts` is a tested contract (`sources.test.ts`), not an accident.

Everything else publishes — including `RESEARCH_LOG.md`, `RESEARCH_BACKLOG.md`, `sources.md`, and `instructions.md` (agent execution rules are AI-layer content).

## Diagrams (authored in this build, Mermaid in MDX)

1. **Ecosystem topology** — router + N projects, subset running, hostname routing
2. **Folder contract** — annotated tree (landing + Start)
3. **Compose layering** — baseline `service.yml` + `service.dev.yml` override
4. **The AI loop** — agent → MCP → `rag_context` → corpus ← research pipeline ← weekly schedule; PR gate with human merge
5. **Recipe install flow** — `mx add` → placeholders → files landed → router label

## Deploy & CI

- **`site.yml`**: on push to `main` with changes in `site/**` or `docs/**` → install, `astro build`, deploy via wrangler to Cloudflare (the astro recipe's deploy path) → mechcrate.dev. On PRs touching the same paths → build + **preview deploy**, link posted to the PR.
- Secrets: `CLOUDFLARE_API_TOKEN` + `CLOUDFLARE_ACCOUNT_ID` as GitHub Actions secrets (note: bd mech-crate-wd9 tracks CF credential env-var drift in mx itself — the workflow uses wrangler directly and does not depend on that fix).
- The site job is independent of the Rust gates — it never blocks or is blocked by `ci.yml`.
- Local dev: `cd site && make dev` → `http://mechcrate.localhost` through the mx router; `make build s=site` produces the static bundle. The README/site docs both get a line about this ("this site is an mx app — clone and `make dev` it").

## Error handling & policies

- Loader failures (unparseable frontmatter, broken intra-corpus link, secret-lint hit) fail the build loudly with the offending file/line — never publish a partial site silently.
- A corpus doc without `title`/`category` falls back to filename + "uncategorized" and emits a build warning (not a failure) so new docs can't be silently dropped.
- `llms-full.txt` size is unbounded by design (agents want everything); `llms.txt` stays small and indexed.
- Dark/light theme: Starlight default (both), respecting the audience.

## Testing

- **Loader unit tests** (vitest): frontmatter mapping, `publish: false` filtering, slug sanitization, link rewriting, secret-lint patterns, llms.txt generation.
- **Build gate**: `astro build` in `site.yml` on every touching PR — a broken doc breaks the PR, which is correct because docs are the product.
- **Link check**: `lychee` (or Starlight's built-in link validation) over the built site in CI — internal links must resolve.
- E2E (site scaffolding itself) is covered by the existing recipe conformance tests; no new Rust surface.

## Out of scope (v1)

Versioned docs · blog · search beyond Pagefind · analytics · unyform account flows · custom corpus search UI (the RAG is the search UI for agents; Pagefind serves humans).
