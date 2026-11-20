---
title: "mx Cloudflare Deploy: As-Implemented Infra Flow, Credentials, and Known Traps"
category: infra
languages: [typescript]
complexity: advanced
use_cases:
  - deploying an mx app to Cloudflare Workers + Containers front to back
  - setting up Cloudflare credentials that the deploy toolchain actually reads
  - onboarding an existing apps/<svc> service to the Cloudflare container path
  - using the mx cf wrapper over the make cf-* targets, and avoiding doc drift
  - judging how far the scaffold has drifted from the GA Cloudflare Containers platform and wrangler 4, and what to change
  - deciding what Terraform would own versus wrangler in a Cloudflare deploy, and why mx scaffolds none
  - shipping a static site as an assets-only Worker (the mechcrate.dev path)
summary: "The as-implemented mx Cloudflare flow researched from source (mx new --infra scaffolding, cf-setup/cf-init onboarding, the Worker+Container runtime model, the one-pair credential model), re-checked 2026-10-26 against the GA Containers platform: wrangler 4 and Node 22 are current while the scaffold pins wrangler 3, wrangler deploy now builds and pushes the image itself, the Container class has a stated end-of-updates date, Terraform v5 can attach but not configure a container, and a 14-item drift inventory says what to change."
provenance: researched
researched: 2026-11-20
sources:
  - templates/infra/cloudflare/ (mx repo — wrangler.toml, containers-worker, README)
  - templates/scripts/{cf-setup.sh,cf-init-app.sh} (mx repo)
  - templates/make/cloudflare.mk (mx repo)
  - crates/mx-cli/src/commands/{new,infra}.rs (mx repo)
  - crates/mx-lib/src/infra/config.rs + crates/mx-lib/src/config.rs (mx repo)
  - crates/mx-cli/src/main.rs (command surface)
  - docs/development/INFRA_CONFIG.md; docs/development/appendix-build-deploy-recipe.md (drift references)
  - site/apps/site/wrangler.jsonc (mx repo; the one wrangler 4 config in the tree)
  - "external sources for the 2026-10-26 platform re-check, numbered S1-S53 in section 11"
---

# mx Cloudflare Deploy (As Implemented)

Companion to `mx-app-playbook.md` (§build: `mx-recipes-and-build.md`). This documents what the Cloudflare path **actually does today**, because `appendix-build-deploy-recipe.md` describes a layout that does not exist (§6 items 5-6); follow this doc where they disagree. `mx cf <cmd>` exists as a thin wrapper over `make cf-<cmd>` (§6 item 1). Sections 1-7 cite repo-relative files in the mech-crate repo by file and named target or function, not by line number, because line citations went stale within weeks. Sections 8-10 (added 2026-10-26) compare the scaffold with the Cloudflare platform and toolchain, citing the numbered external sources in §11. Sections 12-13 come from an older pass (2026-10-12) and cite sources inline by URL.

*Reviewed 2026-11-20* against the repo and the primary pages. The largest change since October: Cloudflare now documents the `default` scheduling policy, which the scaffold uses, as deprecated (§8.6). Version numbers below are as of that review.

## 1. The model: Worker + Container via Durable Object

Cloudflare deploys are Worker-fronted containers. Containers left beta on 2026-04-13 ("Cloudflare Containers and Sandboxes are now generally available", Workers Paid plan) [S1]; the scaffold predates that and still carries beta-era pins (§9). The generated worker [templates/infra/cloudflare/containers-worker/src/index.ts]:
- `ContainerDO extends Container` — `defaultPort 8080`, `sleepAfter '10m'`, env injection.
- Control plane: `/_health|/_healthz|/_readyz` (container state), `/_container/restart|start|status|stop` (lifecycle).
- **Everything else proxies into the container**: `return container.fetch(request)` [index.ts:75].

`wrangler.toml` binds it together: `[[containers]]` (name, `class_name = "ContainerDO"`, `image = registry.cloudflare.com/<acct>/<app>:v<ver>`, `max_instances`), `[[durable_objects.bindings]]` (`APP_CONTAINER`), sqlite migration, `[env.preview]`/`[env.production]` with `routes = [{pattern, zone_name}]` for custom domains [cf-init-app.sh, the container `wrangler.toml` and worker `index.ts` heredocs].

## 2. Scaffolding: what `mx new --infra cloudflare` gives you

- Copies `templates/infra/cloudflare/` → `infra/cloudflare/` (README, `apps/.gitkeep`, and a standalone `containers-worker/` example) with `{{PROJECT_NAME}}` substituted in `.ts/.toml/.json/.md` [crates/mx-cli/src/commands/new.rs, `setup_cloudflare_infra`].
- Keeps `make/cloudflare.mk` (deleted when CF not selected) and always ships `scripts/cf-setup.sh` + `scripts/cf-init-app.sh` [new.rs, the `has_cloudflare` removal and `copy_templates`].
- **Terraform is NOT scaffolded** — no `*.tf` exists anywhere in `templates/` despite the `.gitignore` ignoring tfstate. `appendix-build-deploy-recipe.md` documents only a DigitalOcean GPU-droplet module whose targets are `make gpu-*`. The only terraform in the repo is a standalone reference example (`reference/gnn/infra/terraform/gpu-node/`).

The shipped `containers-worker/` is an **example**, never touched by tooling; real apps are generated under `infra/cloudflare/apps/<app>/` (§3).

## 3. The working flow, front to back

```bash
make cf-setup                 # wizard → wrangler login, account id, writes infra/cloudflare/.env.cloudflare
make cf-init a=myapp type=container   # or: worker | cron
# edit docker/dockerfiles/myapp/app to build YOUR service (see §4)
make cf-deploy a=myapp        # = cf-publish (build, push) → cf-sync-image → cf-install → wrangler deploy --env production
```

- `cf-setup` [templates/scripts/cf-setup.sh]: `wrangler login` if needed, detects/prompts the account id, optional API token, writes **project-local** `infra/cloudflare/.env.cloudflare` (gitignored) carrying `CLOUDFLARE_ACCOUNT_ID` + `CF_DOCKER_PLATFORM` (+ `CLOUDFLARE_API_TOKEN` when given). It is optional: one `mx infra setup cloudflare` per workstation is enough for every project on it, and `make cf-setup` is the per-project override (§5).
- `cf-init` [templates/scripts/cf-init-app.sh]: generates `infra/cloudflare/apps/<app>/` (worker `src/index.ts`, `wrangler.toml`, `package.json`, `tsconfig.json`) + a skeleton Dockerfile at `docker/dockerfiles/<app>/app` + `docker/.config/.env.<app>`. Three types: `worker` (plain fetch), `cron` (scheduled + KV state — KV namespace id left as TODO to create), `container` (§1 model).
- Deploy pipeline [templates/make/cloudflare.mk, targets `cf-build` through `cf-deploy`]: `cf-build` (`docker buildx build --platform linux/amd64 --build-arg APP_VERSION=<package.json version> --load`) → `cf-push` (`npx wrangler containers push`) → `cf-sync-image` (rewrites `image =` in wrangler.toml) → `wrangler deploy --config … --env production`. `cf-deploy-all` loops every `infra/cloudflare/apps/*/` with a wrangler.toml. Dev loop: `cf-dev`; observability: `cf-logs`, `cf-status`, `cf-container-status`.
- This is the **only** mx build path that stamps the released `package.json` version into the image (`v<APP_VERSION>` tags; see `mx-recipes-and-build.md` §5).
- The build → push → rewrite-`image =` → deploy chain is what ships. It is no longer the shortest documented path: `wrangler deploy` builds and pushes the image itself when `image` is a Dockerfile path (§8.3), which makes `cf-sync-image` a self-inflicted step. §9 says what to change and what to keep.

