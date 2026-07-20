---
title: "mx Recipes & Build Pipeline: Lifecycle, Distribution, and Images"
category: process
languages: []
complexity: advanced
use_cases:
  - understanding how mx recipes/blueprints are discovered, versioned, and distributed
  - updating consumers when a recipe changes upstream (what exists vs the standard approach)
  - building images the mx way (make build vs compose vs cloudflare paths)
  - running or debugging the release/version-bump flow in an mx project
summary: Complete trace of the mx recipe lifecycle (local scaffolding recipes AND unyform remote blueprints), how updates reach consumers, and the three image-build paths with their version/tag semantics — researched from source with known gaps flagged.
provenance: researched
researched: 2026-07-19
sources:
  - crates/mx-cli/src/commands/{recipes,unyform,init,upgrade,build,self_update}.rs (mx repo)
  - crates/mx-lib/src/{recipe,unyform,upgrade,paths,config,docker}/ (mx repo)
  - crates/mx-mcp-server/src/{tools/mod.rs,unyform/mod.rs,mx/mod.rs} (mx repo)
  - templates/recipes/ (recipe.json specs, dockerfiles, compose fragments)
  - templates/make/{build,release,cloudflare,common}.mk + templates/scripts/{build.sh,simple-release.sh,app-version.mjs,release-sync-versions.mjs,.bashrc} (mx repo)
  - templates/recipes/README.md; docs/development/RECIPE_AUTHORING_GUIDE.md; docs/development/appendix-build-deploy-recipe.md
  - tests/testbed/ (mx repo)
---

# mx Recipes & Build Pipeline

