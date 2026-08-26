# mechcrate.dev Site — Implementation Plan

**Spec:** `docs/superpowers/specs/2026-08-20-mechcrate-site-design.md`
**Branch:** `feat/site`
**Verification toolkit:** CLI tools (loader tests + astro build) and web UI (Playwright against the dev server) per task as noted.

Conventions for every task: work stays on `feat/site`; commit per task in repo style; the Rust gates are untouched (site work must not modify `crates/` or root `Makefile` except where a task says so). Site-local commands run from `site/` (mx project) or `site/apps/site` (the Astro app). Node ≥20 assumed locally; CI installs it.

## Phase 1 — Scaffold

### Task 1: Scaffold `site/` as a nested mx project
- [x] From repo root: `mx new site --no-prompt`, then `cd site && mx add site --recipe astro --domain mechcrate.localhost`.
- [x] Known issue (bd: recipe scaffolders never run — skip_if_exists ordering): the astro recipe lands scaffolding but `npm create astro` will NOT have run. Create the app manually in `site/apps/site`: minimal Astro 5 project (`package.json`, `astro.config.mjs`, `src/`, `tsconfig.json`) matching what the recipe's dev override expects (port, mount paths — read `site/docker/compose/site.dev.yml` and `site/docker/dockerfiles/site/` to conform).
- [x] Verify the compose Traefik rule reads ``Host(`mechcrate.localhost`)`` (regression check on the merged mech-crate-290 fix).
- [x] Add `site/apps/site/node_modules`, build output dirs to the repo `.gitignore`.
- **Accept:** `site/` honors the folder contract (Makefile, make/, scripts/, apps/, docker/{compose,.config,system,dockerfiles}); compose file has the correct Host rule; `cd site/apps/site && npm install && npx astro build` exits 0 on the skeleton.

### Task 2: Starlight integration + skeleton IA
- [x] `npx astro add starlight` (or manual integration). Configure `astro.config.mjs`: site `https://mechcrate.dev`, Starlight title "MechCrate", sidebar groups: Start, Framework, AI Layer, Techniques Corpus, Project. Pagefind stays enabled (default).
- [x] Landing route `/` is a custom Astro page (not Starlight docs); docs live under `/docs/…`.
- [x] Placeholder pages so every sidebar group renders.
- **Accept:** `npx astro build` exits 0; `npx astro dev` serves `/` (custom page) and `/docs/start/…` (Starlight shell with all 5 groups).

## Phase 2 — Content pipeline

### Task 3: Corpus loader
- [x] `site/apps/site/src/loaders/corpus.ts` (+ small pure helpers in `src/loaders/lib/`): reads `../../../docs/development/*.md` plus allowlist `docs/router.md`, `docs/cloudflare.md`, `docs-command.md` via Astro content-collection loader API.
- [x] Behavior per spec: skip `publish: false`; map frontmatter (`title,category,summary,complexity,use_cases,provenance,researched,sources`) to page data; sanitize slugs (`mx-mcp~usage` → `mx-mcp-usage`); rewrite intra-corpus `.md` links to site routes and repo-relative links (e.g. `tests/KNOWN_BROKEN.md`) to GitHub URLs; missing title/category → filename + "uncategorized" + build **warning**; unparseable frontmatter or broken intra-corpus link → build **failure** with file/line.
- [x] Secret lint: fail the build if published content matches `postgres://(?!localhost)`, `sk-[A-Za-z0-9]`, `AKIA[0-9A-Z]`, `Bearer [A-Za-z0-9._-]{20,}`.
- [x] Vitest: unit tests for filtering, slug sanitization, link rewriting, secret lint, frontmatter fallback (fixtures under `src/loaders/__fixtures__/`). Add `npm test` script.
- **Accept:** `npm test` green; `npx astro build` renders every publishable corpus doc under `/docs/corpus/<category>/<slug>/`; a fixture with `publish: false` provably absent; a fixture with a fake `sk-…` provably fails the build.