## 4. Onboarding an existing `apps/<svc>` service

There is no automated bridge — `cf-init` always scaffolds a fresh skeleton [cf-init-app.sh, the "already exists" prompt and the skeleton Dockerfile heredoc].

### Synthesis (inferred)

The procedure that follows from the mechanics:
1. `make cf-init a=<svc> type=container` to generate the worker + wrangler.toml + Dockerfile skeleton.
2. Replace the skeleton `docker/dockerfiles/<svc>/app` with a Dockerfile that builds your actual `apps/<svc>` source (build context is repo root — reuse your recipe's production stage as the base). `cf-build` uses exactly this path [cloudflare.mk, `CF_APP_DOCKERFILE` and `cf-build`].
3. Align ports: the worker's `ContainerDO.defaultPort` and the Dockerfile's `EXPOSE`/`PORT` must match your app's listen port; also make your app serve `/health` — the generated Dockerfile HEALTHCHECK hits `/health` on the app directly, NOT the worker's `/_health` routes [cf-init-app.sh, the skeleton Dockerfile `HEALTHCHECK`, versus the worker's port].
4. Set real `routes` in `[env.production]` (pattern + zone) — the defaults are placeholders.
5. Keep local dev on the mx router; Cloudflare is the production path. The two coexist: same Dockerfile family, different front doors.

## 5. Credentials: one variable name, two scopes

*Rewritten 2026-09-25 against the shipped code (bd:mech-crate-wd9). Before that fix this section described a two-name, one-scope split that made `mx infra setup cloudflare` credentials invisible to every `make cf-*` target. That is gone; §6 items 2 and 4 record what it was.*

Two scopes, consistently located, and both now read [crates/mx-lib/src/infra/config.rs, `credential_keys`; crates/mx-lib/src/config.rs]:

| Scope | File | Written by |
|---|---|---|
| global | `~/.mech-crate/config/infra/cloudflare.env` | `mx infra setup cloudflare` |
| project | `infra/cloudflare/.env.cloudflare` | `make cf-setup` |

**One canonical pair: `CLOUDFLARE_ACCOUNT_ID` + `CLOUDFLARE_API_TOKEN`.** Those are the two names wrangler itself reads from the environment, so a value in a credentials file reaches the deploy with no translation step. Every writer emits exactly those credential names (`cf-setup.sh` also writes `CF_DOCKER_PLATFORM`) [crates/mx-cli/src/commands/infra.rs; templates/scripts/cf-setup.sh]. `CF_ACCOUNT_ID` / `CF_API_TOKEN` survive as **deprecated aliases that consumers read and no writer emits**, so an `.env.cloudflare` produced by an older `make cf-setup` keeps working and the toolchain says so rather than failing.

**Resolution order,** highest precedence first, implemented the same way in `cloudflare.mk` [cloudflare.mk, "Credential resolution"] and `cf-init-app.sh` ["Load Cloudflare configuration"]:

1. the environment, or the `make` command line (until 2026-11-20 `cf-init-app.sh` skipped this scope and read only the two files, so CI-style environment-only credentials passed `cf-check-credentials` and then failed in `cf-init`; fixed with tests in `crates/mx-cli/tests/infra_link.rs`)
2. project `infra/cloudflare/.env.cloudflare`
3. global `~/.mech-crate/config/infra/cloudflare.env`

Each scope is captured on its own, which is the part worth knowing: a project file carrying **only** the deprecated `CF_ACCOUNT_ID` still beats a global file carrying the canonical `CLOUDFLARE_ACCOUNT_ID`. Precedence is by scope, never by spelling. Global-only is a complete configuration: `make cf-init`, the deploy chain and the registry image path all resolve from it.

**Inspect it, do not guess:**
- `make cf-vars` prints the resolved account id, `CLOUDFLARE_ACCOUNT_ID_SOURCE` (`environment` | `project` | `global` | `none`), whether a token is configured (masked, never printed), both file paths, and a `DEPRECATED_CF_ACCOUNT_ID_IN=` line naming any file still using the old spelling.
- `make cf-check-credentials` is a prerequisite of the credential-consuming targets. With no account id in any scope it exits non-zero naming both searched paths and both remedies (`mx infra setup cloudflare` for the global config, `make cf-setup` for this project's). Nothing runs on an empty account id, so no deploy can address `registry.cloudflare.com//<app>`.

Token absent from both files is fine: wrangler falls back to its OAuth login.

**Linking** (bd:mech-crate-066). A project file containing the exact line `MX_INFRA_USE_GLOBAL=true` is linked: its values are ignored and credentials resolve from the environment, then the global file. `mx infra link cloudflare` writes that line (keeping the rest of the file), `mx infra unlink cloudflare` removes it, and `cloudflare.mk` (`CF_PROJECT_LINKED=` in `make cf-vars`), `cf-init-app.sh` and `mx infra inspect cloudflare` all read it. `mx infra inspect cloudflare` prints the same resolved set `make cf-vars` does, per key with its source (`environment` / `project` / `global`, deprecated alias tagged), secrets masked. The `.env.linked` marker file the Rust resolver used to look for is deprecated and ignored; inspect warns about one and link/unlink delete it. Linking is rarely needed: a project with no project file already falls through to global.

**Agent rule:** one `mx infra setup cloudflare` per workstation covers every project on it. Add `make cf-setup` only when a single project needs a different account than the rest. When credentials look wrong, run `make cf-vars` first: it names the scope that answered, and that is usually the whole diagnosis.

## 6. Drift & gaps inventory (items 2 and 4 closed 2026-09-25, items 1 and 3 closed 2026-10-20, items 9-14 added 2026-10-26, items 15-17 added 2026-11-20)

1. **Closed 2026-10-20** (bd:mech-crate-vxq). Was: `mx cf` documented in INFRA_CONFIG.md + MCP resources, unimplemented in the CLI. Now `mx cf <cmd>` runs `make cf-<cmd>` in the project root (an app argument becomes `a=<app>`; `mx cf config` is an alias of `mx cf vars`), and `crates/mx-cli/tests/docs_drift.rs` fails if either doc names an mx command the CLI lacks.
2. **Closed 2026-09-25** (bd:mech-crate-wd9). Was: `CLOUDFLARE_ACCOUNT_ID` (mx infra) vs `CF_ACCOUNT_ID` (toolchain) mismatch, which made global credentials unusable by deploys. Now one canonical pair, with the old spelling read as a deprecated alias (§5).
3. **Closed 2026-10-20** (bd:mech-crate-066). Was: `mx infra link/unlink` stubs, with a `.env.linked` marker (Rust) and an `MX_INFRA_USE_GLOBAL` in-file flag (bash) that disagreed and were never written. Now the in-file flag is the one mechanism, written and removed by `mx infra link/unlink` and read by `mx infra inspect`, `cloudflare.mk` and `cf-init-app.sh` alike; `crates/mx-cli/tests/infra_link.rs` runs all three against one scratch project (§5).
4. **Closed 2026-09-25** (same change). Was: `cloudflare.mk` had no global-credential fallback, including the project file only. Now it reads global then project, so the project wins per scope and a global-only workstation deploys (§5).
5. Terraform: gitignored and never scaffolded; the appendix documents only a DigitalOcean module (`make gpu-*`), and its front-matter still advertises "deploying to Cloudflare with Terraform IaC", which nothing in it does. Re-assessed 2026-10-26 in §10: this is defensible, because the Terraform provider cannot configure a container application, so the gap is a missing *decision record*, not a missing scaffold.
6. `appendix-build-deploy-recipe.md` prescribes `infra/dockerfiles/` + `containers-worker/` layout; shipped reality is `docker/dockerfiles/<app>/app` + `infra/cloudflare/apps/<app>/` — treat the appendix as aspirational, this doc as ground truth.
7. Generated Dockerfile HEALTHCHECK path (`/health`) is the app's responsibility, not the worker's `/_health` control route — mismatched expectations if the app doesn't serve it.
8. Cron type ships `kv_namespaces` id `TODO_CREATE_KV_NAMESPACE` — deploy fails until the namespace is created and the id pasted in.
9. **wrangler 3 pins.** `templates/infra/cloudflare/containers-worker/package.json` pins `wrangler ^3.88.0`; `cf-init-app.sh` pins `^3.99.0` in the generated `package.json`. The `latest` dist-tag is 4.165.0 (2026-11-20; it was 4.152.0 on 2026-10-22); 3.x stopped at 3.114.17 (2026-01-13) under the `legacy` tag [S17]. The pin binds only commands run inside the app directory (`cf-dev`, `npm run *`): the `make cf-*` deploy targets run `npx wrangler` from the repo root and resolve whatever the root provides. Cloudflare's v4 announcement: v3 gets "bug fixes and security updates until Q1 2026, and ... critical security updates until Q1 2027, at which point it will be out of support" [S18]. Every containers feature in §8 (Dockerfile build on deploy, `--containers-rollout`, `containers instances|ssh`, `exports`) is wrangler 4 surface. Wrangler 4 also needs `node >= 22.12.0` [S17], so a CI image on Node 18 or 20 fails before it reaches Cloudflare.
10. **`@cloudflare/containers ^0.0.30`** (both templates) resolves to exactly 0.0.30, because a caret on a `0.0.x` version allows no other patch; current is 0.3.7 (2026-06-04) [S16]. The names the scaffold uses (`defaultPort`, `sleepAfter`, `envVars`, `getContainer`, `startAndWaitForPorts`, `container.fetch`) all still exist [S15], so the upgrade is a pin change, but 0.0.30 lacks `enableInternet`/`allowedHosts`/`deniedHosts`, `onActivityExpired`, and the 0.3.4 fix for "subclass sleepAfter overrides being ignored" [S15]. See §8.6 for the class's stated end-of-updates date.
11. **`wrangler kv:namespace create`** is what the cron scaffold prints as its next step [cf-init-app.sh, cron next-steps]. Wrangler 4.0.0 removed the aliases: "`wrangler kv:*`, replaced by `wrangler kv *`" [S19]; the KV docs say "The kv:... syntax is deprecated in versions 3.60.0 and beyond" [S20]. Once item 9 is fixed, that printed command fails.
12. **`[[migrations]] new_sqlite_classes`** still works, but the containers get-started now declares the class with `[exports.MyContainer] type = "durable-object" storage = "sqlite"` [S8]; `exports` "replaces the imperative migrations array" and the two "are mutually exclusive within a single Worker" [S21]; a Worker that has deployed with `exports` "cannot return to the legacy migrations" flow [S22]. New scaffolds should emit `exports`; already-deployed apps may stay on `migrations` (§9).
13. **Types and compatibility date.** `@cloudflare/workers-types ^4.20241112.0` (the example template; `cf-init-app.sh` generates `^4.20241205.0`) is a major behind (`latest` 5.20261120.1 on 2026-11-20 [S24]) and the TypeScript docs now say "We recommend you use wrangler types to generate runtime types, rather than using the @cloudflare/workers-types package" [S23]. `compatibility_date` is `2024-11-16` / `2024-12-01` in the two templates; the one wrangler 4 config in this repo, `site/apps/site/wrangler.jsonc`, uses `2026-08-01`, and wrangler ships a dated workerd (4.152.0 shipped 1.20261019.1, 4.165.0 ships 1.20261118.1), so any date up to the shipped workerd's is valid [S17][S49]. The site workflow pins `WRANGLER_VERSION: "4.125.0"` deliberately, so the repo's only wrangler 4 consumer is itself some forty releases behind.
14. **Config dialect.** Both templates are TOML; Cloudflare "recommends using wrangler.jsonc for new projects, and some newer Wrangler features will only be available to projects using a JSON config file" [S9]. TOML keeps working; `cf-sync-image`'s `perl` rewrite of `image = "..."` would not survive a switch to JSON, which is one more reason to drop that step (§9).
15. **Deprecated scheduling policy.** Generated configs omit `scheduling_policy`, which selects `default`; Cloudflare documents that policy as deprecated and the migration as a replacement application plus a worker rewrite (§8.6) [S12].
16. **Skeleton Node version.** The generated Dockerfile uses `node:20-alpine` and the README's CI example uses `node-version: '20'`, while wrangler 4 needs Node 22.12 or later (item 9). The container's runtime Node is independent of wrangler's, but the CI example runs wrangler.
17. **Closed 2026-11-20.** Was: `cf-init-app.sh` ignored credentials set in the environment (§5).

## 7. Agent quick-reference

- CF flow = `make cf-setup` → `make cf-init a=<app> type=container` → edit the app Dockerfile → `make cf-deploy a=<app>`. `mx cf setup` / `mx cf init <app> --type=container` / `mx cf deploy <app>` are the same targets through the CLI.
- Credentials the deploys read: `CLOUDFLARE_ACCOUNT_ID` (+ optional `CLOUDFLARE_API_TOKEN`), resolved from the environment, then `infra/cloudflare/.env.cloudflare`, then `~/.mech-crate/config/infra/cloudflare.env`. `make cf-vars` prints which scope answered; `make cf-check-credentials` is the guard that fails loudly when none did. `CF_ACCOUNT_ID` is a deprecated alias: read, warned about, never written.
- Production URL comes from `[env.production] routes` in the app's wrangler.toml; local dev URLs still come from the mx router.
- The Worker proxies everything to the container; container image versions come from `package.json` (`v<version>`), unlike every other mx build path.
- Platform facts that change design decisions (all §8): disk is ephemeral and "a fresh disk from the container image" appears after every sleep [S11]; memory and disk bill on *provisioned* size for every 10 ms the container is awake while CPU bills on active use [S4], so `sleepAfter` is the cost lever; a successful deploy means the image rollout *started* [S6]; `linux/amd64` is still mandatory [S8]; the container has internet by default and `allowedHosts`/`deniedHosts` restrict it [S25].
- Terraform (§10): if you adopt it, Terraform owns zone, DNS, KV/R2/D1/Queues, Secrets Store and (optionally) routes or custom domains; wrangler owns the Worker, the image and the container application. The provider can attach a container to a Durable Object class but has no field for image, size, instance count or rollout [S32].

## 8. Platform state at 2026-10-26 (researched)

What Cloudflare Containers is today, from primary pages, restricted to the facts that change how the mx scaffold should look or behave.

### 8.1 Status and sizing

- Public beta 2025-06-24 ("now available in beta for all users on paid plans") [S2]; GA 2026-04-13, listing "Higher limits allow you to run thousands of containers concurrently", "Active-CPU pricing means that you only pay for used CPU cycles", "Docker Hub support" and "SSH support" as the changes since beta [S1].
- Instance types (limits page, read 2026-10-21 and again 2026-11-20) [S3]:

| `instance_type` | vCPU | Memory | Disk |
|---|---|---|---|
| `lite` | 1/16 | 256 MiB | 2 GB |
| `basic` | 1/4 | 1 GiB | 4 GB |
| `standard-1` | 1/2 | 4 GiB | 8 GB |
| `standard-2` | 1 | 6 GiB | 12 GB |
| `standard-3` | 2 | 8 GiB | 16 GB |
| `standard-4` | 4 | 12 GiB | 20 GB |

  Custom sizes exist (`vcpu`, `memory_mib`, `disk_mb`: up to 4 vCPU, 12 GiB and 20 GB, at least 3 GiB per vCPU; the disk-per-GiB cap was removed on 2026-09-29) [S3][S51]. An image may be as large as the instance's disk ("Same as instance disk space"), and "All instance size maximums and account limits on this page are defaults and can be increased" [S3]. The scaffold sets no `instance_type` at all, so it gets the default, which is `lite` (1/16 vCPU, 256 MiB, 2 GB) [S9]: too small for a Node SSR service. Set it explicitly (§9).
- Autoscaling: "Not today, though Cloudflare plans to add built-in autoscaling in a future release" [S11]. `max_instances` is a cap, not a scaler, and "Stopped containers do not count towards this" [S9].
- Placement: "By default, containers run in the location nearest to the incoming request with a pre-fetched image"; `constraints.regions` and `constraints.jurisdiction` (eu, us, fedramp) pin it, and "Limited capacity regions (ME, OC, AFR) cannot be used exclusively" [S27].

### 8.2 Billing, and why `sleepAfter` is the cost knob

"Containers are billed for every 10ms that they are actively running", with three meters: memory $0.0000025 per GiB-second, CPU $0.000020 per vCPU-second, disk $0.00000007 per GB-second, plus egress [S4]. The sentence that matters: "Memory and disk usage are based on the provisioned resources" while "CPU usage is based on active usage only" [S4]; active-CPU billing arrived 2025-11-21 ("based on active usage only, instead of provisioned resources") [S5]. An awake, idle container therefore pays its full memory and disk every 10 ms. Practitioner arithmetic agrees: a `standard-3` kept always-on was priced at $158.42 a month, "roughly 2.5x" a comparable GCP instance, in the Sandbox SDK thread and a 2026-02 write-up [S41][S42] (both before or ignoring active-CPU pricing, so CPU is overstated; memory and disk are not). Workers and Durable Objects usage bills separately, and container logs bill as Workers Logs [S4].

### 8.3 Images and deploys

- "When running wrangler deploy, if you set the image attribute in your Wrangler configuration to a path to a Dockerfile, Wrangler will build your container image locally using Docker, then push it to a registry run by Cloudflare" [S7]; the get-started flow is `image = "./Dockerfile"` and "You must have Docker running locally when you run wrangler deploy" [S8]. Config keys around it: `image_build_context` ("by default it is the directory of image") and `image_vars` ("equivalent to using --build-arg") [S9]. The mx Dockerfile lives at `docker/dockerfiles/<app>/app` with the repo root as build context, which is exactly what `image_build_context` expresses.
- The registry route the scaffold uses is documented, but it belongs to the `default` scheduling policy, which is now deprecated (§8.6): `wrangler containers push <IMAGE>:<TAG>` ("Push a tagged image to a Cloudflare managed registry") and `wrangler containers build -t <name>:<tag> -p` to build and push in one step [S10]; `image = "registry.cloudflare.com/<ACCOUNT>/<IMAGE>:<TAG>"` remains valid for the default scheduling policy only [S7]. Under that policy Docker Hub, Amazon ECR and Google Artifact Registry images are accepted directly, with limits (private Docker Hub needs `wrangler containers registries configure`; public ECR is unsupported; Artifact Registry only on `*-docker.pkg.dev` hosts) [S7]. Under `durable_object` an image is either built from a Dockerfile or a digest-pinned Cloudflare-registry reference; other registries are not pulled directly [S12].
- "Your container image must be able to run on the linux/amd64 architecture" [S8]; `CF_DOCKER_PLATFORM=linux/arm64`, which `INFRA_CONFIG.md` offers as a choice, produces an image the platform cannot run.
- Rollouts: `wrangler deploy` uploads the Worker, pushes the image if needed and *starts* a rollout; "Deploy success means the rollout started, not that every container instance has finished replacing" [S6]. `rollout_step_percentage` defaults to "100 if max_instances is omitted or less than 2; otherwise [10, 100]", and `wrangler deploy --containers-rollout=immediate|none` overrides it [S6]. `wrangler versions upload` "does not publish a new image or roll out container instances" [S28], so preview-branch flows that rely on versions never refresh the image. Old and new containers coexist during a rollout, which a 2025 field report hit on day one [S45].
- Debugging: `wrangler containers list|info|instances|ssh|images list|images delete` [S10]. Container stdout lands in Workers Logs when `observability` is enabled, with a per-container override ("If you omit containers[].observability, Wrangler uses the top-level observability setting for that container") [S26]. The scaffold sets no `observability` block, so its containers log nowhere by default.

### 8.4 Disk, network, cold starts

- "All disk is ephemeral by default. When a Container instance goes to sleep, the next time it starts, it uses a fresh disk from the container image" [S11]. One team left the platform over this ("designed for ephemeral workloads", "we needed something designed for persistence from the ground up") [S43]. Treat the container disk as a cache.
- Egress is controlled per start: the outbound-traffic page documents `enableInternet` as a `ctx.container.start()` option ("a container started with `enableInternet: true` can reach any host") plus outbound intercept handlers that can allow, deny or route a hostname back into Worker bindings [S25]. The class-level `enableInternet`/`allowedHosts`/`deniedHosts` properties this section originally quoted are no longer on that page; check the package README [S15] before relying on them. In the scaffold, inbound traffic is only what the generated Worker forwards, through its `/_container/*` routes [templates/infra/cloudflare/containers-worker/src/index.ts].
- Environment variables: one 2026-06 report hit "a hard 5KB limit on environment variables" and moved configuration into the first request's headers [S44]. Source is a single practitioner post, so size your `envVars` accordingly and verify against the limits page before relying on the number.
- Cold starts: the FAQ says "Container cold starts can often be in the 1-3 second range" [S11]. A 2026-07 benchmark of 700 boots measured a hello-world median of 1.7 s but a p95 of 23.9 s, and an agent-tooling image at 10.5 s median, 15.6 s p95, concluding "a Cloudflare container can be faster; it can also be 30-40 seconds slower, and you can't predict which you'll get" [S30]. Cloudflare's 2026-09-30 rebuild reports median start 4.049 s → 648 ms and p99 6.717 s → 1129 ms, but those numbers are for the new `durable_object` scheduling policy (§8.6), not the default policy the scaffold uses [S13].

### 8.5 The 2026-09 cross-tenant disk incident

Reported 2026-09-04, fixed the same day, fleet cleanup finished 2026-09-19, published 2026-09-24: "When the thin volume backing a container's root disk was deleted, its physical blocks were returned to a pool that served workloads belonging to multiple customer accounts", with block zeroing skipped, so a new container could read a previous tenant's blocks; "We have no evidence that customer data has been compromised" and "remediation does not require any further action by Cloudflare customers" [S29][S53]. Nothing to do, but it is the concrete reason not to write secrets or customer data to the container disk when the Worker can hold them instead.

### 8.6 The `durable_object` scheduling policy and the `Container` class

On 2026-09-30 Cloudflare shipped a second scheduling policy, `scheduling_policy: "durable_object"` (public beta), in which the Durable Object picks image and size at runtime through `ctx.container.start()`, images are a named map (`images: { node: { dockerfile: ... } }`), filesystem snapshots exist, and `max_instances` is not supported [S12][S13]. The same post states: "We'll maintain the `Container` class and legacy `Sandbox` class through December 31, 2026. Existing deployments keep running after that date, but the classes won't get updates. We recommend migrating to `ctx.container`" [S13]. At the same date the package docs still route default-policy users through the class: "For durable_object applications, use the Durable Object Container API", with the `Container` class documented for everything else [S14]. The scaffold's `ContainerDO extends Container` is therefore on a maintenance-only base class with a stated date.

**The default policy itself is now deprecated (checked 2026-11-20).** The scheduling-policy page, last updated 2026-11-18, carries the callout: "The default scheduling policy is deprecated. This includes applications that omit scheduling_policy, which selects default automatically. Migrate to the durable_object scheduling policy. Existing applications require code changes and a replacement application. Changing the setting alone does not migrate them." [S12]. No removal date is given. The get-started page now shows only the `durable_object` form: `scheduling_policy = "durable_object"`, `[containers.images.base] dockerfile = "./Dockerfile"`, and `[exports.MyContainer]` [S8]. Rollouts "apply to Container applications that use the default scheduling policy"; Durable Object-managed instances do not take part, because application code selects the image when it calls `ctx.container.start()` [S6]. The scaffold omits `scheduling_policy`, so every generated app is a default-policy app.

### Synthesis (inferred)

- The scaffold is on a deprecated path twice over: its base class stops receiving updates after 2026-12-31, and its scheduling policy is documented as deprecated with no date. Moving to `durable_object` is not a pin bump. It needs a replacement application and a worker rewritten against `ctx.container`, with images as a named map. Decide whether new scaffolds should be generated for `durable_object` before investing in the §9 table, whose target config is a default-policy config.
- The cold-start spread in §8.4 means a request-path container behind the mx worker needs either a warm instance (a short `sleepAfter` saves money and costs tail latency; a long one is the reverse) or a Worker-side timeout and fallback. There is no setting that buys both.
- Rollout coexistence (§8.3, default policy only) means a container image change that also changes the Worker-to-container contract (ports, paths, headers) must be deployed in two steps, Worker first with both shapes accepted, image second.

## 9. Scaffold against platform: what to change, what to keep

| Scaffold today | Current state | Action |
|---|---|---|
| `wrangler ^3.88.0` / `^3.99.0` | 4.165.0 `latest` (2026-11-20); v3 past bug-fix support [S17][S18]; a final wrangler major pointing at `cf` is expected (§13) | Pin `^4`, require Node 22.12 or later in `engines` and CI images [S17]; treat `^4` as a bounded fix |
| `@cloudflare/containers ^0.0.30` | 0.3.7 [S16]; class maintained through 2026-12-31 [S13] | Pin `^0.3.7` now; re-check §8.6 before 2027 |
| `@cloudflare/workers-types ^4.20241112.0` | 5.x; `wrangler types` recommended [S23][S24] | Drop from app templates; generate `worker-configuration.d.ts` with `wrangler types`, and remove the generated `tsconfig.json` entry `"types": ["@cloudflare/workers-types/2023-07-01"]` with it |
| `compatibility_date = "2024-11-16"` / `"2024-12-01"` | site config uses `2026-08-01`; workerd 1.20261019.1 shipped [S17][S46] | Stamp the scaffold date at `cf-init` time |
| `[[migrations]] new_sqlite_classes` | `exports` is the documented path; one-way, mutually exclusive [S21][S22] | New apps: `exports`; deployed apps: leave on `migrations` or migrate deliberately |
| `wrangler kv:namespace create` (cron next-steps) | removed in 4.0.0 [S19] | Print `wrangler kv namespace create` |
| `cf-build` → `cf-push` → `cf-sync-image` (perl rewrite) → `wrangler deploy` | `wrangler deploy` builds and pushes from `image = <Dockerfile>` under the default policy [S7][S9] | Default: `image` = Dockerfile path + `image_build_context` = repo root; drop `cf-sync-image`. Keep `cf-publish` + registry tag as the opt-in for immutable prebuilt tags |
| No `instance_type`, no `observability` | default is `lite`; names are `lite`/`basic`/`standard-1..4` [S3][S9]; logs off unless enabled [S26] | Emit `instance_type` and `observability.enabled = true` |
| `routes = [{ pattern = "<domain>/*", zone_name = "<domain>" }]` | `custom_domain = true` creates DNS and certificates for you [S9][S47] | Prefer `custom_domain` when the Worker *is* the origin, as the site config does [S46]; keep `zone_name` patterns for path-scoped routes in front of an existing origin |
| TOML | JSONC recommended, some features JSON-only [S9] | Switch new scaffolds to `wrangler.jsonc`; wrangler reads both |
| `CF_DOCKER_PLATFORM` offered as `linux/amd64` or `linux/arm64` (prompt and doc text corrected 2026-11-20; the variable remains) | `linux/amd64` required [S8] | Done for the prompt and `INFRA_CONFIG.md`; originally: remove the arm64 choice from `INFRA_CONFIG.md` and `cf-setup.sh` |

Illustrative target config for a container app under the default scheduling policy, which is deprecated (§8.6): read it as the smallest change to today's scaffold, not as the destination. Not validated with wrangler. Every key is from the configuration reference [S9]; the get-started page now shows the `durable_object` form instead [S8].

```jsonc
// infra/cloudflare/apps/myapp.example.com/wrangler.jsonc
{
  "$schema": "https://json.schemastore.org/wrangler.json",
  "name": "myapp-worker",
  "main": "src/index.ts",
  "compatibility_date": "2026-10-19",
  "compatibility_flags": ["nodejs_compat"],
  "observability": { "enabled": true },
  "containers": [{
    "class_name": "ContainerDO",
    // Paths are relative to this file, which sits four levels below the repo root.
    // cf-init names both folders after a=, so the Dockerfile folder matches the app folder.
    "image": "../../../../docker/dockerfiles/myapp.example.com/app", // a Dockerfile path: wrangler deploy builds and pushes it
    "image_build_context": "../../../..",                            // repo root, as cf-build uses today
    "image_vars": { "APP_VERSION": "1.4.0" },                        // cf-init must write this from package.json; it replaces cf-build's --build-arg
    "instance_type": "standard-1",
    "max_instances": 5
  }],
  "durable_objects": { "bindings": [{ "name": "APP_CONTAINER", "class_name": "ContainerDO" }] },
  "exports": { "ContainerDO": { "type": "durable-object", "storage": "sqlite" } },
  "routes": [{ "pattern": "myapp.example.com", "custom_domain": true }]
}
```

Open question to settle when this is implemented: the docs say `image` is "a local path to a Dockerfile" [S9]; whether a file not literally named `Dockerfile` (mx names it `app`) is accepted was not verified. If not, `cf-init` should emit `docker/dockerfiles/<app>/Dockerfile` for container apps.

### Synthesis (inferred)

- Do the toolchain bump (wrangler 4, Node 22, containers 0.3.x, `kv namespace`) as one change with `cf-dev` and `cf-deploy-preview` as the test, before touching the pipeline shape. Those four are compatibility fixes; the rest of the table is design.
- Keep `cf-publish` and the registry-tag path alive after moving the default to Dockerfile-on-deploy. It is the only mx path that produces an immutable `v<version>` tag you can roll back to by editing one line, and under the `durable_object` policy an image is either built from a Dockerfile named in `images.<name>.dockerfile` or referenced from the Cloudflare registry by digest (`@sha256:`); a `:v<version>` tag reference is not accepted there, so `cf-publish` would have to record and pin the digest [S12].
- `cf-init` should write `compatibility_date` from `date +%F` rather than a constant, so the drift in §6 item 13 cannot recur.

## 10. Terraform's place

The backlog asked for "terraform flow front to back". The honest answer is that there is no Cloudflare-sanctioned Terraform path that deploys a container Worker end to end, so the mx decision is *what Terraform may own*, not how to scaffold it.

**What the provider can do.** Cloudflare's Terraform provider is at v5.29.0 (2026-11-11; it was v5.28.1 at the October pass) with v4.52.10 and v3.35.1 still receiving releases [S36]. v5 is generated: "Unlike the earlier Terraform providers, v5 is automatically generated based on the OpenAPI Schemas for our REST APIs", and the repository "is auto-generated and maintained internally at Cloudflare, so we are no longer accepting pull requests from the community" [S34]; it carried roughly 200 open issues at the research date [S36]. v4 → v5 renamed more than 40 resources (`cloudflare_worker_script` → `cloudflare_workers_script`, `cloudflare_workers_secret` folded into `secret_text` bindings) and ships `tf-migrate` plus state upgraders for the move [S35]. The `cloudflare_workers_script` page itself now says "For more direct control over Workers resources, we recommend the beta `cloudflare_worker`, `cloudflare_worker_version`, and `cloudflare_workers_deployment` resources" [S33]. Zone, DNS records, KV, R2, D1, Queues, routes and custom domains all have resources [S35][S36].

**What it cannot do.** `cloudflare_worker_version.containers` is a "List of containers attached to a Worker. Containers can only be attached to Durable Object classes of this Worker script", and its nested schema has one required attribute, `class_name` [S32]. There is no attribute for the image, `instance_type`, `max_instances` or rollout, so the container *application* (image, size, instances, rollout) is outside Terraform. Cloudflare's own infrastructure-as-code page frames Terraform, the SDKs and the API as the tools for "build and deploy scripts, CI/CD pipelines, custom developer tools" and warns that "None of the examples below do Workers Bundling. This is usually done with Wrangler or a tool like esbuild"; it also records the sequencing trap "Durable Object migrations are applied with deployments. This means you can't bind to a Durable Object in a Version if a deployment doesn't exist" [S31], which the provider's issue tracker resolves as a two-apply dance: a version with migrations and no bindings, a deployment, then a version with the binding [S37]. A production migration record from GitLab names a representative v4 pain: the provider "lacks the observability attribute on cloudflare_worker_script, and manually enabling observability in the Cloudflare dashboard is overwritten on each Terraform apply" [S52].

**Adjacent tools.** Pulumi has first-party Cloudflare docs ("With the Pulumi Cloudflare package, you can build, deploy, and manage Cloudflare resources using standard programming languages") and is bridged from the same Terraform provider [S38]; OpenTofu consumes the same provider from its registry with no separate Cloudflare documentation. Neither adds container coverage. For CI, `cloudflare/wrangler-action@v4` (v4.1.3, 2026-09-24) takes `apiToken` and `accountId` inputs and installs wrangler 4 by default [S39]; Workers Builds (git-connected) can build containers: "Dockerfile builds can run in the Workers Builds environment", production branches "Publishes the image when needed and rolls out container instances", other branches run `wrangler versions upload`, which does not [S28].

**Decision for mx (recorded 2026-10-26).** Keep Terraform unscaffolded for the container path; wrangler is the only complete deploy tool for it. Where a project adopts Terraform, the split is: Terraform owns account- and zone-level resources (zone, DNS, KV/R2/D1/Queues, Secrets Store [S48], and routes or custom domains if the team prefers them outside wrangler) and emits their IDs; wrangler owns the Worker, its bundling, the image and the container application. `appendix-build-deploy-recipe.md`'s Terraform section describes a DigitalOcean GPU-droplet module with `make gpu-*` targets, not Cloudflare, and should not be read as the Cloudflare plan.

### Synthesis (inferred)

- If the split above is adopted, the natural mx shape is `infra/terraform/cloudflare/` with one module per project, outputs for `kv_namespace_id`, `d1_database_id`, `r2_bucket_name` and `zone_id`, and a `make cf-tf-outputs` target that renders them into the app's `wrangler.jsonc` `vars`/bindings. The handoff point is the ID, nothing else, so the two tools never fight over a resource.
- Routes are the one resource both tools can own. Pick one: wrangler `routes` for projects that are wrangler-only, Terraform `cloudflare_workers_route`/`cloudflare_workers_custom_domain` for projects where DNS is already in Terraform, never both, or each `apply`/`deploy` rewrites the other's work (the GitLab observability clash in §10 is the same failure class).
- Re-evaluate when `cloudflare_worker_version.containers` gains image or instance attributes, or when `cloudflare_worker`/`cloudflare_worker_version` leave beta; either would move the container application inside Terraform's reach and reopen the scaffold question.

## 11. Sources (2026-10-26 pass)

- [S1] Cloudflare changelog, "Containers and Sandboxes are now GA", 2026-04-13 — https://developers.cloudflare.com/changelog/post/2026-04-13-containers-sandbox-ga/
- [S2] Cloudflare blog, Containers public beta, 2025-06-24 — https://blog.cloudflare.com/containers-are-available-in-public-beta-for-simple-global-and-programmable/
- [S3] Containers limits — https://developers.cloudflare.com/containers/platform-details/limits/
- [S4] Containers pricing — https://developers.cloudflare.com/containers/pricing/
- [S5] Changelog, active-CPU pricing, 2025-11-21 — https://developers.cloudflare.com/changelog/post/2025-11-21-new-cpu-pricing/
- [S6] Containers rollouts — https://developers.cloudflare.com/containers/platform-details/rollouts/
- [S7] Containers image management — https://developers.cloudflare.com/containers/platform-details/image-management/
- [S8] Containers get started — https://developers.cloudflare.com/containers/get-started/
- [S9] Wrangler configuration reference — https://developers.cloudflare.com/workers/wrangler/configuration/
- [S10] `wrangler containers` commands — https://developers.cloudflare.com/workers/wrangler/commands/containers/
- [S11] Containers FAQ — https://developers.cloudflare.com/containers/faq/
- [S12] Containers scheduling policy — https://developers.cloudflare.com/containers/configuration/scheduling-policy/
- [S13] Cloudflare blog, "Cloudflare Containers, rebuilt to scale agent sandboxes", 2026-09-30 (HN https://news.ycombinator.com/item?id=49909470) — https://blog.cloudflare.com/faster-agent-sandboxes/
- [S14] Container package docs — https://developers.cloudflare.com/containers/container-package/
- [S15] cloudflare/containers README and releases — https://github.com/cloudflare/containers
- [S16] npm registry, `@cloudflare/containers` (latest 0.3.7, 2026-06-04) — https://registry.npmjs.org/@cloudflare/containers
- [S17] npm registry, `wrangler` (latest 4.152.0 at the October pass and 4.165.0 on 2026-11-20, legacy 3.114.17, engines node >=22.12.0, peer workers-types ^5) — https://registry.npmjs.org/wrangler
- [S18] Changelog, Wrangler v4, 2025-03-13 — https://developers.cloudflare.com/changelog/post/2025-03-13-wrangler-v4/
- [S19] workers-sdk wrangler CHANGELOG, 4.0.0 removals — https://github.com/cloudflare/workers-sdk/blob/main/packages/wrangler/CHANGELOG.md
- [S20] KV commands reference — https://developers.cloudflare.com/kv/reference/kv-commands/
- [S21] Changelog, declarative Durable Object class exports, 2026-06-30 — https://developers.cloudflare.com/changelog/post/2026-06-30-declarative-do-class-exports/
- [S22] Durable Objects migrations reference — https://developers.cloudflare.com/durable-objects/reference/durable-objects-migrations/
- [S23] Workers TypeScript guide — https://developers.cloudflare.com/workers/languages/typescript/
- [S24] npm registry, `@cloudflare/workers-types` (latest 5.20261026.1 in October, 5.20261120.1 on 2026-11-20) — https://registry.npmjs.org/@cloudflare/workers-types
- [S25] Containers outbound traffic — https://developers.cloudflare.com/containers/configuration/outbound-traffic/
- [S26] Changelog, per-container observability, 2026-09-08 — https://developers.cloudflare.com/changelog/post/2026-09-08-per-container-observability/
- [S27] Containers placement — https://developers.cloudflare.com/containers/platform-details/placement/
- [S28] Containers deploy guide — https://developers.cloudflare.com/containers/guides/deploy/
- [S29] Cloudflare blog, Containers cross-tenant vulnerability, 2026-09-24 — https://blog.cloudflare.com/containers-cross-tenant-vulnerability/
- [S30] Alchemy, "Benchmarking Cloudflare Containers vs. AWS MicroVMs", 2026-07-01 (HN https://news.ycombinator.com/item?id=48882032) — https://alchemy.run/blog/2026-07-01-microvm-cold-starts/
- [S31] Workers infrastructure as code — https://developers.cloudflare.com/workers/platform/infrastructure-as-code/
- [S32] Terraform provider docs, `cloudflare_worker_version` — https://github.com/cloudflare/terraform-provider-cloudflare/blob/main/docs/resources/worker_version.md
- [S33] Terraform provider docs, `cloudflare_workers_script` — https://github.com/cloudflare/terraform-provider-cloudflare/blob/main/docs/resources/workers_script.md
- [S34] cloudflare/terraform-provider-cloudflare README — https://github.com/cloudflare/terraform-provider-cloudflare
- [S35] Terraform provider v5 migration guide — https://github.com/cloudflare/terraform-provider-cloudflare/blob/main/docs/guides/version-5-migration.md
- [S36] Terraform provider releases (v5.28.1 2026-10-20, v5.29.0 2026-11-11, v4.52.10 2026-10-22, v3.35.1 2026-10-22) and issues — https://github.com/cloudflare/terraform-provider-cloudflare/releases
- [S37] Terraform provider issue #6322, Durable Object binding sequencing — https://github.com/cloudflare/terraform-provider-cloudflare/issues/6322
- [S38] Cloudflare Pulumi docs — https://developers.cloudflare.com/pulumi/
- [S39] cloudflare/wrangler-action releases (v4.1.3, 2026-09-24) — https://github.com/cloudflare/wrangler-action/releases
- [S40] Workers Builds — https://developers.cloudflare.com/workers/ci-cd/builds/
- [S41] HN, "Cloudflare Sandbox SDK" discussion, 2025-10 — https://news.ycombinator.com/item?id=45610523
- [S42] Alec Armbruster, "Building an Agent SaaS with Cloudflare Containers", 2026-02-19 — https://alec.is/posts/building-an-agent-saas-with-cloudflare-containers/
- [S43] camelAI, "We tried Modal, Cloudflare Containers, and Sprites, then built our own", 2026-03-21 — https://camelai.com/blog/we-tried-every-container-service-then-built-our-own
- [S44] Waystones, "JIT Provisioning on Cloudflare Containers", 2026-06-01 — https://waystones.cloud/journal/jit-provisioning-cloudflare-containers
- [S45] Ashley Peacock, "Running containers on Cloudflare", 2025-06-24 — https://blog.ashleypeacock.co.uk/p/running-containers-on-cloudflare
- [S46] mech-crate repo, `site/apps/site/wrangler.jsonc`
- [S47] Workers custom domains — https://developers.cloudflare.com/workers/configuration/routing/custom-domains/
- [S48] Secrets Store with Workers — https://developers.cloudflare.com/secrets-store/integrations/workers/
- [S49] Workers compatibility dates — https://developers.cloudflare.com/workers/configuration/compatibility-dates/
- [S51] Containers product changelog — https://developers.cloudflare.com/changelog/product/containers/
- [S52] GitLab infrastructure epic 1663, Cloudflare provider v5 migration — https://gitlab.com/groups/gitlab-com/gl-infra/-/epics/1663
- [S53] BleepingComputer on the cross-tenant flaw, 2026-09-27 (HN https://news.ycombinator.com/item?id=49870848) — https://www.bleepingcomputer.com/news/security/cloudflare-fixes-containers-cross-tenant-flaw-exposing-customer-data/

Sections 12 and 13 come from an earlier refresh of this doc (2026-10-12, PR #67) that the 2026-10-26 pass above superseded everywhere else. They cover two topics that pass did not, and cite their sources inline by URL rather than by `[S n]`. Version numbers in them are as of 2026-10-12.

## 12. The second path: an assets-only Worker for mechcrate.dev (2026-10-12 pass)

The repo's own docs site does not use `cloudflare.mk` at all. `site/apps/site/wrangler.jsonc` has an `assets` block and **no `main`**: `"directory": "./dist"`, `"html_handling": "auto-trailing-slash"`, `"not_found_handling": "404-page"`, and a `routes` entry with `"custom_domain": true` for the apex, which fails loudly ("Could not find zone for mechcrate.dev") rather than half-deploying when the zone is missing [site/apps/site/wrangler.jsonc]. `.github/workflows/site.yml` builds, link-checks and deploys it independently of the Rust CI, and skips with a `::notice` until the two repository secrets exist; note the org secret is named `CLOUDFLARE_API_KEY` and is exported to wrangler as `CLOUDFLARE_API_TOKEN` [site.yml:14-36]. Response headers live in `site/apps/site/public/_headers` (immutable cache on `/_astro/*`, HSTS without `preload`, nosniff, referrer policy, explicit `text/plain` on the llms artefacts) and the workflow re-proves them against a local workerd before deploying [public/_headers; site.yml:52-60].

Why this is the right shape for a static bundle, in Cloudflare's words: "Now that Workers supports both serving static assets and server-side rendering, you should start with Workers." and "Cloudflare Pages will continue to be supported, but, going forward, all of our investment, optimizations, and feature work will be dedicated to improving Workers." [https://blog.cloudflare.com/full-stack-development-on-cloudflare-workers/, 2025-04-08]. `_headers` and `_redirects` are honoured by Workers static assets with their own limits (`_headers`: 100 rules and 2,000 characters per line, as the repo's `_headers` comment reads out of the wrangler source; `_redirects`: 2,000 static plus 100 dynamic rules and 1,000 characters per rule), and neither applies to responses a Worker script generates, which matters the moment `run_worker_first` routes a path to code [https://developers.cloudflare.com/workers/static-assets/headers/]. "Requests to static assets are free and unlimited" and storing them costs nothing [https://developers.cloudflare.com/workers/static-assets/billing-and-limitations/]; the caps are 20,000 files per Worker version on Free, 100,000 on Paid, 25 MiB per file, and the `_headers` figures above (100 rules, 2,000 characters per line) come from the same table [https://developers.cloudflare.com/workers/platform/limits/]. Cloudflare "recommends using `wrangler.jsonc` for new projects, and some newer Wrangler features will only be available to projects using a JSON config file" [https://developers.cloudflare.com/workers/wrangler/configuration/], which the site follows and the container generator (TOML) does not.

## 13. Tooling horizon: cf CLI, wrangler, Deno (2026-10-12 pass)

- **`cf` CLI, open beta since 2026-09-28**, with a TypeScript configuration format (`cloudflare.config.ts`): "When the open beta ends we will release a final major version of Wrangler that directs you and your agent to use cf." and "We'll continue to provide maintenance support for Wrangler for 18 months after the beta ends, to give you time to migrate." [https://blog.cloudflare.com/cloudflare-cf-cli-launch/]. `cf` was at 1.0.0-beta.14 on 2026-10-09 and is at 1.0.0-rc.2 on npm as of 2026-11-20, so general availability looks close; the beta-end date, which starts Wrangler's 18-month maintenance clock, has still not been announced. Its docs say `cf migrate` converts wrangler config but Durable Objects, Workflows and Containers need manual adjustment [https://developers.cloudflare.com/cf/wrangler/]. Wrangler itself is still 4.x (4.149.0 on 2026-10-08, 4.165.0 on 2026-11-20); there is no wrangler 5.
- **Deno joined Cloudflare on 2026-10-09.** Cloudflare's side: the Deno team will "lead a new effort to make workerd self-hosting a first-class supported way to build and run apps using the Workers programming model", "merging code and ideas from celld back into workerd" [https://blog.cloudflare.com/deno-joins-cloudflare/]. Deno's side: "We will support the Deno runtime for another year with monthly releases containing bug fixes and security updates. After that year we will end our development of the Deno runtime."; "Deno Deploy will continue operating for six months before shutting down"; JSR continues [https://deno.com/blog/cloudflare]. Nothing changes for existing Workers deployments; for mx the relevant promise is self-hostable workerd, a possible third front door for the same container image.
