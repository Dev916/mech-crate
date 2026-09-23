# Wave 3 — CLI Functionality Gaps (multi-stack concurrency, recipe build repair, non-dev env)

**Branch:** `fix/wave3-cli-gaps` (from main @ abd0fca, post 0.1.4 release)
**Tracking:** existing bd issues own tasks 1-4 (T1=fq3, T2=q1w, T3=1a0, T4=298); T5 gets a new issue at dispatch.
**Toolkit:** cli throughout. TDD discipline: red first where a test can express the defect; conformance nets extend the recipes/templates suites; never weaken a test.
**Compatible with:** devloop skill v0.1+
**Corpus note:** mx techniques MCP offline this session; domain facts inherited from the wave-2 runbook intel (compose interpolation, env layering, label routing) instead.

Conventions: all work on this branch; one commit per task in repo style; `make test` must stay green after every task (verify the baseline fresh at T1 and record the count — main has moved through 0.1.3/0.1.4 since wave 2's 252); no .env* commits; bd handled by the orchestrator; bd pre-commit hook sweeps .beads/issues.jsonl, so commit with --no-verify and explicit pathspecs, restore staged state. Standing invariants: every compose call in templates/scripts/*.sh carries `-p "$COMPOSE_PROJECT_NAME"` (templates_compose_isolation.rs); no em dashes in authored site/README copy (authored-copy.test.ts); live scaffolding uses the repo-built mx (cargo build; MECH_CRATE_ROOT=repo; target/debug/mx new --no-prompt + mx add).

### Task 1: rust-worker Dockerfile unexpanded {{RUST_VERSION}} — recipe cannot build (bd mech-crate-fq3)

**Acceptance Criteria (cli-observable):**
**Verify via:** cli
- Fresh scratch project: `mx new --no-prompt` + `mx add worker --recipe rust-worker` produces a Dockerfile whose FROM line parses (no `{{...}}` left in any installed file).
- `docker compose -p <proj> build` (or `make build s=worker`) for the scaffolded rust-worker proceeds past image reference parsing (no `invalid reference format`) and completes or fails only on genuine compile steps, not template placeholders.
- Conformance net: recipes suite asserts NO shipped recipe file contains a `{{PLACEHOLDER}}` token that neither recipe.json options nor the installer substitutes (mirror of the wave-2 `__GENERATE_*__` net).
- `make test` green.

- [ ] Decide the mechanism: substitution at install time (recipe.json option with a pinned default, like other options the installer already expands) vs a hard pin in the Dockerfile. Align with how rust-api/rust-leptos pin their toolchain; record the call in the commit message and issue notes.
- [ ] Red-first conformance test for the unexpanded-placeholder class across all 7 recipes, then the fix.
- [ ] Live E2E in scratch per the criteria above.

### Task 2: non-dev compose renders DATABASE_URL empty — class fix across db-bearing recipes (bd mech-crate-q1w)

**Acceptance Criteria (cli-observable):**
**Verify via:** cli
- Fresh scratch rust-api and astro projects: `docker compose -p <proj> -f <base files> config` (non-dev path) renders a non-empty, correct DATABASE_URL and emits ZERO `variable is not set` warnings.
- Live non-dev boot (`make up`) of one db-bearing recipe reaches a healthy db and the app answers its health endpoint with generated credentials and no hand edits.
- `environment:` blocks in shipped compose files no longer override the generated literals with bare `${DB_*}` interpolations (rust-api's derive-at-runtime shape is the model); astro's `:-secret` fallbacks that mask the defect are gone.
- The repo's own `site/docker/compose/site.yml` is swept into the same class fix (it shares the shape AND still lists .env.secrets before .env.shared), and the conformance net now covers site/docker/compose in addition to templates/.
- `make test` green.

- [ ] Red-first: conformance test asserting no compose `environment:` value in templates/, recipes, or site/docker/compose interpolates `${DB_USER}`/`${DB_PASSWORD}`/`${DB_NAME}` (compose only reads project-dir .env for `${}`; env_file layers are invisible to interpolation — proven live in wave-2 T1).
- [ ] Class fix: follow the rust-api model (runtime derivation / literals written by the wave-2 generator), not per-file patches. Fix the env_file ORDER in site.yml while in there (shared before secrets).
- [ ] Live E2E per the criteria.

### Task 3: pinned host ports block two concurrent stacks (bd mech-crate-1a0)

**Acceptance Criteria (cli-observable):**
**Verify via:** cli
- TWO fresh scratch projects, each with a db-bearing recipe, `make dev` BOTH concurrently with zero hand edits: no `port is already allocated` from any of the ~20 pinned publishes (db 5432, redis 6379, HMR/metrics 24678 astro/nuxt, 5173+13714 laravel, 3001 rust-leptos, 1024 zola, 9090 rust-worker, 9229 app.dev.yml).
- Single-stack UX unchanged: a lone `make dev` still exposes the dev-convenience ports at their documented defaults (HMR still reachable by the browser).
- Conformance net: every host-side port publish in templates/ and recipes dev overrides is either env-parameterized (`${VAR:-default}` or `${VAR:-0}`) or absent; no bare pinned host port remains.
- `make test` green.

- [ ] Design call, recorded per port class: infra ports (db, redis, metrics, debugger) default EPHEMERAL via `${<SVC>_HOST_PORT:-0}` with discovery via `docker compose -p <proj> port`; browser-facing dev ports (HMR, dev servers) default PINNED but env-overridable (`${VAR:-24678}`) so a second stack overrides instead of dying. Record rationale in the conformance test header.
- [ ] Read bd mech-crate-qpy (legacy port-publishing overlap) before sweeping; if the same class, cover its sites too and note it for the orchestrator (folding is the orchestrator's call).
- [ ] Red-first conformance net, then the sweep, then live two-stack E2E.

### Task 4: Traefik router/service names collide across projects (bd mech-crate-298)

**Acceptance Criteria (cli-observable):**
**Verify via:** cli
- TWO fresh scratch projects each defining a service with the SAME name (e.g. `api`), `make dev` both: Traefik logs show NO `Router defined multiple times` error, and BOTH apps answer 200 through the router at their own hostnames.
- `make down` on one stack leaves the other routable (re-verify its 200 through the router).
- Conformance net: every `traefik.http.routers.*` / `traefik.http.services.*` name in templates/ and recipes is project-qualified (`${COMPOSE_PROJECT_NAME}-...`); no bare `{{SERVICE_NAME}}`-only router name remains. mx-router's own container/labels are the deliberate singleton exception (wave-2 design call).
- `make test` green.

- [ ] Fix shape per the issue: `traefik.http.routers.${COMPOSE_PROJECT_NAME}-{{SERVICE_NAME}}.*` — .bashrc already exports COMPOSE_PROJECT_NAME for every compose call, so CLI-time interpolation resolves. Sweep all 7 recipes' service.yml plus laravel worker/scheduler fragments and any templates/docker/compose files carrying traefik labels; sweep site/docker/compose for the same shape.
- [ ] Red-first conformance net, then the sweep, then the two-stack finale E2E (this is the proof wave 2 could not honestly claim).

### Task 5: docs rider + ship

**Acceptance Criteria (cli-observable):**
**Verify via:** cli
- The four wave-2 honesty caveats ("name collisions fixed, two-stack concurrency blocked by 1a0+298") are lifted everywhere they shipped (grep for them: RECIPE_AUTHORING_GUIDE.md, site framework pages, tests/KNOWN_BROKEN.md, llms.ts) and replaced with the now-true multi-stack story.
- Docs carry a breaking-change callout for anyone scripting against pinned host ports or unqualified traefik router names, with the new env-override names and label shape.
- Full gates green: `make test`, `make check`; site `npm test` + `npx astro build`; `mx rag ingest --dry-run` 0 warnings.
- PR open from `fix/wave3-cli-gaps` to main, ci.yml + site.yml green. Do NOT merge.

- [ ] Ride-along: the pending 3-line RESEARCH_BACKLOG.md append (uncommitted in the worktree) — include and flag it in the PR body.
- [ ] PR title `fix: Wave 3 CLI gaps — ephemeral host ports, project-qualified Traefik labels, rust-worker build, non-dev env rendering`; body: per-fix summary + breaking-change note + design calls recorded; trailer per repo convention.