Companion to `mx-app-playbook.md`. That doc covers *using* mx; this one covers the machinery underneath: where recipes come from, how they reach machines and projects, what happens (and doesn't) when they change, and exactly how images get built. All bracketed citations are repo-relative paths in the mech-crate repo.

## 1. Two different things are called "recipe" — never conflate them

| | **Local scaffolding recipes** | **Unyform remote recipes ("blueprints")** |
|---|---|---|
| Live at | `templates/recipes/<name>/` (JSON + template tree) | `api.unyform.ai`, generated server-side from connected repos |
| Consumed by | `mx add`, `mx new --with`, `mx recipes list/info` | `mx recipes pull/apply/versions/cache`, `mx unyform`, MCP `unyform_*` |
| Produce | Real files: `apps/`, compose pair, dockerfiles, env | One coding-rules doc: `.cursor/rules/<name>-patterns.md` |
| Versioning | `version` field exists but is **dead metadata** — never read, shown, or compared [crates/mx-lib/src/recipe/parser.rs:26-28] | Real versions: list/pull `name@version`, `versions` endpoint, cached per-version |

`mx recipes` straddles both: `list`/`info` are local, `pull/apply/versions/cache` are Unyform [crates/mx-cli/src/commands/recipes.rs:57-69].

## 2. Local recipe lifecycle

**Authoring.** A recipe is `recipe.json` (services, options, placeholders with `slug|upper|rust_crate|ssr_bool` transforms, `init_app` scaffolder, `templates` mappings, `post_install`, `next_steps`) plus its template files [templates/recipes/README.md; crates/mx-lib/src/recipe/parser.rs]. Shared fragments are referenced cross-recipe via `common://` → `templates/recipes/common/` [installer.rs:202-214]. Full authoring guide: `RECIPE_AUTHORING_GUIDE.md`.

**Discovery & resolution.** `mx recipes list/info` and `mx add` resolve recipes ONLY from `<templates>/recipes`, where `<templates>` is searched in order: `$MECH_CRATE_ROOT/templates` → `~/.mech-crate/templates` → exe-relative walk-up [crates/mx-lib/src/paths.rs:65-102]. Two traps:
- `~/.mech-crate/recipes` is **not** a recipe source — it's the Unyform download cache [crates/mx-lib/src/config.rs:39-42].
- **Projects cannot carry their own recipes** — resolution never consults the project dir or cwd.

**Consumption.** `mx add <name> --recipe=<r>` runs the installer: placeholders → mkdir → `init_app` (skipped if target exists + `skip_if_exists`) → Tera-interpolated template copy → post_install → next_steps [installer.rs:71-123]. File mapping (rust-api, representative): `service.yml`→`docker/compose/<svc>.yml`, `service.dev.yml`→`<svc>.dev.yml`, `app.Dockerfile`→`docker/dockerfiles/<svc>/app`, `app.prod.Dockerfile`→`…/app.prod`, `app/`→`apps/<svc>/` [templates/recipes/rust-api/recipe.json:61-80].

**Maintenance & testing.** Recipes are hand-edited in `templates/recipes/`. Validation today: serde parse at load + parser/interpolator unit tests against hardcoded JSON only [parser.rs:359-420]; shell smoke tests exist in `tests/testbed/` (laravel, rust-api scaffold) but are **not wired into CI** [.github/workflows/release.yml is the only workflow]. No linter loads the real on-disk recipe.json files.

**Distribution to machines.** Recipe edits reach a machine only via a full templates recopy:
- `mx init --update` (or `--force`): deletes and recopies `~/.mech-crate/templates` from the source repo [crates/mx-cli/src/commands/init.rs:92-121].
- `mx self-update [--pull]`: git pull, rebuild binaries, then runs `mx init --force` — so it refreshes recipes as a side effect [crates/mx-cli/src/commands/self_update.rs:96-249].
- There is **no unyform push** — the remote registry is read-only from this CLI; blueprints are generated server-side from connected repositories [crates/mx-mcp-server/src/tools/mod.rs:2275-2278].

## 3. Unyform blueprint distribution (the built mechanism)

**Auth.** `mx login --api-key <k>` → `GET /v1/auth/me`, credentials stored at `~/.mech-crate/config/unyform/credentials.json` (0600) [crates/mx-lib/src/unyform/mod.rs:119-157, 241-270]. Default base `https://api.unyform.ai`. Browser OAuth is not implemented in the CLI [crates/mx-cli/src/commands/unyform.rs:73-107].

**Commands and endpoints** [mx-lib/unyform/mod.rs:296-452; recipes.rs:178-338]:

| Command | Endpoint | Effect |
|---|---|---|
| `mx recipes list` | `GET /v1/orgs/{org}/recipes` | Remote list appended after local list when logged in |
| `mx recipes versions <name>` | `GET …/recipes/{name}/versions` | Prints version, is_latest, generated_at |
| `mx recipes pull <name[@ver]>` | `GET …/recipes/{name}[/versions/{v}]` | Caches to `~/.mech-crate/recipes/{org}/{name}/{version}/recipe.json` + `latest` symlink |
| `mx recipes apply <name[@ver]>` | same GET (always API, ignores cache) | Writes `.cursor/rules/<name>-patterns.md` from `patterns` |
| `mx recipes cache [list\|clear]` | — | Walks/clears the cache tree |

**Cache layout:** `~/.mech-crate/recipes/{org}/{name}/{version}/recipe.json`; `latest` symlink points at the **last pulled** version (not necessarily newest). The MCP client also writes a `manifest.json` (`pulled_at/org/name/version`) — the only pull provenance that exists anywhere.

**Known defects in this surface** (verified; candidates for issues):
- `apply --fix` is a dead flag (`_fix` unused) and the advertised dependency-drift/infrastructure comparison is unimplemented [recipes.rs:206; tools/mod.rs:1325-1333, 2358].
- The two Unyform clients disagree on the org path segment: CLI sends the org **id**, MCP sends the org **slug** [mx-lib/unyform/mod.rs:206-211 vs mx-mcp-server/unyform/mod.rs:258-267].
- No update checking or pinning: nothing compares cached vs remote versions.
- `mx init` records `~/.mech-crate/config/source-root` but `mx self-update`'s fallback reads `~/.mech-crate/source` — mismatched marker files [paths.rs:17-21 vs self_update.rs:315].

## 4. Consumer update propagation — what exists vs the standard approach

**What exists today (the honest answer): recipe-generated files have no update path.**
- `mx upgrade` refreshes *tooling* (make/, scripts/, Makefile — prompt-to-update with `.bak` backups) and treats docker config as add-if-missing; anything under `recipes/` is categorized **Skip** — recipe-generated services are never touched [crates/mx-lib/src/upgrade/mod.rs:66-102]. (And upgrade is currently broken anyway — see `mx-app-playbook.md` §4.)
- Re-running `mx add` overwrites template-managed files **unconditionally** — an "update" that also clobbers local edits, with no diff, no merge, no prompt [installer.rs:252-299].
- Unyform `apply` rewrites its rules file wholesale each run; fine for that file, irrelevant for scaffolded code.
- **No project records which recipe or version generated its services.** No lockfile, no marker; `InstallResult` is printed and discarded [installer.rs:401-409]. Upstream changes therefore cannot even be *detected* from a consumer project.

### Synthesis (inferred)

The standard approach until mx grows native support — treat recipes as **scaffolding, not a managed dependency**, and make provenance explicit:

1. **Record provenance at add-time.** Immediately after `mx add`, commit a `docker/.config/recipe-lock.json` in the project: `{service: {recipe, version (from recipe.json), templates_sha (git SHA of mech-crate at add time), added_at}}`. This is convention, not tooling — but it makes every later question answerable.
2. **Updating a service against a newer recipe** = regenerate-and-diff, never overwrite-in-place: run `mx add <svc> --recipe=<r>` in a **scratch project**, then diff the rendered output against the real project's files and port changes deliberately. Hand-edited compose/dockerfiles are the norm, so a blind re-add is a destructive operation — the scratch-render diff is the safe substitute for the missing update command.
3. **Divergence is expected and fine** for app source (`apps/`); keep the *contract surfaces* (compose labels/networks, env layering, dockerfile stage names) recipe-conformant so tooling and the router keep working.
4. **Unyform blueprints**: pull pinned (`name@version`), commit the applied `.cursor/rules` file, and re-apply deliberately after reviewing `mx recipes versions <name>` — the `latest` symlink is last-pulled, so never trust it as "newest".
5. **The native fix** (proposed as an mx feature, filed as an issue): `mx add` writes the lock entry itself, and an `mx recipe diff <svc>` renders the current recipe to a temp dir and shows the three-way delta. That single pair would turn recipes into a real managed dependency.

## 5. The build pipeline — three paths, three version semantics

| Aspect | `make build` / `mx build` | Compose (`make dev`/`up`) | Cloudflare (`make cf-*`) |
|---|---|---|---|
| Dockerfile (prod) | prefers `docker/dockerfiles/<svc>/app.prod`, falls back to `app` | always `app`, `target: production` | `app` |
| Builder | `DOCKER_BUILDKIT=1 docker build` | `docker compose up` — builds only if image absent (**no `--build` anywhere**) | `docker buildx build --platform linux/amd64 --load` |
| Version into image | `IMAGE_VERSION` = git `branch:sha8[-dirty]`; tag from `t=` (default `latest`) | none | `APP_VERSION` = package.json version |
| Image name | `<PROJECT>/<svc>:<tag>` + `:<tag>-dev\|-prod` twin | compose default naming or `<SVC>_IMAGE_TAG` from `docker/compose/.env` | `registry.cloudflare.com/<acct>/<app>:v<ver>` |
| Push | `docker push` iff `push=1` (no registry prefix logic) | n/a | `npx wrangler containers push` |

[templates/make/build.mk; templates/scripts/build.sh:29-202; templates/scripts/.bashrc:36-180; templates/make/cloudflare.mk:99-186]

**`make build s=<svc> [t=<tag>] [prod=1] [push=1] [nocache=1]`** → `scripts/build.sh`: picks the dockerfile by mode (prod prefers the hardened distroless `app.prod`; dev uses the `app` file's `development` stage), builds from **project root** context, passes `IMAGE_VERSION/IMAGE_TAG/BUILD_TIME/BUILD_MODE` build-args (+`NODE_ENV/RUST_ENV/APP_ENV=production` in prod), runs an optional `scripts/prebuild/<svc>.sh` hook, and writes the tag back to `docker/compose/.env` as `<SVC>_IMAGE_TAG` [build.sh:55-185]. `mx build` is a thin wrapper over `make _build` (MCP `mx_build` → `mx build` → make → build.sh) [crates/mx-cli/src/commands/build.rs:48-102; crates/mx-mcp-server/src/mx/mod.rs:215-239].

**Compose builds**: recipe compose declares `build: {context: ../.., dockerfile: docker/dockerfiles/<svc>/app, target: production}` with the dev override flipping only `target: development` [templates/recipes/rust-api/docker/compose/service.yml:9-14, service.dev.yml:6-8]. Because `up` never passes `--build`, **compose reuses existing images; to pick up code changes in the image you must `make build` (or rely on dev bind-mounts, which is why dev mode rarely needs rebuilds)**. There is no `make rebuild` — it's listed `.PHONY` but never defined [templates/make/common.mk:4].

**Dockerfile convention** (rust-api, representative): one multi-stage `app` file — `chef → planner → builder` (cargo-chef cached deps) then a `development` stage (`cargo watch`) and a `production` stage (alpine, non-root) [templates/recipes/rust-api/docker/dockerfiles/app.Dockerfile]; plus a hardened `app.prod` (static musl, stripped, distroless nonroot) used only by the standalone prod build [app.prod.Dockerfile].

**Release flow**: `make release* app=<app>` targets [templates/make/release.mk] — version source of truth is `apps/<app>/package.json`; conventional-commit releases delegate to per-app `yarn release*` (standard-version), `post_release` syncs versions into `.release-please-manifest.json` and amends the changelog; `release` auto-pushes with `--follow-tags`. The simple path (`release-simple*`) uses `npm version` + tag `<app>-v<ver>` without conventional commits [simple-release.sh]. **Gap:** recipe-generated apps don't ship the `release*` package scripts or standard-version — `make release` fails on a fresh recipe app until they're added (bootstrap steps in `appendix-build-deploy-recipe.md`); `release-simple` works out of the box.

**Released version ≠ image version** for standalone/compose builds (git-derived `IMAGE_VERSION` only); the **Cloudflare path is the only one that stamps `APP_VERSION` from package.json** into the image [cloudflare.mk:24-29, 108].

## 6. Gaps inventory (as of research date)

Filed or worth filing as mx issues; treat these as "known, don't rediscover":
1. Local recipe `version` is dead metadata; no changelog/version discipline for scaffolding recipes.
2. No consumer provenance (no lockfile) → no update detection or propagation for recipe-generated code (§4 standard approach compensates).
3. `mx recipes apply --fix` dead flag; advertised dependency/infrastructure comparison unimplemented.
4. Unyform org id-vs-slug mismatch between CLI and MCP clients.
5. `mx upgrade` broken (`templates/project/` missing — also flagged in mx-app-playbook).
6. `mx build --platform` is accepted but dropped by the make layer (`_build` doesn't forward it); only `build-multiplatform` wires platforms through.
7. No `make rebuild`; compose never passes `--build`.
8. Recipe apps lack `release*` scripts → `make release` fails until bootstrapped.
9. `source-root` vs `source` marker mismatch (init vs self-update).
10. testbed smoke tests not in CI; no recipe.json validator.

## 7. Agent quick-reference

- "Recipe" alone = local scaffolding; "blueprint"/pull/apply/versions = Unyform remote. Different systems, different outputs.
- Updating a recipe-built service: scratch-render + diff (§4.2). Never re-run `mx add` over hand-edited files.
- After editing recipes in the mx repo: `mx init --update` to distribute to the machine.
- Fresh code into a prod image: `make build s=<svc> prod=1` (compose won't rebuild for you).
- Pin Unyform pulls with `@version`; don't trust the `latest` symlink.
