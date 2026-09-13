# Wave 1 — CLI Functionality Gaps (broken promises & data safety)

**Branch:** `fix/wave1-cli-gaps` (from main @ f25b82f)
**Tracking:** existing bd issues z5i / 71u / eic / 0uq own tasks 1-4; T5 gets a new issue.
**Toolkit:** cli throughout. TDD discipline: where a red `#[ignore = "bd:..."]` lane test exists, the fix makes it pass and REMOVES the ignore; the test joins the release gate. Never weaken a test.

Conventions: all work on this branch; one commit per task in repo style; `make test` must stay green after every task (189+ passed baseline; grows as lane tests join); no .env* commits; bd handled by the orchestrator; bd pre-commit hook sweeps .beads/issues.jsonl — amend out with --no-verify, restore staged state.

### Task 1: mx upgrade discovery against the real templates layout (bd mech-crate-z5i)
- [x] Red test exists: `upgrade_discovery_works_against_real_templates_layout` (crates/mx-lib/src/upgrade/mod.rs, `#[ignore = "bd:mech-crate-z5i"]`). Make it pass: upgrade discovery must read the SHIPPED layout (templates/Makefile.template, templates/make/*.mk, templates/scripts/*, templates/docker/**) — no phantom `templates/project/`. Preserve the categorization matrix (tooling/config/conditional/skip) and backup naming.
- [x] Un-ignore the test; remove the z5i row from tests/KNOWN_BROKEN.md; update its scoreboard counts.
- [x] Live E2E: scaffold a project in scratch (`mx new` + `mx add api --recipe rust-api --domain api.localhost` using repo-built mx with MECH_CRATE_ROOT), then `mx upgrade --dry-run` and `--diff` exit 0 with sensible plans; `mx upgrade --yes` applies cleanly and is idempotent (second run = no-op).
- **Accept:** lane test green un-ignored; dry-run/diff/apply/idempotence proven live in scratch; `make test` green.

### Task 2: per-project compose project name — stop adopting foreign containers (bd mech-crate-71u)
- [x] Generated projects must pin COMPOSE_PROJECT_NAME to a per-project value (sanitized project dir name) everywhere compose runs: templates scripts (.bashrc compose_context_files/dev.sh/up.sh etc.) or docker/.config/.env.shared — pick the single authoritative point and document it. Two different mx projects on one machine must never share the default compose project name or adopt each other's containers.
- [x] Existing-project migration note: changing the project name orphans running containers from the old name — write the behavior into the upgrade docs rider input (T5) and `make doctor` should surface a mismatch if detectable cheaply.
- [x] Tests: template-level assertions + a scaffold test proving two scratch projects produce distinct project names and that compose invocations carry them (stub docker or parse the composed command).
- **Accept:** two scratch projects verified isolated (distinct COMPOSE_PROJECT_NAME flowing into compose calls); tests green; `make test` green.

### Task 3: astro recipe compose include fix (bd mech-crate-eic)
- [x] Read the issue notes (design context in bd). `include: optional: true` is not a Compose field — silently ignored, so a missing db.yml/redis.yml is a hard error. Fix the astro recipe (and grep ALL recipes/templates for the same pattern) so `mx add <svc> --recipe astro` + `make dev` starts WITHOUT db/redis siblings and WITH them when present: either ship the db/redis compose files with the recipe unconditionally, or drop the include and rely on compose_context_files aggregation — choose from how the other 6 recipes handle it (consistency wins) and record the choice.
- [x] Extend recipes conformance tests: for EVERY recipe, the composed dev config must `docker compose config` cleanly straight after `mx add` (no missing-file include errors) — that's the regression net for this class.
- [x] Live E2E: scratch project + astro recipe + `make dev` reaches a running container (or at minimum `docker compose config` + compose up dry paths clean if the full boot is heavy — but prefer the real boot; T11-era agents did it).
- **Accept:** astro `make dev` works without siblings; conformance suite covers compose-config for all 7 recipes; `make test` green.

### Task 4: recipe scaffolders actually run (bd mech-crate-0uq)
- [x] Root cause on record: mx creates the manifest's `directories` BEFORE init_app, so `skip_if_exists: true` always trips and `npm create astro` / `nuxi init` / `zola init` never execute — apps land with only a health endpoint. Fix the ordering (init_app before directory creation) or narrow skip_if_exists to framework marker files (e.g. package.json) — read the installer code and pick the change that keeps the round-trip conformance tests meaningful.
- [x] Tests: unit test on the ordering/skip decision + conformance extension asserting that after `mx add`, the app dir contains framework scaffolding markers when the tool is available.
- [x] Live E2E: astro + nuxt scaffolders actually execute in scratch (npm available on this machine). zola: run if the binary exists, otherwise verify via recorded-invocation stub and say so honestly.
- [x] Interaction guard: T3's compose behavior and T1's upgrade must still pass after this ordering change (run their suites).
- **Accept:** scaffolders proven to execute (astro/nuxt live); recipes conformance still green for all 7; `make test` green.

### Task 5: docs rider + ship
- [x] Lift the honesty markers the fixes retire: site framework/upgrade page (z5i banner → working feature with real output), site recipes page + README recipe notes if they reference eic/0uq behavior, llms.txt agent-instructions "mx upgrade is mid-repair" line (src/loaders/lib/llms.ts), and any KNOWN_BROKEN cross-references. Add the COMPOSE_PROJECT_NAME migration note (T2) to the upgrade/compose docs.
- [x] Scoreboard: tests/KNOWN_BROKEN.md counts updated for every un-ignored test; `make test-known-broken` output consistent.
- [x] Full gates: `make test`, `make check`; site: `npm test` + `npx astro build` (docs edits ride site.yml on merge). `mx rag ingest --dry-run` 0 warnings.
- [x] Push branch; PR base main titled `fix: Wave 1 CLI gaps — mx upgrade, compose isolation, astro include, recipe scaffolders`; body: per-fix summary with lane tests un-ignored, docs lifted, migration note; watch site.yml + ci.yml to green. Do NOT merge.
- **Accept:** PR open, both workflows green, honesty markers lifted, scoreboard consistent.
