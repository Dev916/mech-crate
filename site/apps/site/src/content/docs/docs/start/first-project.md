---
title: Your first project
description: mx new, mx add, make dev. A service answering on api.localhost in five commands.
sidebar:
  order: 3
---

## The quickstart

```bash
mx router install && mx router up   # once per machine

mx new my-app
cd my-app
mx add api --recipe rust-api --domain api.localhost
make doctor                          # check dependencies
make init                            # initialize environment
make dev                             # develop at http://api.localhost
```

That is the whole path. What follows is what each step actually does.

## `mx new my-app`

Creates the project skeleton (`Makefile`, `make/`, `scripts/`, `apps/` and the
`docker/` tree) and copies the shared Docker config. Nothing is running yet;
this is the [folder contract](/docs/start/folder-contract/) on disk.

Two flags are worth knowing:

```bash
mx new my-app --no-prompt              # skip the interactive prompts
mx new my-app --infra cloudflare       # also install make/cloudflare.mk + infra/cloudflare/
mx new my-app --with api               # add services during creation
```

`--infra` matters more than it looks: without it, the `cf-*` make targets simply
are not in the project. See [Cloudflare deploy](/docs/framework/cloudflare-deploy/).

## `mx add api --recipe rust-api --domain api.localhost`

Applies a recipe. `mx recipes list` shows what is installed and
`mx recipes info <name>` shows a recipe's options, features and the services it
brings with it. `mx add` lands, for a typical recipe:

- an app skeleton under `apps/api/`
- `docker/compose/api.yml` and `docker/compose/api.dev.yml`
- `docker/dockerfiles/api/app` and `app.prod`
- `docker/.config/.env.api`
- Traefik labels wired to the `--domain` you passed

`--domain` is what makes the service reachable at `http://api.localhost` rather
than a port. Omit it and the recipe defaults to `<service>.localhost`. Extra
recipe options go through `--opt key=value`. `mx recipes info` lists which ones
a recipe accepts.

Recipes that also declare backing services (Postgres, Redis) drop their compose
files in at the same time, so `docker/compose/` grows `db.yml` and `redis.yml`
on the first `mx add` that needs them.

:::note[What lands, and what you still run]
`mx add` gives you the operational layer (compose, dockerfile, env, routing) plus
a source tree with the recipe's structure and a health endpoint. Where a recipe
declares a framework initializer, `mx add` runs it: `astro`, `nuxt` and `zola`
call their own starter and print `Scaffolded with: <command>`, so the app arrives
as the framework would have made it with mx's wiring layered on top. You still run
the dependency install (`npm install`, `cargo build`, `composer install`) the
first time. Each recipe prints its exact next steps when it finishes.

A second `mx add` over a service that already has an app skips the scaffolder and
says so. To start that app over from the framework's own starter, the three
recipes with an initializer accept `--opt force_init=true`, which **deletes**
`apps/<service>/` and re-runs the initializer before mx's wiring is layered back
on. It is destructive by design; anything you wrote in that directory goes with
it.
:::

## `make doctor`

Checks Docker, Compose and Make, then the project structure (`Makefile`,
`make/`, `scripts/`, `docker/compose/`, `docker/.config/`, the secrets file and
the Docker network) and lists the services it found. It exits non-zero if
something is missing, so it is safe to put in front of a script.

## `make init`

Creates `docker/.config/.env.secrets` from `.env.secrets.template` if it does not
exist yet, generates a real development value for every credential still empty or
still a placeholder, resolves any `${VAR}` references in the other env files into
literals, and ensures the project network. Idempotent, so run it whenever you are
not sure: a value that is already real, generated earlier or typed by you, is
never overwritten.

That is what makes the quickstart above hold with **no hand edits**. `mx new`,
`mx add` with a recipe that brings Postgres, then `make dev`, and the database
comes up with credentials the application can actually authenticate with.
`make dev` runs the same step on every start, so a service added later is covered
too. There is nothing to fill in first.

`REDIS_PASSWORD` stays blank deliberately, because the shipped Redis runs without
auth and a password there would break clients that build
`redis://:$REDIS_PASSWORD@redis:6379`. `make doctor` knows that and does not flag
it, while it does name any other key left without a value.

## `make dev`

Starts the stack with the dev overrides merged in: source mounts, debug logging,
relaxed health checks. Two shapes:

```bash
make dev                # everything in docker/compose/
make dev s=api          # just this service (and what it depends on)
make dev s="api site"   # a named subset, quotes required
```

The quotes matter: make splits on whitespace, so `s=api site` would read `site`
as a second goal. `dev`, `up`, `down`, `stop`, `restart` and `logs` accept a
list. `build`, `run`, `exec` and `sh` act on one container or image and say so
rather than quietly taking the first name.

`make dev` stops the existing services first, so it is also the "restart into a
clean state" command. `make up` is the same thing without the dev overrides:
the baseline compose files alone, which is what ships.

Then:

```bash
make logs s=api     # tail it
make sh s=api       # shell into it
make ps             # what is running
make down           # stop and remove
```

Full verb list: [CLI reference](/docs/start/cli-reference/).

## Why the hostname works

`api.localhost` resolves to `127.0.0.1` in every current browser and in
`getaddrinfo` on macOS and Linux. Port 80 is the router; the router reads the
`Host` header, matches it against the `Host()` rule in your compose labels, and
forwards to the container over the shared `devmesh-traefik` network. Your service
publishes no HTTP host port at all, which is why two projects can both have a
service listening on 3000 internally and never collide.

## Next

- [The folder contract](/docs/start/folder-contract/): what you just created
- [The router](/docs/framework/router/): how the hostname routing works
- [Recipes](/docs/framework/recipes/): what is available, and what each carries
