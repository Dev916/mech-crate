---
title: Recipes
description: What ships in the box, what each recipe carries, the measured apply status, and how to write your own.
sidebar:
  order: 3
---

A recipe is a production-shaped service definition. It carries the decisions of
the stack it came from: dependency choices, dockerfile targets, a dev override,
a health endpoint, admin tooling, deploy configuration. `mx add` applies all of
it in one motion.

```bash
mx recipes list                                      # what is installed
mx recipes info rust-api                             # options, features, services
mx add api --recipe rust-api --domain api.localhost  # apply it
```

## What ships

| Recipe | Version | Apply | What it carries |
|---|---|---|---|
| `astro` | 2.0 | ✅ | Astro 5 with Vue 3 islands, SSR, shadcn-vue, PrimeVue, global state, Cloudflare deployment. Brings `db` and `redis`. |
| `laravel` | 12.0 | ✅ | Laravel 12 + Octane (Swoole), Filament admin, Inertia.js SSR. Brings a worker, a scheduler, `db` and `redis`. |
| `nuxt` | 3.15 | ✅ | Nuxt 3 SSR/SSG with the Nitro server and Tailwind CSS. Standalone. |
| `rust-api` | 1.0 | ✅ | Actix-web + SQLx, hexagonal architecture. Brings `db` and `redis`. |
| `rust-leptos` | 1.0 | ✅ | Leptos SSR on Actix-web with shadcn-ui, actor model, Postgres and Redis. |
| `rust-worker` | 1.0 | ✅ | Job worker on Redis pub/sub with Postgres and local LLM evaluation. |
| `zola` | 1.0 | ✅ | Zola static site generator. Single binary, no runtime dependencies. |

:::note[How that column was measured]
Run, not asserted. On 2026-08-25, against a build of this repository, a scratch
project was created outside the repo with `mx new` and then each recipe applied
into it:

```bash
mx add svc<recipe> --recipe <recipe> --domain svc<recipe>.localhost
```

All seven exited `0` and landed their compose files, dockerfiles and env files.
This supersedes the ⚠️ markers the README carried for `laravel`, `rust-worker`
and `zola`, which described a Tera templating defect that has since been fixed.

Two later fixes are now gated rather than measured by hand. Every installed
recipe's assembled dev config has to pass `docker compose config` straight after
`mx add`, against the file set `compose_context_files` hands `make dev`, and no
recipe may reference a compose file it does not ship. That net was built for the
`astro` recipe, which declared `include:` entries for `db.yml` and `redis.yml`
while shipping neither, and it immediately caught `rust-worker` depending on the
same two services without defining them. And `mx add` now runs a recipe's
framework scaffolder for real, which it previously skipped every single time.

What the column still does **not** claim. "Applies" is not "builds": you still
run the dependency install yourself, and every recipe prints its exact next steps
when it finishes.

Three further gates joined since. Every `env_file` list in every shipped compose
file has to follow the documented layering (`.env.shared` then `.env.secrets`
then `.env.<service>`), and a db-bearing recipe has to carry `.env.secrets` at
all. No recipe may pin a `container_name` or join an `external: true` network mx
does not create, both of which render clean through `docker compose config` and
then fail at `up`. And no recipe may ship a `__GENERATE_*__` placeholder that
nothing replaces, which is what used to leave `rust-api` with a literal
`__GENERATE_DB_PASSWORD__` as its database password.
:::

## A recipe boots unaided

`mx new`, `mx add <svc> --recipe <r>`, `make dev`. No hand edit in between, for
every recipe that brings Postgres with it.

