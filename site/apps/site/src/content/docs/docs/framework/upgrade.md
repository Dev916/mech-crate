---
title: Upgrade
description: How mx upgrade keeps a project current with the templates, what it will never overwrite, and the two migrations you can feel (the compose project name, and container names).
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

:::note[What this does and does not unblock]
Two projects of the same shape no longer collide on container names. That is the
whole of the claim. Running two same-shape stacks fully concurrently is still
blocked by two separate issues: dev overrides publish fixed host ports for
Postgres and Redis, so the second stack's `5432` is taken, and the Traefik router
names in the labels are not qualified by project, so two projects shipping the
same service name register the same router. Both are tracked
(`mech-crate-1a0`, `mech-crate-298`) and neither is fixed yet.

Different projects with *different* service names and domains do coexist, which
is the ordinary case and what the [router](/docs/framework/router/) is for. It is
specifically two copies of the same shape that hit these limits.
:::

## Recipes are upgraded separately

`mx upgrade` covers the project skeleton. Service-level scaffolding comes from
recipes and has its own path: `mx recipes apply <name>` re-applies a recipe to
the current project, and `mx recipes versions <name>` lists what is available.
See [Recipes](/docs/framework/recipes/).