### Task 4: Hold-backs + corpus page chrome
- [x] Add `publish: false` to frontmatter of `docs/development/INDEX.md`, `APPLE_DESIGN_GUIDELINES.md`, `APPLE_DESIGN_QUICK_GUIDE.md` (repo docs — the one task that edits outside `site/`). Run `cargo run -q -p mx-cli -- rag ingest --dry-run` → must stay 0 warnings (frontmatter addition must not break ingest).
- [x] Corpus page template: banner "This doc ships inside mx's agent corpus — agents retrieve it via `rag_context`."; footer rendering provenance/researched/sources when present.
- [x] Corpus category index pages (auto-generated from loader data, grouped by `category`).
- **Accept:** built site shows 64 corpus docs (67 − 3 held back), each with banner; researched docs (e.g. `llm-token-cache-efficiency`) show the provenance footer with source count; ingest dry-run 0 warnings.

## Phase 3 — Authored content

### Task 5: Diagrams
- [x] Five Mermaid diagrams as MDX-embeddable components (Starlight renders mermaid via rehype-mermaid or client component — pick one, document choice): ecosystem topology, folder contract, compose layering, AI loop, recipe install flow.
- **Accept:** each diagram renders in the built site (Playwright: SVG present on its page, no mermaid error text).

### Task 6: Landing page
- [x] Hero (positioning line, install one-liner, GitHub link), four problem sections mirroring the README narratives (each with its diagram where it fits), AI-layer section (AI loop diagram), quality strip (proven gates + known-broken lane, linking to Project pages), footer CTA. Content adapted from README — tightened for web, not pasted.
- [x] Raccoon logo from `assets/mechcrate-logo.png` (copy into site assets); professional tone, mascot as accent (README rules apply).
- **Accept:** Playwright on the dev server: `/` renders hero + 4 problem sections + AI section + quality strip; all internal links resolve; lighthouse-level sanity (no console errors).

