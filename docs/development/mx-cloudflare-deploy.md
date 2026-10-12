---
title: "mx Cloudflare Deploy: As-Implemented Infra Flow, Credentials, Platform State and Known Traps"
category: infra
languages: [typescript]
complexity: advanced
use_cases:
  - deploying an mx app to Cloudflare Workers + Containers front to back
  - setting up Cloudflare credentials that the deploy toolchain actually reads
  - onboarding an existing apps/<svc> service to the Cloudflare container path
  - avoiding the documented-but-unimplemented mx cf command and other drift
  - deciding what Terraform can own in a Cloudflare Workers deploy and what must stay with wrangler
  - shipping a static site as an assets-only Worker (the mechcrate.dev path)
  - sizing, pricing and rolling out a Cloudflare Container as the platform stands in October 2026
summary: The as-implemented mx Cloudflare flow researched from source, covering mx new --infra scaffolding, cf-setup/cf-init onboarding, the Worker+Container runtime model, and the credential model (one canonical variable pair, global-then-project resolution); refreshed 2026-10-12 against the Cloudflare platform (Containers GA since April, the Container base class frozen after 2026-12-31, exports replacing migrations, staged rollouts, instance types, pricing, the September cross-tenant disk flaw), the repo's second Cloudflare path (assets-only Worker for mechcrate.dev), what Terraform can and cannot own, the cf CLI and Deno horizon, and a 16-item drift inventory whose template pins are three wrangler majors behind.
provenance: researched
researched: 2026-10-12
sources:
  - templates/infra/cloudflare/ (mx repo — wrangler.toml, containers-worker, README)
  - templates/scripts/{cf-setup.sh,cf-init-app.sh} (mx repo)
  - templates/make/cloudflare.mk (mx repo)
  - crates/mx-cli/src/commands/{new,infra}.rs (mx repo)
  - crates/mx-lib/src/infra/config.rs + crates/mx-lib/src/config.rs (mx repo)
  - crates/mx-cli/src/main.rs (command surface)
  - crates/mx-cli/tests/known_broken.rs (phantom mx cf tracked as bd:mech-crate-vxq)
  - site/apps/site/wrangler.jsonc, site/apps/site/public/_headers, .github/workflows/site.yml (mx repo — the assets-only path)
  - templates/recipes/astro/docker/dockerfiles/app.prod.Dockerfile (mx repo)
  - docs/development/INFRA_CONFIG.md; docs/development/appendix-build-deploy-recipe.md; docs/development/BUILD_DEPLOY_RECIPE.md (drift references)
  - https://developers.cloudflare.com/changelog/post/2026-04-13-containers-sandbox-ga/
  - https://developers.cloudflare.com/changelog/product/containers/
  - https://developers.cloudflare.com/containers/platform-details/limits/
  - https://developers.cloudflare.com/containers/pricing/
  - https://developers.cloudflare.com/containers/container-package/
  - https://developers.cloudflare.com/containers/configuration/scheduling-policy/
  - https://developers.cloudflare.com/containers/guides/migrate-to-durable-object-scheduling-policy/
  - https://developers.cloudflare.com/containers/platform-details/image-management/
  - https://developers.cloudflare.com/containers/platform-details/rollouts/
  - https://developers.cloudflare.com/containers/scaling-and-routing/
  - https://developers.cloudflare.com/containers/faq/
  - https://developers.cloudflare.com/containers/get-started/
  - https://developers.cloudflare.com/containers/platform-details/architecture/
  - https://developers.cloudflare.com/workers/platform/limits/
  - https://developers.cloudflare.com/durable-objects/reference/durable-objects-migrations/
  - https://developers.cloudflare.com/changelog/post/2026-07-09-restrict-new-kv-backed-namespaces/
  - https://developers.cloudflare.com/workers/wrangler/configuration/
  - https://developers.cloudflare.com/workers/wrangler/commands/containers
  - https://developers.cloudflare.com/workers/static-assets/headers/
  - https://developers.cloudflare.com/workers/static-assets/billing-and-limitations/
  - https://developers.cloudflare.com/workers/static-assets/migration-guides/migrate-from-pages/
  - https://developers.cloudflare.com/workers/platform/infrastructure-as-code/
  - https://developers.cloudflare.com/workers/ci-cd/builds/configuration/
  - https://developers.cloudflare.com/cf/wrangler/
  - https://developers.cloudflare.com/observability/pricing/
  - https://blog.cloudflare.com/faster-agent-sandboxes/
  - https://blog.cloudflare.com/containers-cross-tenant-vulnerability/
  - https://blog.cloudflare.com/cloudflare-cf-cli-launch/
  - https://blog.cloudflare.com/full-stack-development-on-cloudflare-workers/
  - https://blog.cloudflare.com/deno-joins-cloudflare/
  - https://deno.com/blog/cloudflare
  - https://github.com/cloudflare/terraform-provider-cloudflare/releases
  - https://github.com/cloudflare/terraform-provider-cloudflare/blob/main/docs/resources/worker_version.md
  - https://github.com/cloudflare/terraform-provider-cloudflare/blob/main/docs/resources/workers_script.md
  - https://github.com/cloudflare/terraform-provider-cloudflare/blob/main/docs/guides/version-5-upgrade.md
  - https://developers.cloudflare.com/changelog/post/2025-10-09-assets-terraform/
  - https://github.com/cloudflare/containers/blob/main/CHANGELOG.md
  - https://github.com/cloudflare/workers-sdk/issues/9898
  - https://github.com/cloudflare/developer-platform/issues/26
  - https://alchemy.run/blog/2026-07-01-microvm-cold-starts/
  - https://www.bleepingcomputer.com/news/security/cloudflare-fixes-containers-cross-tenant-flaw-exposing-customer-data/
  - https://news.ycombinator.com/item?id=49909470
  - https://news.ycombinator.com/item?id=50019911
  - npm registry (wrangler, @cloudflare/containers, @cloudflare/workers-types versions and publish dates, read with `npm view` on 2026-10-12)
