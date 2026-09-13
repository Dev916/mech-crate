---
title: Cloudflare deploy
description: The cf-* make targets, the three worker types, and how this site is meant to ship.
sidebar:
  order: 8
---

mx ships a Cloudflare deploy path as a **conditional** part of the folder
contract: a `make/cloudflare.mk` module and an `infra/cloudflare/` tree that
exist only in projects that asked for them.

```bash
mx new my-app --infra cloudflare   # at creation time
mx infra setup cloudflare          # credentials, once per workstation
```

:::caution[The `cf-*` targets are not in every project]
Without `--infra cloudflare`, `make/cloudflare.mk` is simply not in `make/`, and
`make cf-setup` fails with "no rule to make target". Check `make help`. If you
see the `cf-*` block, you have it. `mx upgrade` treats these files as conditional
on `infra/cloudflare/` existing, so a project that opted in stays current and one
that did not is never offered them.
:::

## The flow

```bash
make cf-setup                    # interactive wizard: auth, account id, optional API token
make cf-init a=myapp type=worker # scaffold an app (worker | cron | container)
make cf-deploy a=myapp           # deploy to production
```

Three worker types, chosen at `cf-init`:

| Type | For |
|---|---|
| `worker` | standard edge worker handling HTTP requests |
| `cron` | scheduled worker, with KV for state and optional webhook notifications |
| `container` | a container image behind a worker: port, sleep timeout, max instances, custom domain |

## Target reference

| Target | Purpose |
|---|---|
| `make cf-setup` | Interactive setup wizard |
| `make cf-login` / `make cf-whoami` | Authenticate / show auth status |
| `make cf-status` / `make cf-list` | All apps' status / list configured apps |
| `make cf-init a=<app> type=<type>` | Initialize an app |
| `make cf-install a=<app>` | Install worker dependencies |
| `make cf-dev a=<app>` | Run the worker locally |
| `make cf-build a=<app>` | Build the container image |
| `make cf-push a=<app>` | Push to the Cloudflare registry |
| `make cf-publish a=<app>` | Build and push |
| `make cf-images a=<app>` | List registry images |
| `make cf-sync-image a=<app>` | Sync `wrangler.toml` to a new image tag |
| `make cf-deploy a=<app>` | Deploy to production |
| `make cf-deploy-preview a=<app>` | Deploy to preview |
| `make cf-deploy-all` | Deploy every configured app |
| `make cf-logs a=<app>` / `cf-logs-preview` | Tail production / preview logs |
| `make cf-restart a=<app>` | Restart the container |
| `make cf-container-status a=<app>` | Check container status |

Per-app configuration lives in that app's `wrangler.toml`; credentials resolve
through [`mx infra`](/docs/framework/infra-credentials/) rather than a file in
the project.

## The full guides

Worker type internals, directory layout, per-app `wrangler.toml` configuration,
versioning, CI/CD wiring and troubleshooting:

**→ [Cloudflare Infrastructure](/docs/corpus/framework-guides/cloudflare/)**

And a researched, as-implemented account of the same flow, including a drift
inventory of the places the documentation and the code disagree. Worth reading
before you debug a credential problem:

**→ [mx Cloudflare Deploy: as-implemented flow, credentials, and known traps](/docs/corpus/infra/mx-cloudflare-deploy/)**

## How this site ships

mechcrate.dev is built from `site/` in the mech-crate repository, a nested mx
project with the Astro app at `site/apps/site/`. The intended deploy is the
astro recipe's Cloudflare path: a GitHub Actions workflow that runs `astro build`
on pushes to `main` touching `site/**` or `docs/**` and publishes the static
bundle with wrangler, plus a preview deploy on pull requests. The site job is
independent of the Rust gates. It never blocks `ci.yml` and is never blocked by
it.

That workflow is now `.github/workflows/site.yml`. `site/` was scaffolded without
`--infra cloudflare`, so it has no `make/cloudflare.mk`. The workflow calls
wrangler directly rather than through the `cf-*` targets.

:::tip[This site is an mx app]
The page you are reading is not built by a bespoke docs pipeline. `site/` is an
ordinary mx project (the same folder contract, the same compose layering, the
same router) scaffolded from the `astro` recipe. You can clone the repository
and run it exactly like any other mx service:

```bash
git clone https://github.com/Dev916/mech-crate.git
cd mech-crate/site
make doctor && make init
make dev            # http://mechcrate.localhost through the mx router
make down
```

No ports to remember and nothing to configure: the container publishes no HTTP
port, Traefik routes to it by the `Host(mechcrate.localhost)` label in
`docker/compose/site.yml`, and the mx router serves it alongside every other
project you have running.
:::

Two things about the dev container are specific to this site, and both are
commented where they live:

- **No database or cache.** The astro recipe's compose `include`s `db.yml` and
  `redis.yml`; a static documentation build needs neither, so
  `docker/compose/site.yml` drops them.
- **The corpus lives outside the app.** The techniques corpus is read from the
  repository's `docs/development/`, which is above `apps/site` and therefore
  outside the container's source mount. `docker/compose/site.dev.yml` bind-mounts
  `docs/` read-only at `/repo/docs` and sets `MECHCRATE_REPO_ROOT`, so `make dev`
  renders the same 110 pages CI builds. Without it the site still comes up. It
  just silently loses all 67 corpus pages.

The production deploy does not use this container at all: CI runs `astro build`
and ships the static `dist/` to Cloudflare.
