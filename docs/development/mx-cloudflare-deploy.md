---
title: "mx Cloudflare Deploy: As-Implemented Infra Flow, Credentials, and Known Traps"
category: infra
languages: [typescript]
complexity: advanced
use_cases:
  - deploying an mx app to Cloudflare Workers + Containers front to back
  - setting up Cloudflare credentials that the deploy toolchain actually reads
  - onboarding an existing apps/<svc> service to the Cloudflare container path
  - avoiding the documented-but-unimplemented mx cf command and other drift
summary: The as-implemented mx Cloudflare flow — mx new --infra scaffolding, cf-setup/cf-init onboarding, the Worker+Container runtime model, the credential paths that work (and the ones that don't) — researched from source with an 8-item drift/gap inventory.
provenance: researched
researched: 2026-07-19
sources:
  - templates/infra/cloudflare/ (mx repo — wrangler.toml, containers-worker, README)
  - templates/scripts/{cf-setup.sh,cf-init-app.sh} (mx repo)
  - templates/make/cloudflare.mk (mx repo)
  - crates/mx-cli/src/commands/{new,infra}.rs (mx repo)
  - crates/mx-lib/src/infra/config.rs + crates/mx-lib/src/config.rs (mx repo)
  - crates/mx-cli/src/main.rs (command surface)
  - docs/development/INFRA_CONFIG.md; docs/development/appendix-build-deploy-recipe.md (drift references)
---

# mx Cloudflare Deploy (As Implemented)

Companion to `mx-app-playbook.md` (§build: `mx-recipes-and-build.md`). This documents what the Cloudflare path **actually does today**, because two shipped docs describe flows that don't exist — follow this, not `INFRA_CONFIG.md`'s `mx cf` examples (§5). All citations are repo-relative paths in the mech-crate repo.

## 1. The model: Worker + Container via Durable Object

Cloudflare deploys are Worker-fronted containers (CF Containers beta). The generated worker [templates/infra/cloudflare/containers-worker/src/index.ts]:
- `ContainerDO extends Container` — `defaultPort 8080`, `sleepAfter '10m'`, env injection.
- Control plane: `/_health|/_healthz|/_readyz` (container state), `/_container/restart|start|status|stop` (lifecycle).
- **Everything else proxies into the container**: `return container.fetch(request)` [index.ts:75].

`wrangler.toml` binds it together: `[[containers]]` (name, `class_name = "ContainerDO"`, `image = registry.cloudflare.com/<acct>/<app>:v<ver>`, `max_instances`), `[[durable_objects.bindings]]` (`APP_CONTAINER`), sqlite migration, `[env.preview]`/`[env.production]` with `routes = [{pattern, zone_name}]` for custom domains [cf-init-app.sh:769-849].

## 2. Scaffolding: what `mx new --infra cloudflare` gives you

- Copies `templates/infra/cloudflare/` → `infra/cloudflare/` (README, `apps/.gitkeep`, and a standalone `containers-worker/` example) with `{{PROJECT_NAME}}` substituted in `.ts/.toml/.json/.md` [crates/mx-cli/src/commands/new.rs:388-471].
- Keeps `make/cloudflare.mk` (deleted when CF not selected) and always ships `scripts/cf-setup.sh` + `scripts/cf-init-app.sh` [new.rs:91-97, 187-210].
- **Terraform is NOT scaffolded** — no `*.tf` exists anywhere in `templates/` despite the `.gitignore` ignoring tfstate and `appendix-build-deploy-recipe.md` describing `make terraform-*` targets. The only terraform in the repo is a standalone reference example (`reference/gnn/infra/terraform/gpu-node/`).

The shipped `containers-worker/` is an **example**, never touched by tooling; real apps are generated under `infra/cloudflare/apps/<app>/` (§3).

## 3. The working flow, front to back

```bash
make cf-setup                 # wizard → wrangler login, account id, writes infra/cloudflare/.env.cloudflare
make cf-init a=myapp type=container   # or: worker | cron
# edit docker/dockerfiles/myapp/app to build YOUR service (see §4)
make cf-deploy a=myapp        # = cf-build → cf-push → cf-sync-image → wrangler deploy --env production
```

- `cf-setup` [templates/scripts/cf-setup.sh]: `wrangler login` if needed, detects/prompts `CF_ACCOUNT_ID`, optional `CLOUDFLARE_API_TOKEN`, writes **project-local** `infra/cloudflare/.env.cloudflare` (gitignored).
- `cf-init` [templates/scripts/cf-init-app.sh]: generates `infra/cloudflare/apps/<app>/` (worker `src/index.ts`, `wrangler.toml`, `package.json`, `tsconfig.json`) + a skeleton Dockerfile at `docker/dockerfiles/<app>/app` + `docker/.config/.env.<app>`. Three types: `worker` (plain fetch), `cron` (scheduled + KV state — KV namespace id left as TODO to create), `container` (§1 model).
- Deploy pipeline [templates/make/cloudflare.mk:99-186]: `cf-build` (`docker buildx build --platform linux/amd64 --build-arg APP_VERSION=<package.json version> --load`) → `cf-push` (`npx wrangler containers push`) → `cf-sync-image` (rewrites `image =` in wrangler.toml) → `wrangler deploy --config … --env production`. `cf-deploy-all` loops every `infra/cloudflare/apps/*/` with a wrangler.toml. Dev loop: `cf-dev`; observability: `cf-logs`, `cf-status`, `cf-container-status`.
- This is the **only** mx build path that stamps the released `package.json` version into the image (`v<APP_VERSION>` tags; see `mx-recipes-and-build.md` §5).

## 4. Onboarding an existing `apps/<svc>` service

There is no automated bridge — `cf-init` always scaffolds a fresh skeleton [cf-init-app.sh:125, 853-903].

### Synthesis (inferred)

The procedure that follows from the mechanics:
1. `make cf-init a=<svc> type=container` to generate the worker + wrangler.toml + Dockerfile skeleton.
2. Replace the skeleton `docker/dockerfiles/<svc>/app` with a Dockerfile that builds your actual `apps/<svc>` source (build context is repo root — reuse your recipe's production stage as the base). `cf-build` uses exactly this path [cloudflare.mk:21, 104-106].
3. Align ports: the worker's `ContainerDO.defaultPort` and the Dockerfile's `EXPOSE`/`PORT` must match your app's listen port; also make your app serve `/health` — the generated Dockerfile HEALTHCHECK hits `/health` on the app directly, NOT the worker's `/_health` routes [cf-init-app.sh:900 vs index.ts:3].
4. Set real `routes` in `[env.production]` (pattern + zone) — the defaults are placeholders.
5. Keep local dev on the mx router; Cloudflare is the production path. The two coexist: same Dockerfile family, different front doors.

## 5. Credentials: what works and what is broken

Two scopes exist and are consistently *located*: global `~/.mech-crate/config/infra/cloudflare.env`, project `infra/cloudflare/.env.cloudflare` [crates/mx-lib/src/infra/config.rs:84-97; crates/mx-lib/src/config.rs:54-57].

**The working path:** `make cf-setup` → project-local `.env.cloudflare` containing `CF_ACCOUNT_ID` (+ `CF_DOCKER_PLATFORM`, optional `CLOUDFLARE_API_TOKEN`) → `cloudflare.mk` `-include`s that file and exports `CLOUDFLARE_ACCOUNT_ID ?= $(CF_ACCOUNT_ID)` for wrangler [cloudflare.mk:7-16]. Token absent → wrangler uses its OAuth login.

**The broken paths (verified; do not send agents down these):**
- `mx infra setup cloudflare` writes `CLOUDFLARE_ACCOUNT_ID=` to the **global** file [crates/mx-cli/src/commands/infra.rs:107-136], but the deploy toolchain reads **`CF_ACCOUNT_ID`** from the **project** file — credentials created via `mx infra setup` are never consumed by `make cf-*`; `cf-init` errors "CF_ACCOUNT_ID not set".
- `mx infra link`/`unlink` are stubs (print-only) [infra.rs:307-333], and the two link mechanisms that do exist disagree: the Rust resolver looks for a `.env.linked` marker file [config.rs:117] while the bash scripts look for `MX_INFRA_USE_GLOBAL=true` **inside** the env file [cf-init-app.sh:102]. Neither is ever written by tooling.
- `cloudflare.mk` includes only the project file — even a hand-linked global setup works for `cf-init` but not for the deploy targets [cloudflare.mk:15].
- **`mx cf <anything>` does not exist.** No `Cf` variant in the CLI command enum [crates/mx-cli/src/main.rs:34-151]; `INFRA_CONFIG.md` (`mx cf setup/config/deploy`) and the MCP server's resource text both document a phantom command.

**Agent rule:** for Cloudflare credentials, use `make cf-setup` (project-local) and treat `mx infra setup` as global-credential storage for other tooling until the env-var mismatch is fixed.

## 6. Drift & gaps inventory (as of research date)

1. `mx cf` documented in INFRA_CONFIG.md + MCP resources, unimplemented in the CLI.
2. `CLOUDFLARE_ACCOUNT_ID` (mx infra) vs `CF_ACCOUNT_ID` (toolchain) mismatch — global creds unusable by deploys.
3. `mx infra link/unlink` stubs; `.env.linked` marker vs `MX_INFRA_USE_GLOBAL` in-file flag — two incompatible, both unwired.
4. `cloudflare.mk` has no global-credential fallback (project `-include` only).
5. Terraform: gitignored, documented in the appendix, never scaffolded; no `make terraform-*` targets exist.
6. `appendix-build-deploy-recipe.md` prescribes `infra/dockerfiles/` + `containers-worker/` layout; shipped reality is `docker/dockerfiles/<app>/app` + `infra/cloudflare/apps/<app>/` — treat the appendix as aspirational, this doc as ground truth.
7. Generated Dockerfile HEALTHCHECK path (`/health`) is the app's responsibility, not the worker's `/_health` control route — mismatched expectations if the app doesn't serve it.
8. Cron type ships `kv_namespaces` id `TODO_CREATE_KV_NAMESPACE` — deploy fails until the namespace is created and the id pasted in.

## 7. Agent quick-reference

- CF flow = `make cf-setup` → `make cf-init a=<app> type=container` → edit the app Dockerfile → `make cf-deploy a=<app>`. Never `mx cf` (doesn't exist).
- Credentials the deploys read: `infra/cloudflare/.env.cloudflare` with `CF_ACCOUNT_ID`. Nothing else reaches `make cf-*`.
- Production URL comes from `[env.production] routes` in the app's wrangler.toml; local dev URLs still come from the mx router.
- The Worker proxies everything to the container; container image versions come from `package.json` (`v<version>`), unlike every other mx build path.