That is new. `scripts/init.sh` used to copy `.env.secrets.template` verbatim,
leaving `DB_USER`, `DB_PASSWORD` and `DB_NAME` empty, and Postgres refuses to
initialize with an empty password. `scripts/generate-secrets.sh` is now the single
generation point every recipe shares: `make init` runs it, `make dev` runs
`make init`, and it fills only what is still empty or still a placeholder. See
[Compose &amp; env conventions](/docs/framework/compose-env/#the-three-env-layers).

## Re-scaffolding: `force_init`

`astro`, `nuxt` and `zola` run a framework initializer, and by default a second
`mx add` over a service that already has an app skips it and says so. To start
over from the framework's own starter:

```bash
mx add site --recipe astro --opt force_init=true
```

This **deletes** `apps/site/` and re-runs the initializer before mx's wiring is
layered back on. Destructive by design, and it refuses any `target_dir` that
escapes the project rather than following it outward.

:::note[The division of labour inside an Astro service]
Worth stating because getting it wrong is what broke the recipe. The scaffolder
(`create-astro`) owns `package.json`, `tsconfig.json` and the app payload. The
recipe owns the infrastructure (compose, dockerfiles, env) plus exactly one app
file: a dependency-free `/api/health`. It has to run on a fresh scaffold with
nothing installed but `astro` itself, which is why it imports nothing. It used to
import `@/lib/db` and `@/lib/redis` from a payload the recipe never installed,
under a path alias the scaffolded `tsconfig.json` never defines, so `/api/health`
answered 500 on every new service and the container healthcheck never passed.

The production image serves whichever shape the build produced.
`create-astro --template minimal` ships no adapter, so `astro build` emits a
static `dist/` and no `dist/server/entry.mjs`. The stage runs the SSR entry when
the app built one, and otherwise serves the static build. Adding the adapter is
the app's call (`astro add node`), because the scaffolder owns
`astro.config.mjs`. Both shapes answer `/api/health`: with static output the
route is prerendered to `dist/api/health`.
:::

## What `mx add` actually does

`mx add` reads the recipe manifest and works through it in order:

1. **Options**: defaults from the manifest, overridden by `--domain` and any
   `--opt key=value`. `mx recipes info <name>` lists what a recipe accepts;
   `rust-api`, for example, takes `rust`, `port` and `domain`.
2. **Placeholders**: `{{SERVICE_NAME}}`, the port, the domain and the rest are
   substituted through every template.
3. **Framework scaffold** (`init_app`, where a recipe declares one): `astro`,
   `nuxt` and `zola` run the framework's own initializer here. `mx add` prints
   `Scaffolding the app…` and then the command it used, or
   `Scaffolder skipped: <dir> already holds an app` when the target is not empty.
   This step comes *first*, so mx's own wiring layers on top of the starter files
   and wins every collision with them.
4. **Directories**: any part of the app's source tree the scaffolder did not
   create is filled in.
5. **Templates**: the app files, `docker/compose/<service>.yml` and
   `<service>.dev.yml`, `docker/dockerfiles/<service>/app` and `app.prod`, and
   `docker/.config/.env.<service>`.
6. **Router labels**: the `Host()` rule for your `--domain` and the
   `devmesh-traefik` wiring.
7. **Post-install**: anything the recipe declares, such as generating secrets.

Recipes that need backing services also drop `db.yml` / `redis.yml` in, once.

A scaffolder command has to be non-interactive, because `mx add` runs it without a
terminal. The shipped defaults carry the flags that make that true, and a command
that exits `0` while leaving the target empty is treated as a failure rather than
a success, with the command, its working directory and both streams reported. To
use a different starter, pass your own:
`mx add site --recipe astro --opt init_cmd='...'`.

The corpus has the long-form version of this, including how recipes and the build
system fit together:

**→ [mx recipes and build](/docs/corpus/process/mx-recipes-and-build/)**

## Writing your own

Recipes are directories of templates plus a `recipe.json` manifest that declares
options, placeholders, directories, templates, an optional `init_app` command and
optional post-install steps. The full authoring guide (manifest schema,
placeholder rules, the conformance tests a recipe has to pass) is in the corpus:

**→ [Recipe Authoring Guide](/docs/corpus/process/recipe-authoring-guide/)**

Also useful while authoring:
[Docker assembly guide](/docs/corpus/docker/docker-assembly-guide/) for the
dockerfile and compose halves, and
[Compose &amp; env conventions](/docs/framework/compose-env/) for what your
generated compose files have to honour.

## Remote recipes

`mx recipes pull`, `mx recipes versions` and `mx recipes cache` work against
Unyform-hosted blueprints. That path is optional and needs an account; local
recipes need neither. See [Remote blueprints](/docs/framework/unyform/).
