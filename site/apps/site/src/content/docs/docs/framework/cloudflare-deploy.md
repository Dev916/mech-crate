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
`make cf-setup` fails with "no rule to make target". Check `make help` — if you
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
| `container` | a container image behind a worker — port, sleep timeout, max instances, custom domain |

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

And a researched, as-implemented account of the same flow — including a drift
inventory of the places the documentation and the code disagree, which is worth
reading before you debug a credential problem:

**→ [mx Cloudflare Deploy: as-implemented flow, credentials, and known traps](/docs/corpus/infra/mx-cloudflare-deploy/)**

## How this site ships

mechcrate.dev is built from `site/` in the mech-crate repository — a nested mx
project with the Astro app at `site/apps/site/`. The intended deploy is the
astro recipe's Cloudflare path: a GitHub Actions workflow that runs `astro build`
on pushes to `main` touching `site/**` or `docs/**` and publishes the static
bundle with wrangler, plus a preview deploy on pull requests. The site job is
independent of the Rust gates — it never blocks `ci.yml` and is never blocked by
it.

:::note[Not wired yet]
As of this writing that workflow has not landed: there is no
`.github/workflows/site.yml` in the repository, and `site/` was scaffolded
without `--infra cloudflare`, so it has no `make/cloudflare.mk` either. The
design is settled; the wiring is a later task. This note comes down when the
workflow is green.
:::

Locally the site is an ordinary mx project:

```bash
cd site
make dev            # http://mechcrate.localhost through the mx router
```
