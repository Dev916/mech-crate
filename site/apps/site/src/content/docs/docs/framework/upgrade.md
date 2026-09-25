---
title: Upgrade
description: How mx upgrade keeps a project current with the templates, what it will never overwrite, and the four migrations you can feel (the compose project name, container names, host ports, and Traefik label names).
sidebar:
  order: 6
---

The premise of the folder contract is that mx owns the verb layer, so
improvements to `make dev` can reach a project scaffolded a year ago. `mx
upgrade` is the command that carries them.

```bash
mx upgrade --dry-run   # show what would be done, change nothing
mx upgrade --diff      # show a diff for each changed file
mx upgrade             # walk the changes interactively
mx upgrade --yes       # accept everything, non-interactive
```

## What a run looks like

Discovery reads the layout that actually ships: `templates/Makefile.template`,
`templates/make/*.mk`, the top-level files of `templates/scripts/` and
`templates/docker/`. A project scaffolded before the compose isolation change
below sees exactly the ten script files that changed:

```
$ mx upgrade --dry-run

   🦝 MechCrate Project Upgrade

→ Upgrading project at: /path/to/legacyapp

Checking directories...

Scanning templates...

┌────────────────────────────────────────────────────────────┐
│  📝 Tooling Updates Available                              │
└────────────────────────────────────────────────────────────┘

The following tooling files have updates available:

    • scripts/.bashrc
    • scripts/doctor.sh
    • scripts/down.sh
    • scripts/exec.sh
    • scripts/logs.sh
    • scripts/ps.sh
    • scripts/run.sh
    • scripts/sh.sh
    • scripts/stop.sh
    • scripts/test.sh

  [DRY RUN] Would prompt to update 10 file(s)

┌────────────────────────────────────────────────────────────┐
│  📊 Upgrade Summary                                        │
└────────────────────────────────────────────────────────────┘

  [DRY RUN] No changes were made


🦝 Crate Raccoon says: Your tooling is fresh!
```

Exit code `0`. `mx upgrade --yes` then applies that plan and keeps a `.bak`
beside every file it replaces, and the run after it reports `✓ Project is up to
date!`. The command is safe to repeat: it compares content, so a project already
current is a no-op rather than a second round of prompts.

:::note[This was broken until recently]
Discovery used to look for a `templates/project/` directory the shipped layout
never contained, so every invocation failed before doing anything. That was
`mech-crate-z5i`, and the test written to define it,
`upgrade::tests::upgrade_discovery_works_against_real_templates_layout`, lost its
`#[ignore]` and joined the gate suite when the fix landed. A second test,
`upgrade_discovery_scope_mirrors_mx_new`, pins discovery's scope to what `mx new`
actually writes, which is what stops a file like `scripts/.bashrc` falling back
out of reach. See [Testing](/docs/framework/testing/) for how that lane works.
:::

## The categories

Which files an upgrade may touch follows the same rule that governs who owns what
in the [folder contract](/docs/start/folder-contract/). Every template file is
categorised:

