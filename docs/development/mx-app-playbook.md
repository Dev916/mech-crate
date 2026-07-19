---
title: "mx App Playbook: Building, Migrating, and Routing Apps the MechCrate Way"
category: process
languages: []
complexity: intermediate
use_cases:
  - starting any new app with the mx framework front to back
  - migrating an existing app onto the mx folder contract
  - wiring every app URL through the mx router instead of localhost ports
  - troubleshooting a service URL that will not resolve through the router
summary: The agent-facing operating manual for building apps WITH mx — philosophy, project anatomy, scaffolding flow, migration strategy, and the always-use-mx-router rule with URL discovery and troubleshooting.
provenance: researched
researched: 2026-07-19
sources:
  - README.md (mx repo)
  - docs/router.md (mx repo)
  - docs/product-structure.md (mx repo)
  - templates/ (mx repo — Makefile.template, make/, scripts/, recipes/, router/)
  - crates/mx-cli/src/commands/{new,add,init,upgrade,router,doctor}.rs (mx repo)
  - crates/mx-lib/src/{project,paths,recipe,router,upgrade}/ (mx repo)
  - crates/mx-mcp-server/src/tools/mod.rs (mx repo)
  - bin/lib/router.sh (mx repo)
  - ~/.claude/skills/devloop/references/url-discovery.md
---

# mx App Playbook

The operating manual for building apps **with** the mx (MechCrate) framework. Written for agents: follow this and every app gets the same anatomy, the same tooling, and — non-negotiably — **URLs through the mx router, never bare localhost ports**. All bracketed citations are repo-relative paths in the mech-crate repo; this doc was researched directly from source.

## 1. The Why

MechCrate is "a reusable set of Docker and Docker Compose conventions plus Makefile modules plus scripts that let you drop an app into a known structure and start building immediately" [README.md:61-70]. It standardizes four things: local development (compose with dev overrides), environment config (`.env` files split per service), filesystem layout (1:1 host-to-container mapping), and a Make-based CLI that is identical across every project.

The payoff is compounding: every project answers "how do I run this / see logs / shell in / build it" identically, agents never rediscover project conventions, and the global Traefik router eliminates port allocation entirely — every app is `http://<name>.localhost`, no conflicts, no winging it.

**The folder contract is mandatory**: "If a repo uses MechCrate, the structure must exist exactly like this" [README.md:136-163]. Partial adoption breaks tooling and agent detection (§3).

Three core conventions [README.md:165-186, docs/product-structure.md:39-44]:
1. **Centralized env layering**, loaded in order: `.env.shared` → `.env.secrets` → `.env.<service>`.
2. **Atomic service files** — each service is its own compose file in `docker/compose/`; stacks are composed by passing multiple `-f` files.
3. **Baseline + dev override** — production runs `service.yml` only; development runs `service.yml` + `service.dev.yml`. That one difference is the entire dev/prod split.

## 2. Project anatomy (what every mx app looks like)

`mx new <name>` emits exactly this [crates/mx-cli/src/commands/new.rs:56-163]:

```
my-app/
├── Makefile              # thin dispatcher; includes make/*.mk [templates/Makefile.template]
├── README.md  .gitignore # generated
├── apps/                 # service source code lands here (one dir per service)
├── make/                 # one .mk per command: dev, up, down, logs, sh, build, restart, ...
├── scripts/              # the real logic; make targets delegate here (.bashrc = compose-layering lib)
├── docker/
│   ├── .config/          # .env.shared, .env.secrets(.template), .env.<service>
│   ├── compose/          # ONE yml (+ one .dev.yml) PER SERVICE — starts EMPTY
│   ├── system/           # 1:1 container filesystem mounts
│   └── dockerfiles/      # <service>/app + app.prod
└── tmp/up/               # snapshot of the exact compose -f set last started
```

