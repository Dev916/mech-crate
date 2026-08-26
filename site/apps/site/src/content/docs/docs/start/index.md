---
title: Start
description: What mx is, and the shortest path from a clone to a service answering on a hostname.
sidebar:
  order: 1
---

`mx` is a Rust CLI that gives every project you work on the same shape. It
scaffolds a project skeleton and per-service definitions from **recipes**, runs a
single workstation-wide Traefik **router** so services answer on hostnames
instead of ports, and exposes the whole thing to AI agents through an MCP server
and a retrievable techniques corpus. Nothing in it is a hosted service: the CLI,
the router and the corpus store all run on hardware you control.

The bet is that consistency compounds. A developer who knows one mx project
knows all of them; an agent that has learned the folder contract once does not
re-derive it per repository. That only pays off if the contract is real, so mx
owns the parts that make it real — `Makefile`, `make/`, `scripts/`,
`docker/compose/` — and leaves `apps/<service>/` and `docker/system/` to you.

## Where to go next

| Page | What it answers |
|---|---|
| [Install](/docs/start/install/) | Getting the `mx` binary and the router onto a machine |
| [Your first project](/docs/start/first-project/) | `mx new` → `mx add` → `make dev`, end to end |
| [The folder contract](/docs/start/folder-contract/) | What every mx project looks like, and who owns which directory |
| [CLI reference](/docs/start/cli-reference/) | Every `mx` verb and every `make` verb, from the shipped `--help` |

Once the basics are in place, [Framework](/docs/framework/) covers the router,
recipes, compose and env conventions, upgrade, testing and deploy; the
[AI Layer](/docs/ai/) covers the MCP server and the techniques corpus.