| Category | Paths | Behaviour |
|---|---|---|
| **Tooling** | `Makefile`, `make/*.mk`, every top-level file of `scripts/` (including the extensionless `scripts/.bashrc`) | Offered for update (these are mx's) |
| **Config** | `docker/compose/`, `docker/config/`, `docker/dockerfiles/`, `docker/system/` | Added if missing, **never overwritten** (these are yours) |
| **Conditional** | `make/cloudflare.mk`, `scripts/cf-*.sh`, `infra/cloudflare/` | Only touched if the project has `infra/cloudflare/` |
| **Skip** | `recipes/`, `router/`, nested script bundles such as `scripts/md2pdf/` | Not part of a project upgrade |

The line that matters: **your compose files and dockerfiles are never replaced.**
An upgrade can improve how `make dev` composes them; it cannot rewrite what they
say. That is what makes accepting an upgrade a low-stakes decision rather than a
merge conflict.

Conditional files are keyed on evidence rather than a config flag. The
Cloudflare set is in scope exactly when `infra/cloudflare/` exists on disk, so a
project that never opted in never sees those files offered.

Skip is a scope rule, not an oversight. `mx new` copies the top-level files of
`templates/scripts/` and nothing deeper, so an upgrade that laid down
`scripts/md2pdf/` would be handing an existing project a subtree a fresh project
never gets. Upgrade's reach and `mx new`'s reach are the same set, and a test
holds them there.

## Migrating: the compose project name

One upgrade in this set changes behaviour you can feel, so it is worth reading
before you accept it.

Every mx project now pins `COMPOSE_PROJECT_NAME` to its sanitised
project-directory name. `scripts/.bashrc` is the single source for that value,
and every script that shells out to compose passes `-p "$COMPOSE_PROJECT_NAME"`.
Two mx projects on one machine therefore no longer share a namespace, and cannot
adopt or recreate each other's containers.

The migration cost is that a project's namespace changes. Containers started
before the upgrade ran under the default name compose derives from the compose
files' parent directory, which is the string `compose` for every mx project on
the machine. After upgrading, `make down` and `make ps` look in the new namespace
and no longer see them. They are orphans, still running, invisible to the
project's own verbs.

`make doctor` is where you find them:

```
ℹ Checking compose project name...
✓ Compose project name: legacyapp
✓ No orphans left under the old 'compose' project name
ℹ Another stack is using the shared default project name 'compose':
    - other-db (service 'db', from /path/to/another-project/docker/compose)
ℹ Left alone - pinning 'legacyapp' is what keeps this project out of it.
```

When there is something of yours to clean up, the same check names each container
and tells you to remove them once with `docker rm -f <name>`, then run `make dev`
again. Ownership is settled by compose's own `project.working_dir` label, which is
how a colleague's stack sitting in the shared namespace gets reported and left
alone rather than offered up for deletion.

Two limits are worth knowing. A `docker compose -f docker/compose/api.yml ...`
typed by hand, outside `make` and the scripts, still gets the default name, so
pass `-p` yourself if you work that way. And two project directories that happen
to share a directory name still derive the same project name, so either name them
distinctly or export `COMPOSE_PROJECT_NAME` in your own environment, which wins
over the default.

## Migrating: container names

A second behavioural change rides in the same set, and it is the one most likely
to break something you wrote.

The shipped compose files used to pin `container_name: db`, `container_name: api`
and so on. With project names pinned per project, that became a hard blocker
rather than a cosmetic choice: container names are a Docker-daemon-wide
namespace, so two projects whose compose files both declared `db` could not run
at the same time. The second one failed with
`Conflict. The container name "/db" is already in use`. Those pins are gone from
every template and every recipe, and a conformance test keeps them gone.

Compose now derives the name itself:

```
<COMPOSE_PROJECT_NAME>-<service>-<index>      e.g. myproj-db-1
```

**Service names did not change.** `depends_on`, the `include:` graph, the Traefik
labels and every `s=<service>` on the `make` line address services, so all of
that behaves exactly as before. Upgrading does not change a single compose file
of yours either, because compose files are yours and an upgrade never rewrites
them. What changes is what you get after you adopt a newer template or apply a
newer recipe, and what breaks meanwhile is anything that addressed a *container*
by a fixed name:

```bash
# before
docker exec db psql -U postgres
docker logs -f api

# after
make exec s=db c=psql
make logs s=api
docker compose -p "$COMPOSE_PROJECT_NAME" exec db psql -U postgres
docker compose -p "$COMPOSE_PROJECT_NAME" logs -f api
```

The project's own verbs are the stable interface, and they were always the
intended one. Reach for the explicit `docker compose -p` form when you need
something the verbs do not cover.

One exception: the global router container is still `mx-router`. It is installed
once per machine rather than per project, so it has nothing to collide with, and
`mx router` addresses it by that name.

:::note[What this unblocks]
Container names were the first of three things standing between two same-shape
stacks and running side by side. The other two landed with the migrations below:
host ports stopped being pinned, and Traefik label names started carrying the
compose project. Two copies of the same recipe now `make dev` concurrently with no
hand edits.

One thing is still yours to do, and it is the hostname. Both stacks run, but
Traefik serves a given host to exactly one of them, so the second stack exports
`<SERVICE_UPPER>_ROUTER_HOST` to claim its own. See
[the router](/docs/framework/router/#two-ecosystems-at-once).
:::

## Migrating: host ports

Every dev override used to publish a fixed host port, which is why a second stack
died on `Bind for 0.0.0.0:5432 failed: port is already allocated` with nothing in
the project to edit short of hand-patching a shipped template. The repair asked
one question per port: who dials it?

Ports that **tooling** dials on demand now publish on host port `0`, which Docker
allocates from the free range, so they can never collide:

| Container port | Was | Pin it with |
|---|---|---|
| Postgres 5432 | `5432:5432` | `DB_HOST_PORT` |
| Redis 6379 | `6379:6379` | `REDIS_HOST_PORT` |
| Node inspector 9229 | `9229:9229` | `NODE_DEBUG_HOST_PORT` |
| Worker metrics 9090 | `9090:9090` | `METRICS_HOST_PORT` |
| Vite 5173 (laravel) | `5173:5173` | `VITE_HOST_PORT` |
| Inertia SSR 13714 | `13714:13714` | `INERTIA_SSR_HOST_PORT` |

Discover the allocated one instead of guessing:

```bash
docker compose -p "$COMPOSE_PROJECT_NAME" port db 5432
```

Or pin one for a session, leaving its siblings ephemeral:

```bash
DB_HOST_PORT=5432 make dev
```

Ports a **browser** dials from a URL it already holds keep today's number as a
default and gain an override, so an overriding second stack moves instead of
failing to boot: `LEPTOS_RELOAD_HOST_PORT` (3001), `ZOLA_LIVERELOAD_PORT` (1024),
`NGINX_HTTP_PORT` / `NGINX_HTTPS_PORT`, `TRAEFIK_HTTP_PORT` /
`TRAEFIK_HTTPS_PORT` / `TRAEFIK_DASHBOARD_PORT`, and the global router's own
`MX_ROUTER_HTTP_PORT` / `MX_ROUTER_HTTPS_PORT`.

:::caution[24678 is gone, not parameterized]
The astro and nuxt dev overrides published `24678:24678` labelled "Vite HMR", and
so did this repo's own site. Nothing was listening on it: a live astro dev
container has 4321 open and nothing else, nuxt has 3000 and nothing else. Both
frameworks carry HMR over the app's own port, and the router upgrades that
websocket same-origin. Docker binds a published host port whether or not anything
answers behind it, so the publish cost a real port for nothing: an astro stack
holding 24678 killed a nuxt stack's boot outright. The three publishes are gone,
along with the `EXPOSE 24678` in each dev Dockerfile that made them look
justified. HMR still works, through the router, at the app's own hostname.
:::

What breaks is a script or GUI connection profile pointed at `localhost:5432`,
`localhost:6379`, `localhost:9229`, `localhost:9090`, `localhost:5173`,
`localhost:13714` or `localhost:24678` of an mx dev stack. Discover the port or
pin it.

## Migrating: Traefik label names

`traefik.http.routers.<service>.*` is now
`traefik.http.routers.${COMPOSE_PROJECT_NAME}-<service>.*`, and the same for
`.services.` and `.middlewares.`.

Traefik keeps one router table, one service table and one middleware table per
provider for the whole machine, so the old unqualified names meant two projects
that each scaffolded an `api` service wrote the same key. Mismatched labels made
Traefik drop the router outright and 404 both hostnames; matching labels were
quieter and worse, merging both containers into one load-balancer pool so
consecutive requests alternated between two projects' apps with nothing logged.

Compose resolves `COMPOSE_PROJECT_NAME` from the project name it already
resolved, so nothing has to be exported and the qualification holds under a
hand-rolled `docker compose -f …`. **Hostnames are unchanged**, and the routing
rule gained a per-service override in the default position:

```yaml
- traefik.http.routers.${COMPOSE_PROJECT_NAME}-api.rule=Host(`${API_ROUTER_HOST:-api.localhost}`)
```

What breaks is a script or dashboard bookmark that addressed a router or service
entry by its bare service name. `api@docker` is now `myproj-api@docker`. The
global `mx-router` container is unaffected: it configures Traefik by file rather
than by label, and one machine has one router.

## Recipes are upgraded separately

`mx upgrade` covers the project skeleton. Service-level scaffolding comes from
recipes and has its own path: `mx recipes apply <name>` re-applies a recipe to
the current project, and `mx recipes versions <name>` lists what is available.
See [Recipes](/docs/framework/recipes/).