Load-bearing facts:
- A fresh `mx new` project has **no services** — `docker/compose/` is empty and `make dev` tells you to `mx add` first [templates/scripts/dev.sh]. Services only enter via recipes.
- **Detection markers**: a directory is an mx project when `Makefile` + `docker/` exist (lenient CLI check) — but agent-facing MCP tools require all four of `Makefile`, `docker/`, `make/`, `scripts/` [crates/mx-lib/src/project.rs:45-66; crates/mx-mcp-server/src/project/mod.rs:42-47]. Migrations must create all four.
- **Compose layering mechanics**: `scripts/.bashrc` assembles the `-f` file list — baseline `docker/compose/<svc>.yml`, plus `<svc>.dev.yml` only in dev mode, plus an optional arch-specific `$(uname -m).yml` — and snapshots the resolved set into `tmp/up/` so `down`/`logs`/`ps` operate on exactly what was started [templates/scripts/.bashrc:36-180].
- Daily loop: `make dev [s=svc]` (downs first, then up WITH dev overlays) · `make up` (prod mode, no overlays) · `make down` · `make logs s=svc` · `make sh s=svc` · `make ps` · `make build s=svc [prod=1]` [templates/make/*.mk]. Agents have 1:1 MCP wrappers (`make_dev`, `make_up`, `make_logs`, ...).

## 3. Starting a new app, front to back

```bash
mx init                       # once per machine: installs ~/.mech-crate (templates, recipes, router)
mx router install && mx router up   # once per machine: global Traefik (§5)
mx new my-app [--infra cloudflare]  # scaffold the project shell
cd my-app
mx add web --recipe=nuxt            # or: astro | laravel | rust-api | rust-leptos | rust-worker | zola
mx add api --recipe=rust-api --domain api.my-app.localhost
make dev                            # up with hot reload
# → http://web.localhost, http://api.my-app.localhost — NO ports
```

What `mx add <name> --recipe=<r>` does [crates/mx-cli/src/commands/add.rs; crates/mx-lib/src/recipe/installer.rs:71-123]: resolves the recipe's `recipe.json` (services, options, placeholders, templates, post_install), optionally runs the framework's own scaffolder (`init_app`, e.g. `npx nuxi init`), then Tera-interpolates templates into place: app source → `apps/<name>/`, compose baseline → `docker/compose/<name>.yml`, dev override → `<name>.dev.yml`, dockerfiles → `docker/dockerfiles/<name>/`, env → `docker/.config/.env.<name>`. The `--domain` option defaults to `<name>.localhost` and feeds the Traefik Host rule [templates/recipes/rust-api/recipe.json:23-26].

Recipe fragments come router-wired out of the box — the five Traefik labels, the `devmesh-traefik` network, and **zero published app ports** (§5). Databases/redis ride along via `common://` fragments and stay off the router [templates/recipes/common/docker/compose/db.yml].

Gotchas (verified in source):
- `mx new` refuses to run in an existing directory [new.rs:37-39] — for existing apps use §4, not `mx new`.
- The top-level `templates/docker/compose/*.yml` (app.yml, nginx.yml, traefik.yml, worker.yml) are **legacy reference samples**: they publish host ports, join `mech-network`, and carry no router labels. They are the anti-pattern this playbook exists to prevent — never copy them into a project. Canonical patterns live in `templates/recipes/*/docker/compose/service.yml`.
- Re-running `mx add` with an existing service name overwrites template-managed files unconditionally [installer.rs:252-299] — safe for regeneration, destructive for hand-edited compose files.

## 4. Migrating an existing app onto mx

There is no `mx migrate`; `mx new` won't wrap existing code, and **`mx upgrade` is currently broken against the template layout** (it expects a `templates/project/` dir that does not exist — [crates/mx-lib/src/upgrade/mod.rs:141-148]; fix tracked in the mx repo). Migration is a deliberate, mostly mechanical procedure:

### Synthesis (inferred)

The following migration strategy is inferred from the folder contract, the recipe installer mechanics, and the detection rules above — it is the procedure an agent should follow, not a documented mx feature:

1. **Scaffold a sibling, steal the shell.** Run `mx new <app>-mx` next to the existing repo, then copy `Makefile`, `make/`, `scripts/`, `docker/` (with its `.config/compose/system/dockerfiles` skeleton), and `tmp/` into the real repo. This satisfies the strict 4-marker detection contract in one move and guarantees current-template tooling.
2. **Move source under `apps/<service>/`.** One directory per deployable unit. Keep the app's own package management untouched — mx wraps, it does not replace.
3. **Write the service compose pair by cloning the closest recipe**, not the legacy samples: copy `templates/recipes/<closest>/docker/compose/service.yml` + `service.dev.yml`, substitute the `{{SERVICE_NAME}}`/`{{DOMAIN}}`/port placeholders by hand (or run `mx add <name> --recipe=<closest>` in a scratch project and copy the rendered output). Adjust `loadbalancer.server.port` to the app's internal port. Delete every `ports:` mapping for the app itself.
4. **Split env** into `docker/.config/.env.shared` (project-wide), `.env.secrets` (gitignored; template checked in), `.env.<service>` — matching the documented load order.
5. **Dockerfiles** go to `docker/dockerfiles/<service>/app` (dev) and `app.prod`, built from repo root [docs/product-structure.md:39-41]; dev override sets `build.target: development` + source bind-mount `../../apps/<service>:/app:cached` for hot reload.
6. **Verify the contract**: `mx doctor` (checks structure, not router), then `make dev` and confirm the URL resolves through the router (§5). An agent should treat `project_detect` via MCP returning true as the done-signal for the structural half.
7. **Iterate one service at a time.** A half-migrated repo where the old `docker-compose.yml` still binds ports 80/443 will fight the router (§6, port conflicts) — retire legacy compose files as each service moves.

## 5. The Router Rule: every app URL goes through mx router

**The rule (non-negotiable): apps are reached at `http://<service>.localhost` via the global mx router. Never publish an app port, never hand out `http://localhost:<port>`.** Dev-tool ports (HMR websockets, a dev-only 5432/6379) are the only sanctioned `ports:` entries [templates/recipes/*/docker/compose/service.dev.yml].

Architecture [templates/router/docker-compose.yml; crates/mx-lib/src/router/mod.rs; docs/router.md]: one global Traefik v3 container (`mx-router`) owning host ports 80/443 plus a dashboard on an auto-allocated port (7680–7799, cached in `~/.mech-crate/router/.dashboard-port`). It watches the Docker socket (`exposedByDefault: false`) and routes to any container that opts in via labels on the external `devmesh-traefik` network. State lives in `~/.mech-crate/router/` (static config, hot-reloaded `config/dynamic/`, `letsencrypt/`). Routing updates are live — starting/stopping labeled containers re-routes with no router restart.

**The five labels that give a service its URL** (canonical block, identical across recipes) [templates/recipes/rust-api/docker/compose/service.yml:22-44]:

```yaml
networks:
  - default            # talk to db/redis
  - devmesh-traefik    # be reachable by the router
labels:
  - traefik.enable=true
  - traefik.http.routers.<name>.rule=Host(`<name>.localhost`)
  - traefik.http.routers.<name>.entrypoints=web
  - traefik.http.services.<name>.loadbalancer.server.port=<internal-port>
  - traefik.docker.network=devmesh-traefik
# ...and at file bottom:
networks:
  devmesh-traefik:
    external: true
```

`loadbalancer.server.port` is the app's **internal** port (3000 for nuxt/rust-api, 80 for laravel, 4321 for astro) — it is never published to the host. Dev and prod carry identical routing labels; the URL does not change between `make dev` and `make up`.

**URL discovery (how an agent finds the URL — instead of guessing localhost):**
1. Parse the service's compose file with a YAML parser (never regex) for `traefik.http.routers.<n>.rule=Host(\`h\`)`; scheme is `https` iff a sibling `tls=true` label exists; result is `<scheme>://<host>` with no port [~/.claude/skills/devloop/references/url-discovery.md].
2. Or `mx router inspect` — lists dashboard URL and connected services (the shell implementation enumerates `docker network inspect devmesh-traefik`) [bin/lib/router.sh:184-204].
3. Reachability gate: `curl -k -I -m 5 <url>` retried up to 30s; 2xx/3xx/401/403 all count as up.
4. Zero or multiple Host matches → stop and disambiguate with `mx router inspect` + `mx ps`; never fall back to a localhost port.

Lifecycle: `mx router install` (once; copies `templates/router/` → `~/.mech-crate/router/`, creates the network) · `up` / `down` · `status` · `logs` · `inspect` · `network` [crates/mx-cli/src/commands/router.rs]. Known implementation gaps: the Rust CLI's `inspect` is an alias of `status` and `reload` (SIGHUP hot-reload) exists only in the shell lib [router.rs:175-177; bin/lib/router.sh:178-182]. HTTPS locally = mkcert cert + `config/dynamic/certs.yml` + switch labels to `entrypoints=websecure`/`tls=true`; production = ACME resolver + `tls.certresolver=letsencrypt` label [docs/router.md:364-421].

## 6. Router troubleshooting (when the URL doesn't work)

Work this ladder top-down; each symptom isolates a different layer [docs/router.md:488-553 unless cited otherwise]:

| Symptom | Meaning | Fix |
|---|---|---|
| `curl http://x.localhost` → connection refused | Router itself is down (nothing owns :80) | `mx router status`; `mx router up`; if "not installed" → `mx router install` |
| Router won't start: `bind: address already in use` | Something else owns 80/443 | `lsof -i :80` / `lsof -i :443`; usual suspects: Apache, nginx, a legacy per-project traefik/nginx compose file — retire it |
| **404** from Traefik | Router is up but no route matches | Missing `traefik.enable=true` or `traefik.docker.network=devmesh-traefik` label, container not on the `devmesh-traefik` network, or Host rule typo. Verify: `docker inspect <c> --format '{{json .Config.Labels}}' \| jq` and `docker network inspect devmesh-traefik` |
| **502** from Traefik | Route matches, backend unreachable | Container crashed/unhealthy, wrong `loadbalancer.server.port`, or app still booting — `make logs s=<svc>` |
| Compose up fails: network `devmesh-traefik` not found | Router never installed/started on this machine | `mx router install` (creates the network) or `mx router network` |
| Dashboard URL unknown | Port is auto-allocated | `mx router inspect` or `cat ~/.mech-crate/router/.dashboard-port` (range 7680–7799) |
| Labels changed but routing didn't | Stale container config | Recreate the service (`make restart s=<svc>`) — labels are read at container start; dynamic-file changes hot-reload, label changes need a container recreate |

Diagnostic bundle to gather when stuck: `mx router status`, `mx router logs`, `docker network inspect devmesh-traefik`, the service's rendered labels, and `make ps`. Note: **`mx doctor` performs zero router checks** [crates/mx-cli/src/commands/doctor.rs] — passing doctor says nothing about routing.

### Synthesis (inferred)

- **Escalation rule for agents**: if the ladder above doesn't produce a working URL in two passes, the correct behavior is to surface the diagnostic bundle and ask — not to silently fall back to publishing a port. A temporary `ports:` mapping "just to unblock" is how projects rot off the router; it defeats the entire convention and should be treated as a migration regression.
- **`.localhost` resolution**: the scheme relies on `.localhost` resolving to 127.0.0.1 (RFC 6761 behavior; modern browsers and macOS do this natively). curl on some setups may differ from the browser — if curl fails but the browser works, test with `curl --resolve <host>:80:127.0.0.1`. The mx docs carry no OS-level resolver caveats today; if a machine genuinely doesn't resolve `.localhost`, an `/etc/hosts` entry per hostname is the workaround.
- **Host-run (non-Docker) services** can still get router URLs via a dynamic-file service pointing at `host.docker.internal` [docs/router.md:481] — useful mid-migration when a service hasn't been containerized yet.

## 7. Agent quick-reference

- New app: `mx new` → `mx add --recipe` → `make dev` → URL from compose Host labels. Never invent ports.
- Existing app: §4 procedure — shell first, services one at a time, router from day one.
- Before touching a project: `project_detect` (MCP) or check the 4 markers; before running anything: `mx router status`.
- The URL is in the compose file. Read it; don't guess it.
