---
title: Recipes
description: What ships in the box, what each recipe carries, the measured apply status, and how to write your own.
sidebar:
  order: 3
---

A recipe is a production-shaped service definition. It carries the decisions of
the stack it came from — dependency choices, dockerfile targets, a dev override,
a health endpoint, admin tooling, deploy configuration — and `mx add` applies all
of it in one motion.

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
| `zola` | 1.0 | ✅ | Zola static site generator — single binary, no runtime dependencies. |

:::note[How that column was measured]
Not asserted — run. On 2026-08-25, against a build of this repository, a scratch
project was created outside the repo with `mx new` and then each recipe applied
into it:

```bash
mx add svc<recipe> --recipe <recipe> --domain svc<recipe>.localhost
```

All seven exited `0` and landed their compose files, dockerfiles and env files.
This supersedes the ⚠️ markers the README carried for `laravel`, `rust-worker`
and `zola`, which described a Tera templating defect that has since been fixed.

Two things the column does **not** claim. `laravel`'s post-install
`generate-secrets.sh` emitted a non-fatal warning during the run (running the
same script directly afterwards exits `0`). And "applies" is not "builds": for
`astro`, `nuxt` and `zola` the recipe's framework scaffolder is skipped when the
app directory already exists, so those give you the operational layer plus a
source skeleton and you run the framework's own install step yourself. Every
recipe prints its exact next steps when it finishes.
:::

## What `mx add` actually does

`mx add` reads the recipe manifest and works through it in order:

1. **Options** — defaults from the manifest, overridden by `--domain` and any
   `--opt key=value`. `mx recipes info <name>` lists what a recipe accepts;
   `rust-api`, for example, takes `rust`, `port` and `domain`.
2. **Placeholders** — `{{SERVICE_NAME}}`, the port, the domain and the rest are
   substituted through every template.
3. **Directories** — the app's source tree is created.
4. **Framework scaffold** (`init_app`, where a recipe declares one) — skipped if
   the app directory already exists.
5. **Templates** — the app files, `docker/compose/<service>.yml` and
   `<service>.dev.yml`, `docker/dockerfiles/<service>/app` and `app.prod`, and
   `docker/.config/.env.<service>`.
6. **Router labels** — the `Host()` rule for your `--domain` and the
   `devmesh-traefik` wiring.
7. **Post-install** — anything the recipe declares, such as generating secrets.

Recipes that need backing services also drop `db.yml` / `redis.yml` in, once.

The corpus has the long-form version of this, including how recipes and the build
system fit together:

**→ [mx recipes and build](/docs/corpus/process/mx-recipes-and-build/)**

## Writing your own

Recipes are directories of templates plus a `recipe.json` manifest that declares
options, placeholders, directories, templates, an optional `init_app` command and
optional post-install steps. The full authoring guide — manifest schema,
placeholder rules, the conformance tests a recipe has to pass — is in the corpus:

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