### Task 7: Start + Framework docs
- [x] **Start**: install, first project (mirrors README quickstart incl. `--domain`), folder contract, CLI reference (mx verbs + make verbs — source from `mx --help` output, keep honest).
- [x] **Framework**: router (adapted from `docs/router.md`), recipes + status table (mirror README's ✅/⚠️ honesty — after PR #28 all 7 recipes ✅ at apply; verify claim by running `mx add` for each in a temp project before writing it), recipe authoring (link corpus/authoring guide), compose & env conventions, infra credentials (short page cross-linking corpus `INFRA_CONFIG`), upgrade (mirror mech-crate-z5i honesty marker — still open), testing, Cloudflare deploy (from `docs/cloudflare.md`), remote blueprints/unyform (README posture).
- **Accept:** every page renders; no command shown that fails when executed (spot-verify: the recipes status table matches live `mx add` behavior in a scratch project); cross-links to corpus pages resolve.

### Task 8: AI Layer + Project docs
- [x] **AI Layer**: MCP server & 4 tool families (from README + `mx mcp` reality), RAG setup (`rag.toml`, Neon or local Postgres, `OPENAI_API_KEY` or local `embedding_base_url` — the honest locality story), research pipeline (cross-link RESEARCH_LOG corpus page), agent execution rules (cross-link `instructions` corpus page).
- [x] **Project**: research log + backlog (corpus pages, linked), known-broken page — loader reads `tests/KNOWN_BROKEN.md` from the repo root (same pipeline rules: fail loudly if missing), license page (dual MIT/Apache).
- **Accept:** built pages render; known-broken page shows the live table from `tests/KNOWN_BROKEN.md`; license page matches LICENSE files.

### Task 9: llms.txt + llms-full.txt
- [x] Loader-driven generation at build time into the site root: `llms.txt` (site title, one-line description, grouped links with summaries per the llmstxt convention) and `llms-full.txt` (concatenated full text of all published guides + corpus docs, with per-doc separators and source paths).
- [x] Vitest coverage for both generators.
- **Accept:** `dist/llms.txt` lists every published page grouped by section; `dist/llms-full.txt` contains full corpus text (spot-check 3 docs); both excluded from Pagefind indexing.

## Phase 4 — Ship

### Task 10: `site.yml` CI + deploy
- [x] `.github/workflows/site.yml`: trigger on push to main + PRs, with path filter `site/**`, `docs/**`, `tests/KNOWN_BROKEN.md`. Jobs: `build` (setup-node, npm ci, `npm test`, `astro build`, link check via `lychee` on `dist/` offline mode) always; `deploy` (wrangler deploy to Cloudflare, env `CLOUDFLARE_API_TOKEN`/`CLOUDFLARE_ACCOUNT_ID` from GH secrets) only on main; `preview` (wrangler versions/preview deploy, comment URL on PR) on PRs — preview job `continue-on-error` until secrets exist.
- [x] Wrangler config in `site/apps/site` per the astro recipe's deploy path (static assets project targeting mechcrate.dev).
- [x] Do NOT touch `ci.yml`/`release.yml`.
- **Accept:** workflow YAML validates (`gh workflow view` after push or `actionlint` if available); `build` job passes on this PR; deploy steps are cleanly gated on secret presence (documented in the workflow header comment + handoff note for the user: register mechcrate.dev, create CF token, add 2 GH secrets).

### Task 11: Dogfood pass + final acceptance
- [x] `cd site && make doctor && make init && make dev` → Playwright against `http://mechcrate.localhost`: landing renders, a Start page renders, a corpus page renders with banner, Pagefind search returns a corpus hit, `/llms.txt` serves. Then `make down`.
  - Two real defects surfaced and were fixed, which is what the dogfood was for:
    1. `make dev` would not start at all — the astro recipe's compose `include:`s `db.yml`/`redis.yml` with `optional: true`, which is not a Compose field, so a missing db.yml is a hard error. Dropped db/redis from `site/docker/compose/site.{yml,dev.yml}` (static docs site needs neither). Recipe bug filed as `mech-crate-eic` (P1); the sibling `target: runner` mismatch as `mech-crate-47j` (P2).
    2. The container rendered `corpus: published 0 docs` — the loader reads repo-root `docs/development/`, which is outside the `apps/site` source mount. `site.dev.yml` now bind-mounts `docs/` read-only at `/repo/docs` and sets `MECHCRATE_REPO_ROOT`; `content.config.ts` honours it (unset elsewhere → unchanged behaviour). Container now logs `published 67 docs, held back 3`, and `/llms.txt` went from 50 lines / 0 corpus URLs to 177 lines / 83.
  - Pagefind is **not** verifiable under `make dev`: `astro dev` serves no search index and Starlight says so in the dialog ("Search is only available in production builds"). Recorded honestly in `task-11-search-devmode.png`, then verified against the real artifact via `npm run build` (110 pages, Pagefind indexed 110 HTML files) + `astro preview`: query "pgvector" → "14 results", top hit `/docs/corpus/database/pgvector-rust-batch-embedding/`, clicked through and landed on the page with its corpus banner. Zero console errors on every page checked.
- [x] Full repo gates still green: `make test` → 189 passed / 14 skipped; `mx rag ingest --dry-run` → 66 docs, 2201 chunks, 0 warnings. Site suite 191 vitest passed, `make site-build` 110 pages.
- [x] Add a "this site is an mx app" note to the site's Cloudflare-deploy page and a one-line README pointer to mechcrate.dev (README's only edit: add the site link under the header, plus the authorized recipe-table ⚠️→✅ sync). The Cloudflare page's stale ":::note[Not wired yet]" (asserting no `site.yml` exists) was replaced — Task 10 landed that workflow.
- [x] Push `feat/site`, open PR (base main) with screenshots; do not merge.
- **Accept:** all boxes above demonstrated in the session log; PR open with the site.yml build job green.

## Out of scope
Domain registration + CF secret creation (user); versioned docs, blog, analytics; fixing the recipe-scaffolder ordering bug (tracked in bd, Task 1 works around it).
