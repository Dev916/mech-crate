---
title: CLI reference
description: Every mx verb and every make verb, taken from the shipped --help output.
sidebar:
  order: 5
---

Two command surfaces, deliberately mirrored. `mx` works from anywhere and knows
about global state — templates, recipes, the router. `make` works inside a
project and is the same in every project. Where they overlap (`dev`, `up`,
`down`, `logs`, `sh`, `ps`, `build`), they do the same thing.

Everything below is transcribed from `mx --help`, `mx <verb> --help` and
`make help` against the current build. Ask any of them yourself — they are the
authority, this page is a map.

## `mx`

```
mx [OPTIONS] <COMMAND>

Options:
  -v, --verbose  Enable verbose output
  -h, --help     Print help
  -V, --version  Print version
```

### Scaffolding

| Command | Purpose |
|---|---|
| `mx init` | Install templates to `~/.mech-crate`. `--force` to overwrite, `--update` to refresh templates and keep config |
| `mx new <NAME>` | Create a project. `--with <SERVICE>` (repeatable), `--infra <PROVIDER>`, `--no-prompt` |
| `mx add <NAME>` | Add a service. `--recipe <RECIPE>`, `--domain <DOMAIN>`, `--opt <KEY=VALUE>` |
| `mx recipes list` \| `ls` | List available recipes |
| `mx recipes info <NAME>` | Recipe description, features, options, services |
| `mx recipes apply <NAME>` | Apply a recipe to the current project |
| `mx recipes versions <NAME>` | List available versions for a recipe |
| `mx recipes pull <NAME>` | Pull a recipe from Unyform |
| `mx recipes cache` | Manage cached recipes |
| `mx upgrade` | Upgrade project scaffolding — `--diff`, `--dry-run`, `--yes`. See [Upgrade](/docs/framework/upgrade/) |
| `mx self-update` | Update the `mx` binary itself |

### Operating a project

All of these take an optional `[SERVICE]` and support `-f/--follow` and
`-v/--verbose`:

| Command | Purpose |
|---|---|
| `mx dev [SERVICE]` | Start the development environment |
| `mx up [SERVICE]` | Start services in production mode |
| `mx down [SERVICE]` | Stop services |
| `mx restart [SERVICE]` | Restart a service |
| `mx logs [SERVICE]` | View service logs |
| `mx sh [SERVICE]` | Open a shell in a service container |
| `mx ps [SERVICE]` | List running services |

`mx build <SERVICE>` requires the service name and takes `--prod`, `--dev`,
`-t/--tag <TAG>`, `--push`, `--no-cache`, `--platform <PLATFORM>`.

`mx doctor` checks project health and exits non-zero on a failed check.

### Router

```
mx router install | up | down | restart | status | logs | inspect | network | uninstall
```

`start` aliases `up`, `stop` aliases `down`, `ps` aliases `status`, `info`
aliases `inspect`, and `remove` aliases `uninstall`. Detail on
[The router](/docs/framework/router/).

### Infrastructure

```
mx infra setup [PROVIDER] | list | inspect | link | unlink | remove
```

`ls` aliases `list`. See [Infra credentials](/docs/framework/infra-credentials/).

### AI layer

| Command | Purpose |
|---|---|
| `mx mcp build` | Build the MCP server binaries |
| `mx mcp status` \| `ps` | Corpus backend and counts |
| `mx mcp config` | Print MCP client configuration |
| `mx mcp run` | Run the MCP server interactively |
| `mx mcp info` / `mx mcp test` | Server info / smoke test |
| `mx rag ingest` | Ingest technique docs into the corpus |
| `mx rag status` | Backend, doc/chunk counts, embedding model |
| `mx rag gaps` | Mine research-gap themes from weak-scoring queries |

More on [AI Layer](/docs/ai/).

### Documents

`mx docs [INPUT]` compiles Markdown to PDF/HTML. It takes a file or a directory
and has a large flag surface — `--output`, `--title`, `--author`, `--theme`,
`--order`, `--markdown-only`, `--html-only`, `--no-toc`, `--no-recursive`,
`--logo`, `--company-name`, plus a `docs.json` config mode (`--config`,
`--list`, `--all`, `--doc`). Run `mx docs --help` for the full list, and see the
corpus guide: [`mx docs`](/docs/corpus/framework-guides/docs-command/).

### Unyform

`mx login`, `mx logout`, `mx whoami` — and the same three under
`mx unyform login | logout | whoami`. `mx cc-plugin` installs or uninstalls the
Unyform Claude Code plugin hooks. All optional; see
[Remote blueprints](/docs/framework/unyform/).

## `make`

Every scaffolded project ships these. `make help` prints them with descriptions.
Two conventions run through all of them:

- `s=<service>` (or `service=<service>`) targets one service. Verbs that operate
  on the whole stack treat it as optional; `restart` requires it.
- `t=<tag>`, `c=<command>`, `a=<app>` follow the same short/long pattern
  (`tag=`, `cmd=`, …).

| Target | Purpose |
|---|---|
| `make help` | Show available commands |
| `make init` | Initialize project environment (creates `.env.secrets` from the template) |
| `make doctor` | Check project health |
| `make test` | Run the project's tests |
| `make ps` | List running services |
| `make dev` | Start services in dev mode — `s=[service]` |
| `make up` | Start services in production mode — `s=[service]` |
| `make down` | Stop and remove services — `s=[service]` |
| `make stop` | Stop services without removing — `s=[service]` |
| `make start` | Resume services from saved state |
| `make restart` | Restart a service — `s=[service]` **required** |
| `make logs` | Tail service logs — `s=[service]` |
| `make sh` / `make bash` | Shell into a running service — `s=[service]` |
| `make exec` | Exec a command in a running container — `s=[service] c=[cmd]` |
| `make run` | Run a command in a new container — `s=[service] c=[cmd]` |
| `make build` | Build an image — `s=[service] t=[tag] prod=[0\|1] push=[0\|1]` |
| `make build-dev` / `make build-prod` | Build one image variant explicitly |
| `make build-multiplatform` | Multi-platform production build |
| `make make-key` | Generate a secret — `BYTES=32 FORMAT=hex\|base64\|uuid` |

:::caution[`c=` takes a single word]
`make exec s=api c=bash` works. A multi-word command (`c="ls -la"`) is split by
make and the extra words are read as targets. Shell into the container with
`make sh s=api` and run it there instead.
:::

Release targets (`make release`, `release-patch`, `release-minor`,
`release-major`, `release-dry`, `release-first`, `release-simple*`,
`release-push`, `release-full`, `release-sync`, `release-changelog`,
`release-version`, `release-list-apps`) all take `app=[app]` and drive the
per-app version/tag flow.

Cloudflare targets (`cf-setup`, `cf-deploy`, `cf-logs`, …) exist **only** in
projects created with `--infra cloudflare`; they are listed on
[Cloudflare deploy](/docs/framework/cloudflare-deploy/).

## Developing mx itself

The repository's own `Makefile` is a different surface — `make build`,
`make test`, `make lint`, `make check`, `make coverage`,
`make test-known-broken`, `make test-e2e`, `make test-mutants`,
`make install-local`. See [Testing](/docs/framework/testing/), and the corpus
cards: [MechCrate CLI Quick Reference](/docs/corpus/process/quick-reference/)
and [MX Rust CLI &amp; MCP Server Quick Reference](/docs/corpus/process/mx-quick-reference/).