---

# mx Cloudflare Deploy (As Implemented)

Companion to `mx-app-playbook.md` (§build: `mx-recipes-and-build.md`). This documents what the Cloudflare path **actually does today**, because two shipped docs describe flows that don't exist — follow this, not `INFRA_CONFIG.md`'s `mx cf` examples (§5). All repo citations are repo-relative paths in the mech-crate repo; platform claims cite Cloudflare's own pages.

*Refreshed 2026-10-12.* §1–§7 are the July 2026 research with corrections marked inline; §8–§11 are new (platform state, the second deploy path, Terraform, tooling horizon); §12 holds the inferred recommendations. The headline since July: Cloudflare Containers went GA in April, the `Container` base class every mx worker extends is maintained only through 2026-12-31, Durable Objects are now declared with `exports` instead of `migrations`, and the template still pins a wrangler major that has no `containers` command.

## 1. The model: Worker + Container via Durable Object

Cloudflare deploys are Worker-fronted containers. *(July text said "CF Containers beta"; Containers and Sandboxes have been generally available on the Workers Paid plan since 2026-04-13 [https://developers.cloudflare.com/changelog/post/2026-04-13-containers-sandbox-ga/; plan requirement on https://developers.cloudflare.com/containers/pricing/].)* The generated worker [templates/infra/cloudflare/containers-worker/src/index.ts]:
- `ContainerDO extends Container` — `defaultPort 8080`, `sleepAfter '10m'`, env injection.
- Control plane: `/_health|/_healthz|/_readyz` (container state), `/_container/restart|start|status|stop` (lifecycle).
- **Everything else proxies into the container**: `return container.fetch(request)` [index.ts:75].

`wrangler.toml` binds it together: `[[containers]]` (name, `class_name = "ContainerDO"`, `image = registry.cloudflare.com/<acct>/<app>:v<ver>`, `max_instances`), `[[durable_objects.bindings]]` (`APP_CONTAINER`), sqlite migration, `[env.preview]`/`[env.production]` with `routes = [{pattern, zone_name}]` for custom domains [cf-init-app.sh:769-849].

**What the platform calls this shape now (2026-10):**
- It is the **`default` scheduling policy**: image and instance size fixed in wrangler config, changes applied by rollouts, meant for "services whose instances use the same image and instance size". A second policy, **`durable_object`** (public beta since 2026-09-30), lets Durable Object code pick image and instance type per start via `ctx.container.start()`; it is for sandboxes and agent environments. "The scheduling policy is immutable. To use a different policy, create a new Container application." [https://developers.cloudflare.com/containers/configuration/scheduling-policy/]
- The `@cloudflare/containers` `Container` class that `ContainerDO` extends "does not support applications that use the durable_object scheduling policy. It is available only with the default scheduling policy." [https://developers.cloudflare.com/containers/container-package/] And Cloudflare's 2026-09-30 post: "We'll maintain the Container class and legacy Sandbox class through December 31, 2026. Existing deployments keep running after that date, but the classes won't get updates. We recommend migrating to ctx.container." The faster scheduling path, runtime image and instance selection, and filesystem snapshots "are available only through ctx.container" [https://blog.cloudflare.com/faster-agent-sandboxes/]. The package's last release is 0.3.7 on 2026-06-04 [npm].
- Durable Objects are now declared with an **`exports` block** (`"exports": { "ContainerDO": { "type": "durable-object", "storage": "sqlite" } }`) rather than the `migrations` array the template emits. "The exports field replaces the imperative migrations array used by older Workers. Both flows are supported, but a Worker can only use one at a time." and "Durable Object entries in exports are mutually exclusive with migrations. A Worker configuration that contains both is rejected at validation." [https://developers.cloudflare.com/durable-objects/reference/durable-objects-migrations/] The template's `new_sqlite_classes` migration still works; what no longer works on fresh accounts is a KV-backed namespace: "Creating new key-value backed Durable Object namespaces is no longer supported on this account." [https://developers.cloudflare.com/changelog/post/2026-07-09-restrict-new-kv-backed-namespaces/]
- `[[containers]]` fields on the default policy, per the configuration reference: `image` (required; a Dockerfile path or a registry reference), `instance_type` (default `"lite"`), `max_instances` (default 20, "only enforced in production"), `scheduling_policy` (default `"default"`), `rollout_step_percentage`, `rollout_active_grace_period`, `image_build_context`, `image_vars`, `constraints`, `observability`. `class_name` may be omitted when the container is linked from `exports.<Class>.container`. [https://developers.cloudflare.com/workers/wrangler/configuration/]

## 2. Scaffolding: what `mx new --infra cloudflare` gives you

- Copies `templates/infra/cloudflare/` → `infra/cloudflare/` (README, `apps/.gitkeep`, and a standalone `containers-worker/` example) with `{{PROJECT_NAME}}` substituted in `.ts/.toml/.json/.md` [crates/mx-cli/src/commands/new.rs:388-471].
- Keeps `make/cloudflare.mk` (deleted when CF not selected) and always ships `scripts/cf-setup.sh` + `scripts/cf-init-app.sh` [new.rs:91-97, 187-210].
- **Terraform is NOT scaffolded** — no `*.tf` exists anywhere in the repo (a `find` for `*.tf`, `*.tfvars*` on 2026-10-12 returns nothing) despite `new.rs:274` gitignoring `infra/**/.terraform/` and both `appendix-build-deploy-recipe.md` and `BUILD_DEPLOY_RECIPE.md` describing `make terraform-*` / `gpu-*` targets. The standalone `reference/gnn/infra/terraform/gpu-node/` example the July text pointed at is gone too. The only terraform words left outside docs are two commented-out prompts in `bin/lib/new.sh:65-68` ("Include AWS via Terraform?"). §10 covers what Terraform could own if it were added.
- **Pins are stale** (verified against npm on 2026-10-12): `containers-worker/package.json` pins `wrangler ^3.88.0`, `@cloudflare/containers ^0.0.30`, `@cloudflare/workers-types ^4.20241112.0` and `compatibility_date = "2024-11-16"`; `cf-init-app.sh:959-963` pins `wrangler ^3.99.0` and `@cloudflare/containers ^0.0.30` with `compatibility_date = "2024-12-01"`. Current: wrangler 4.149.0 (the 3.x line ends at 3.114.17), `@cloudflare/containers` 0.3.7, workers-types 5.20261012.1. Running both majors: `npx wrangler@4.149.0 containers --help` lists `list, info, instances, delete, ssh, build, push, registries, images`; `npx wrangler@3.114.17 containers --help` prints the generic command list because **wrangler 3 has no `containers` command at all**. See §6 items 9–11 for what that does to the dev loop.

The shipped `containers-worker/` is an **example**, never touched by tooling; real apps are generated under `infra/cloudflare/apps/<app>/` (§3).

## 3. The working flow, front to back

```bash
make cf-setup                 # wizard → wrangler login, account id, writes infra/cloudflare/.env.cloudflare
make cf-init a=myapp type=container   # or: worker | cron
# edit docker/dockerfiles/myapp/app to build YOUR service (see §4)
make cf-deploy a=myapp        # = cf-build → cf-push → cf-sync-image → wrangler deploy --env production
```

- `cf-setup` [templates/scripts/cf-setup.sh]: `wrangler login` if needed, detects/prompts the account id, optional API token, writes **project-local** `infra/cloudflare/.env.cloudflare` (gitignored) carrying `CLOUDFLARE_ACCOUNT_ID` + `CF_DOCKER_PLATFORM` (+ `CLOUDFLARE_API_TOKEN` when given). It is optional: one `mx infra setup cloudflare` per workstation is enough for every project on it, and `make cf-setup` is the per-project override (§5).
- `cf-init` [templates/scripts/cf-init-app.sh]: generates `infra/cloudflare/apps/<app>/` (worker `src/index.ts`, `wrangler.toml`, `package.json`, `tsconfig.json`) + a skeleton Dockerfile at `docker/dockerfiles/<app>/app` + `docker/.config/.env.<app>`. Three types: `worker` (plain fetch), `cron` (scheduled + KV state — KV namespace id left as TODO to create), `container` (§1 model).
- Deploy pipeline [templates/make/cloudflare.mk:186-258]: `cf-build` (`docker buildx build --platform linux/amd64 --build-arg APP_VERSION=<package.json version> --load`) → `cf-push` (`npx wrangler containers push`) → `cf-sync-image` (rewrites `image =` in wrangler.toml) → `wrangler deploy --config … --env production`. `cf-deploy-all` loops every `infra/cloudflare/apps/*/` with a wrangler.toml. Dev loop: `cf-dev`; observability: `cf-logs`, `cf-status`, `cf-container-status`.
- This is the **only** mx build path that stamps the released `package.json` version into the image (`v<APP_VERSION>` tags; see `mx-recipes-and-build.md` §5).

**What the platform does with that deploy (2026-10):**
- The pre-built-image route is still supported: "`wrangler containers push <IMAGE>:<TAG>`" lands the image at `registry.cloudflare.com/<ACCOUNT_ID>/<IMAGE>:<TAG>`. The alternative mx does not use: "When running `wrangler deploy`, if you set the `image` attribute in your Wrangler configuration to a path to a Dockerfile, Wrangler will build your container image locally using Docker, then push it to a registry run by Cloudflare." [https://developers.cloudflare.com/containers/platform-details/image-management/] Either way the image must fit the instance's disk, and total image storage is 50 GB per account [https://developers.cloudflare.com/containers/platform-details/limits/]; `wrangler containers images delete` frees space but "Reverting a Worker to a previous version that uses a deleted image will then error" [image-management].
- A deploy starts a **rollout**, it does not finish one. Default steps: with `max_instances` ≥ 2, "about 10% of container instances" first, then "100%"; with fewer, a single 100% step; override with `rollout_step_percentage`. Each replaced instance gets "SIGTERM to the main process", up to 15 minutes to exit, then "SIGKILL", then the Worker's `onStop` hook runs, then the new image starts. `wrangler deploy --containers-rollout=immediate` does it in one step; `--containers-rollout=none` deploys only Worker code and leaves containers untouched. [https://developers.cloudflare.com/containers/platform-details/rollouts/] The mx `cf-deploy` target passes neither flag, so every production deploy is a gradual rollout even when only the worker changed.
- Scaling is still manual: Cloudflare's own docs route by explicit Durable Object IDs or `getRandom` across N instances and say autoscaling and routing are planned, not shipped [https://developers.cloudflare.com/containers/scaling-and-routing/]. The mx worker pins one instance id (`CONTAINER_INSTANCE_ID`, default `<app>-ssr`) and so runs one container per environment regardless of `max_instances` [index.ts:28-31; wrangler.toml `[vars]`].

## 4. Onboarding an existing `apps/<svc>` service

There is no automated bridge — `cf-init` always scaffolds a fresh skeleton [cf-init-app.sh:125, 853-903]. The astro recipe ships its own production Dockerfile labelled "Optimized for Cloudflare Container deployments" (`node:22-alpine`, multi-stage) [templates/recipes/astro/docker/dockerfiles/app.prod.Dockerfile:1-14], and the cupcake repo profile is a live example of an mx monorepo running two containers behind Workers plus KV, R2 and Queues [docs/development/repos/cupcake.md].

### Synthesis (inferred)

The procedure that follows from the mechanics:
1. `make cf-init a=<svc> type=container` to generate the worker + wrangler.toml + Dockerfile skeleton.
2. Replace the skeleton `docker/dockerfiles/<svc>/app` with a Dockerfile that builds your actual `apps/<svc>` source (build context is repo root — reuse your recipe's production stage as the base). `cf-build` uses exactly this path [cloudflare.mk:21, 186-197].
3. Align ports: the worker's `ContainerDO.defaultPort` and the Dockerfile's `EXPOSE`/`PORT` must match your app's listen port; also make your app serve `/health` — the generated Dockerfile HEALTHCHECK hits `/health` on the app directly, NOT the worker's `/_health` routes [cf-init-app.sh:900 vs index.ts:3].
4. Set real `routes` in `[env.production]` (pattern + zone) — the defaults are placeholders.
5. Keep local dev on the mx router; Cloudflare is the production path. The two coexist: same Dockerfile family, different front doors.
6. Pick an `instance_type` on purpose. The template sets none, so the app runs on `lite` (1/16 vCPU, 256 MiB, 2 GB disk; §8). A Node SSR server plus its image has to fit that disk and memory, and "Containers do not use swap memory" (§8); `basic` or `standard-1` is the realistic floor for anything that renders pages.

## 5. Credentials: one variable name, two scopes

*Rewritten 2026-09-25 against the shipped code (bd:mech-crate-wd9). Before that fix this section described a two-name, one-scope split that made `mx infra setup cloudflare` credentials invisible to every `make cf-*` target. That is gone; §6 items 2 and 4 record what it was.*

Two scopes, consistently located, and both now read [crates/mx-lib/src/infra/config.rs:84-97; crates/mx-lib/src/config.rs:54-57]:

| Scope | File | Written by |
|---|---|---|
| global | `~/.mech-crate/config/infra/cloudflare.env` | `mx infra setup cloudflare` |
| project | `infra/cloudflare/.env.cloudflare` | `make cf-setup` |

**One canonical pair: `CLOUDFLARE_ACCOUNT_ID` + `CLOUDFLARE_API_TOKEN`.** Those are the two names wrangler itself reads from the environment, so a value in a credentials file reaches the deploy with no translation step. Every writer emits exactly those names and nothing else [crates/mx-cli/src/commands/infra.rs; templates/scripts/cf-setup.sh]. `CF_ACCOUNT_ID` / `CF_API_TOKEN` survive as **deprecated aliases that consumers read and no writer emits**, so an `.env.cloudflare` produced by an older `make cf-setup` keeps working and the toolchain says so rather than failing.

**Resolution order,** highest precedence first, implemented the same way in `cloudflare.mk` [cloudflare.mk:16-78] and `cf-init-app.sh` [cf-init-app.sh:95-165]:

1. the environment, or the `make` command line
2. project `infra/cloudflare/.env.cloudflare`
3. global `~/.mech-crate/config/infra/cloudflare.env`

Each scope is captured on its own, which is the part worth knowing: a project file carrying **only** the deprecated `CF_ACCOUNT_ID` still beats a global file carrying the canonical `CLOUDFLARE_ACCOUNT_ID`. Precedence is by scope, never by spelling. Global-only is a complete configuration: `make cf-init`, the deploy chain and the registry image path all resolve from it.

**Inspect it, do not guess:**
- `make cf-vars` prints the resolved account id, `CLOUDFLARE_ACCOUNT_ID_SOURCE` (`environment` | `project` | `global` | `none`), whether a token is configured (masked, never printed), both file paths, and a `DEPRECATED_CF_ACCOUNT_ID_IN=` line naming any file still using the old spelling.
- `make cf-check-credentials` is a prerequisite of the credential-consuming targets. With no account id in any scope it exits non-zero naming both searched paths and both remedies (`mx infra setup cloudflare` for the global config, `make cf-setup` for this project's). Nothing runs on an empty account id, so no deploy can address `registry.cloudflare.com//<app>`.

Token absent from both files is fine: wrangler falls back to its OAuth login.

**What a token needs.** Cloudflare publishes no "Containers" permission scope; the permissions reference lists none, and the set Workers Builds creates for itself is Account "Account Settings (read), Workers Scripts (edit), Workers KV Storage (edit), Workers R2 Storage (edit)", Zone "Workers Routes (edit)", User "User Details (read), Memberships (read)" [https://developers.cloudflare.com/workers/ci-cd/builds/configuration/]. The docs site workflow in this repo gets by with Workers Scripts:Edit + Workers Routes:Edit + Zone:Read for an assets-only Worker with a custom domain [.github/workflows/site.yml:22-27]. Two long-running issue threads find that a 403 / "Unauthorized" at the registry push step is the Workers **Free** plan check, not a missing token scope [https://github.com/cloudflare/workers-sdk/issues/9898; https://github.com/cloudflare/developer-platform/issues/26] — discovery-grade, but consistent with the pricing page's plan requirement.

**Still broken (verified 2026-10-12; do not send agents down these):**
- `mx infra link`/`unlink` are stubs (print-only) [infra.rs:327-351], and the two link mechanisms that do exist disagree: the Rust resolver looks for a `.env.linked` marker file [config.rs:117] while the bash scripts look for `MX_INFRA_USE_GLOBAL=true` **inside** the env file [cf-init-app.sh:120]. Neither is ever written by tooling. This matters much less now: the global fallback covers what linking was for, so the common case needs no link at all.
- **`mx cf <anything>` does not exist.** No `Cf` variant in the CLI command enum [crates/mx-cli/src/main.rs:34-151]; `INFRA_CONFIG.md` (`mx cf setup/config/deploy`, lines 32, 132, 173, 176, 243) and the MCP server's resource text both document a phantom command. It is now tracked as a known-broken test, `bd:mech-crate-vxq` [crates/mx-cli/tests/known_broken.rs:154-168].

**Agent rule:** one `mx infra setup cloudflare` per workstation covers every project on it. Add `make cf-setup` only when a single project needs a different account than the rest. When credentials look wrong, run `make cf-vars` first: it names the scope that answered, and that is usually the whole diagnosis.

## 6. Drift & gaps inventory (items 2 and 4 closed 2026-09-25; items 9–16 added 2026-10-12)

1. `mx cf` documented in INFRA_CONFIG.md + MCP resources, unimplemented in the CLI. **Still open**, tracked as bd:mech-crate-vxq.
2. **Closed 2026-09-25** (bd:mech-crate-wd9). Was: `CLOUDFLARE_ACCOUNT_ID` (mx infra) vs `CF_ACCOUNT_ID` (toolchain) mismatch, which made global credentials unusable by deploys. Now one canonical pair, with the old spelling read as a deprecated alias (§5).
3. `mx infra link/unlink` stubs; `.env.linked` marker vs `MX_INFRA_USE_GLOBAL` in-file flag — two incompatible, both unwired. **Still open.**
4. **Closed 2026-09-25** (same change). Was: `cloudflare.mk` had no global-credential fallback, including the project file only. Now it reads global then project, so the project wins per scope and a global-only workstation deploys (§5).
5. Terraform: gitignored, documented in two appendices, never scaffolded; no `make terraform-*` targets exist; the reference example has since been removed as well. **Still open** (§10 says what it could own).
6. `appendix-build-deploy-recipe.md` and `BUILD_DEPLOY_RECIPE.md` prescribe `infra/dockerfiles/` + `containers-worker/` layout; shipped reality is `docker/dockerfiles/<app>/app` + `infra/cloudflare/apps/<app>/` — treat the appendices as aspirational, this doc as ground truth.
7. Generated Dockerfile HEALTHCHECK path (`/health`) is the app's responsibility, not the worker's `/_health` control route — mismatched expectations if the app doesn't serve it.
8. Cron type ships `kv_namespaces` id `TODO_CREATE_KV_NAMESPACE` — deploy fails until the namespace is created and the id pasted in.
9. **wrangler major pinned at 3.x** in both the example (`^3.88.0`) and the generator (`^3.99.0`), while 3.x has no `containers` command (§2). `cf-dev` runs `npm run dev` inside the app dir, so the dev loop uses the pinned 3.x binary and cannot be expected to understand `[[containers]]`.
10. **`npx wrangler` for push and deploy runs from the repo root** [cloudflare.mk:204, 244, 256], where no `package.json` or `node_modules` exists in the template, so the version that pushes and deploys is whatever `npx` finds on the workstation or downloads — not the pinned one. *Inferred from the Makefile and npx's resolution rules; not reproduced end to end.* Deploys work in practice because an unpinned npx resolves to 4.x.
11. `@cloudflare/containers ^0.0.30` and `compatibility_date` 2024-11/2024-12 in generated projects; the package is at 0.3.7 and Cloudflare says to set the compatibility date to today at project creation [https://developers.cloudflare.com/workers/wrangler/configuration/].
12. **The `Container` base class is on a clock**: maintained through 2026-12-31, no updates after, migration to `ctx.container` recommended (§1). Every generated worker extends it.
13. Durable Object declared via `migrations` / `new_sqlite_classes` rather than `exports`; still accepted, but the two are mutually exclusive and `exports` is the documented path forward (§1).
14. No `instance_type` → every generated container runs on `lite` (256 MiB); `max_instances` 5 (example) / 3 (generator prod) is meaningless while the worker pins one instance id (§3, §8).
15. No `scheduling_policy` written explicitly; today's default is `"default"`, which is the only policy the `Container` class supports and is immutable once created (§1).
16. The repo's own docs site deploys through a **second, undocumented-by-mx path** (assets-only Worker, `wrangler.jsonc`, GitHub Actions) that shares nothing with `cloudflare.mk` (§9).

## 7. Agent quick-reference

- CF flow = `make cf-setup` → `make cf-init a=<app> type=container` → edit the app Dockerfile → `make cf-deploy a=<app>`. Never `mx cf` (doesn't exist).
- Credentials the deploys read: `CLOUDFLARE_ACCOUNT_ID` (+ optional `CLOUDFLARE_API_TOKEN`), resolved from the environment, then `infra/cloudflare/.env.cloudflare`, then `~/.mech-crate/config/infra/cloudflare.env`. `make cf-vars` prints which scope answered; `make cf-check-credentials` is the guard that fails loudly when none did. `CF_ACCOUNT_ID` is a deprecated alias: read, warned about, never written.
- Production URL comes from `[env.production] routes` in the app's wrangler.toml; local dev URLs still come from the mx router.
- The Worker proxies everything to the container; container image versions come from `package.json` (`v<version>`), unlike every other mx build path.
- Before the first push: Workers Paid plan, Docker image built for `linux/amd64` ("Containers should be built for the linux/amd64 architecture" [https://developers.cloudflare.com/containers/platform-details/architecture/]; the Makefile already passes `--platform`), image smaller than the instance disk, under 50 GB of images on the account.
- After `wrangler deploy`: the rollout is still running; `wrangler containers info <id>` / `make cf-container-status` tells you when the new image is actually serving. Worker-only change: add `--containers-rollout=none` by hand.
- Static site, no server routes: do not use this flow at all; use the assets-only Worker shape in §9.

## 8. Platform state as of 2026-10-12

**Status and plan.** "Containers and Sandboxes are now generally available" (changelog, 2026-04-13); the pricing page states the requirement as the "$5 USD per month Workers Paid plan". Milestones on the way there, from the product changelog: active-CPU pricing 2025-11-21, custom instance types GA 2026-01-05, SSH via wrangler 2026-03-12, Docker Hub images 2026-03-24, regional placement constraints 2026-04-05, `exec()` 2026-06-18, per-container `observability` 2026-09-08, `durable_object` scheduling policy and snapshots 2026-09-30 [https://developers.cloudflare.com/changelog/product/containers/].

**Instance types and account limits** [https://developers.cloudflare.com/containers/platform-details/limits/]:

| Type | vCPU | Memory | Disk |
|---|---|---|---|
| lite (default) | 1/16 | 256 MiB | 2 GB |
| basic | 1/4 | 1 GiB | 4 GB |
| standard-1 | 1/2 | 4 GiB | 8 GB |
| standard-2 | 1 | 6 GiB | 12 GB |
| standard-3 | 2 | 8 GiB | 16 GB |
| standard-4 | 4 | 12 GiB | 20 GB |

Account defaults: 6 TiB concurrent memory, 1,500 concurrent vCPU, 30 TB concurrent disk, image size "same as instance disk space", 50 GB total image storage; snapshots 20 GB and 30 days. "All instance size maximums and account limits on this page are defaults and can be increased."

**Pricing** [https://developers.cloudflare.com/containers/pricing/]: included per month on Workers Paid — 25 GiB-hours memory, 375 vCPU-minutes, 200 GB-hours disk; beyond that $0.0000025 per GiB-second, $0.000020 per vCPU-second (active CPU), $0.00000007 per GB-second. Egress $0.025/GB in North America and Europe (1 TB included), $0.05/GB Oceania/Korea/Taiwan and $0.04/GB elsewhere (500 GB included). Billing starts "when a request is sent to the container or when it is manually started" and stops "after the container instance goes to sleep"; Workers and Durable Objects usage is billed on top. Container stdout/stderr go to Workers Logs when `observability` is on, "retained for seven days", and "Beginning December 1, 2026, Container logs use Cloudflare Observability pricing" [https://developers.cloudflare.com/containers/faq/; https://developers.cloudflare.com/observability/pricing/].

**Cold starts, two measurements.** Under the new `durable_object` scheduling path Cloudflare reports ComputeSDK's Burst TTI benchmark moving from a median of 4.049 s to 648 ms and p95 from 5.839 s to 910 ms, and "a single account started 100,000 Containers in 5.387 seconds across six locations" [https://blog.cloudflare.com/faster-agent-sandboxes/]. That path is not what the mx default-policy worker gets. For the default path, Alchemy's 2026-07-01 benchmark (700 boots, time to first successful application response) measured a 100 MB opencode image at p50 10.5 s, p95 15.6 s, max 58.6 s on Cloudflare versus 3.4 / 4.1 / 4.5 s on AWS microVMs, and a hello-world container at p50 1.7 s but p95 23.9 s; the authors call the long tail "a property of the platform, not the workload" [https://alchemy.run/blog/2026-07-01-microvm-cold-starts/]. Cloudflare's FAQ puts typical starts "in the 1-3 second range", notes "Containers do not use swap memory" (OOM restarts the instance), and says end users cannot open non-HTTP TCP or UDP connections to an instance [https://developers.cloudflare.com/containers/faq/].

**Security incident worth knowing.** Cloudflare's 2026-09-24 disclosure: the dm-thin storage pool backing container root disks ran with `skip_block_zeroing`, so when "the thin volume backing a container's root disk was deleted, its physical blocks were returned to a pool that served workloads belonging to multiple customer accounts" and a later tenant could read residue ("filesystem metadata, directory structures, database pages, and application data"). Reported 2026-09-04 15:26 UTC by Oren Yomtov, fix merged 21:27 UTC the same day, fleet rollout complete 2026-09-07, pre-mitigation disks and image snapshots retired by 2026-09-19; no evidence of exploitation beyond the researchers and Cloudflare's own tests; no customer action required [https://blog.cloudflare.com/containers-cross-tenant-vulnerability/; https://www.bleepingcomputer.com/news/security/cloudflare-fixes-containers-cross-tenant-flaw-exposing-customer-data/]. The lesson for anyone putting secrets on a container disk: treat the root disk as ephemeral and shared-hardware, keep secrets in Worker bindings or the Secrets Store, and do not write them to disk inside the container.

## 9. The second path: an assets-only Worker for mechcrate.dev

The repo's own docs site does not use `cloudflare.mk` at all. `site/apps/site/wrangler.jsonc` has an `assets` block and **no `main`**: `"directory": "./dist"`, `"html_handling": "auto-trailing-slash"`, `"not_found_handling": "404-page"`, and a `routes` entry with `"custom_domain": true` for the apex, which fails loudly ("Could not find zone for mechcrate.dev") rather than half-deploying when the zone is missing [site/apps/site/wrangler.jsonc]. `.github/workflows/site.yml` builds, link-checks and deploys it independently of the Rust CI, and skips with a `::notice` until the two repository secrets exist; note the org secret is named `CLOUDFLARE_API_KEY` and is exported to wrangler as `CLOUDFLARE_API_TOKEN` [site.yml:14-36]. Response headers live in `site/apps/site/public/_headers` (immutable cache on `/_astro/*`, HSTS without `preload`, nosniff, referrer policy, explicit `text/plain` on the llms artefacts) and the workflow re-proves them against a local workerd before deploying [public/_headers; site.yml:52-60].

Why this is the right shape for a static bundle, in Cloudflare's words: "Now that Workers supports both serving static assets and server-side rendering, you should start with Workers." and "Cloudflare Pages will continue to be supported, but, going forward, all of our investment, optimizations, and feature work will be dedicated to improving Workers." [https://blog.cloudflare.com/full-stack-development-on-cloudflare-workers/, 2025-04-08]. `_headers` and `_redirects` are honoured by Workers static assets with the same limits the repo's `_headers` comment reads out of the wrangler source — 100 rules and 2,000 characters per line — but neither applies to responses a Worker script generates, which matters the moment `run_worker_first` routes a path to code [https://developers.cloudflare.com/workers/static-assets/headers/]. "Requests to static assets are free and unlimited" and storing them costs nothing [https://developers.cloudflare.com/workers/static-assets/billing-and-limitations/]; the caps are 20,000 files per Worker version on Free, 100,000 on Paid, 25 MiB per file, and the `_headers` figures above (100 rules, 2,000 characters per line) come from the same table [https://developers.cloudflare.com/workers/platform/limits/]. Cloudflare "recommends using `wrangler.jsonc` for new projects, and some newer Wrangler features will only be available to projects using a JSON config file" [https://developers.cloudflare.com/workers/wrangler/configuration/], which the site follows and the container generator (TOML) does not.

## 10. Terraform: what it can own and what it cannot

The Cloudflare provider is on the v5 line (v5.27.0, 2026-10-03; v4 receives backports only) with a `tf-migrate` tool for the v4→v5 resource renames [https://github.com/cloudflare/terraform-provider-cloudflare/releases; https://developers.cloudflare.com/changelog/product/terraform/]. The v5 upgrade guide renames `cloudflare_worker_script` → `cloudflare_workers_script`, `cloudflare_worker_route` → `cloudflare_workers_route`, `cloudflare_worker_domain` → `cloudflare_workers_custom_domain`, drops `cloudflare_workers_secret`, and says `cloudflare_workers_script` is no longer the recommended way to ship code: use `cloudflare_worker` + `cloudflare_worker_version` + `cloudflare_workers_deployment` [https://github.com/cloudflare/terraform-provider-cloudflare/blob/main/docs/guides/version-5-upgrade.md]. Static assets are expressible (`assets = { directory = … }` plus headers/redirects config) since provider 5.11.0 [https://developers.cloudflare.com/changelog/post/2025-10-09-assets-terraform/].

**Containers are attach-only in Terraform.** The `cloudflare_worker_version` schema's `containers` attribute is a list of objects whose only field is `class_name` ("Select which Durable Object class should get this container attached"), `cloudflare_workers_script` has no `containers` attribute, and the bindings `type` enum has `durable_object_namespace` but no container type [https://github.com/cloudflare/terraform-provider-cloudflare/blob/main/docs/resources/worker_version.md; …/workers_script.md]. There is no resource for the container application itself — image, instance type, `max_instances`, rollouts — so the thing mx deploys cannot be modelled in Terraform at all; it stays with wrangler (or the raw API). Cloudflare's IaC page draws the line the same way: own the Worker shell (`cloudflare_worker`), bindings and routes in Terraform and let wrangler or your own tooling handle versions and deployments, and if you upload code through Terraform copy every setting out of the wrangler config yourself, since bundling is not something IaC does [https://developers.cloudflare.com/workers/platform/infrastructure-as-code/]. Pulumi's provider is bridged from the Terraform one and inherits the same gap.

### Synthesis (inferred)

If Terraform is ever scaffolded for the mx Cloudflare path, the useful split is: Terraform owns the zone, DNS, the `cloudflare_worker` shell per app, `cloudflare_workers_custom_domain` / routes, KV and R2 namespaces and the API token policy; `cloudflare.mk` keeps owning image build, push, `[[containers]]`, the Durable Object `exports`, and `wrangler deploy`. The Durable Object class must exist in a deployed version before a Terraform binding can reference it, which is the one ordering constraint Cloudflare's IaC page calls out, so the first apply precedes the first `cf-deploy` and the binding is added on the second.

## 11. Tooling horizon: cf CLI, wrangler, Deno

- **`cf` CLI, open beta since 2026-09-28**, with a TypeScript configuration format (`cloudflare.config.ts`): "When the open beta ends we will release a final major version of Wrangler that directs you and your agent to use cf." and "We'll continue to provide maintenance support for Wrangler for 18 months after the beta ends, to give you time to migrate." [https://blog.cloudflare.com/cloudflare-cf-cli-launch/]. No beta end date is announced; `cf` is at 1.0.0-beta.14 (2026-10-09) on npm. Its docs say `cf migrate` converts wrangler config but Durable Objects, Workflows and Containers need manual adjustment [https://developers.cloudflare.com/cf/wrangler/]. Wrangler itself is still 4.x (4.149.0 on 2026-10-08); there is no wrangler 5.
- **Deno joined Cloudflare on 2026-10-09.** Cloudflare's side: the Deno team will "lead a new effort to make workerd self-hosting a first-class supported way to build and run apps using the Workers programming model", "merging code and ideas from celld back into workerd" [https://blog.cloudflare.com/deno-joins-cloudflare/]. Deno's side: "We will support the Deno runtime for another year with monthly releases containing bug fixes and security updates. After that year we will end our development of the Deno runtime."; "Deno Deploy will continue operating for six months before shutting down"; JSR continues [https://deno.com/blog/cloudflare]. Nothing changes for existing Workers deployments; for mx the relevant promise is self-hostable workerd, a possible third front door for the same container image.

## 12. Synthesis (inferred): what the mx generator should change

Ordered by how much each closes, cheapest first; every item traces to a cited fact above.

1. **Bump the pins** in `containers-worker/package.json` and `cf-init-app.sh`: wrangler `^4`, `@cloudflare/containers` `^0.3`, workers-types `^5`, `compatibility_date` = generation date. This alone makes `cf-dev` a wrangler that knows what a container is (§6 items 9–11).
2. **Run the pinned wrangler.** Make `cf-push`/`cf-deploy` call the app directory's `node_modules/.bin/wrangler` (or `npm exec --prefix`) rather than `npx wrangler` from the repo root, so dev and deploy use the same binary (§6 item 10).
3. **Emit `exports` instead of `migrations`** for new apps and never both; keep `new_sqlite_classes` only for projects already deployed with it (§1).
4. **Write `scheduling_policy = "default"` and an explicit `instance_type`** (default `basic` or `standard-1` for SSR services, `lite` only for tiny proxies) into generated configs (§6 items 14–15, §8).
5. **Expose rollout control**: `cf-deploy` passes `--containers-rollout=immediate` for preview and leaves gradual for production; add a `cf-deploy-worker-only` target that passes `--containers-rollout=none` (§3).
6. **Plan the `Container` class exit** before 2026-12-31: the generated `ContainerDO` uses `defaultPort`, `sleepAfter`, `envVars`, `getState`, `startAndWaitForPorts`, `stop`, `destroy`, `fetch`, all of which the native `ctx.container` API covers per the migration guide; a template that extends `DurableObject` and calls `this.ctx.container.start(...)` keeps the default policy and the same wrangler config (§1).
7. **Emit `wrangler.jsonc`**, as the site already does, and keep one eye on `cloudflare.config.ts` — `cf migrate` will not carry Containers across by itself (§9, §11).
8. **Add a static-site type** to `cf-init` (`type=assets`): assets block, no `main`, `_headers` passthrough, `not_found_handling` chosen per framework. The docs site is the reference implementation (§9).
9. **Preflight in `cf-check-credentials`**: plan check (a registry 403 means Free plan), image size against the chosen instance disk, account image storage against 50 GB, and never `images delete` a tag a live version still references (§3, §5).
10. **Pass `observability` per container** and warn in `cf-logs` that container logs become billable under Observability pricing on 2026-12-01 (§8).
