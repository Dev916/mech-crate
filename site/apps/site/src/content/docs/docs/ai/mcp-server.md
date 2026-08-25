---
title: MCP server
description: mx-mcp exposes 47 tools in four families — Scaffold, Operate, Understand, Retrieve — so an agent can create a service, run the right subset of the stack, and consult the corpus through one server.
sidebar:
  order: 2
---

`mx-mcp` is a Model Context Protocol server that speaks for any mx project. It
is the same binary whatever the stack, because the [folder
contract](/docs/start/folder-contract/) makes every project the same shape —
that is the whole reason one server can understand all of them.

## Build it and wire a client

```bash
mx mcp build      # release-build the server, write the launch wrapper
mx mcp config     # print client config to paste
```

`mx mcp build` runs a release `cargo build` for the server crate, reports the
binary path (`target/release/mx-mcp`), and writes a launch wrapper to
`~/.mech-crate/mcp/mx-mcp-wrapper.sh`. `mx mcp config` then prints ready-to-paste
JSON for Claude Desktop and Cursor, pointing at that wrapper and pinning
`MECH_CRATE_ROOT` so the server finds your checkout regardless of the client's
working directory. It only prints — nothing is written to your client config for
you.

```json
{
  "mcpServers": {
    "mechcrate": {
      "command": "~/.mech-crate/mcp/mx-mcp-wrapper.sh",
      "env": { "MECH_CRATE_ROOT": "/path/to/mech-crate" }
    }
  }
}
```

The rest of `mx mcp`: `status` (corpus backend and counts, aliased as `ps`),
`run` (run the server on stdio interactively), `info`, `test`.

## The four families

Forty-seven tools. The grouping is not a namespace in the protocol — every tool
is flat — but it is how they divide by what they let an agent *do*.

### Scaffold — 5 tools

`mx_new` · `mx_add_service` · `mx_recipes_list` · `mx_recipe_info` ·
`mx_upgrade`

Create a project, add a service from a [recipe](/docs/framework/recipes/),
inspect what a recipe would install before installing it. This is where an agent
gets a working service instead of a directory of guesses — the recipe carries
the compose files, dockerfile, env conventions and router labels with it.

`mx_upgrade` is currently broken by
[`mech-crate-z5i`](/docs/project/known-broken/); see
[Upgrade](/docs/framework/upgrade/).

### Operate — 18 tools

`make_dev` · `make_up` · `make_down` · `make_logs` · `make_restart` ·
`make_shell` · `make_ps` · `make_help` · `make_key` · `mx_build` ·
`mx_router_install` · `mx_router_up` · `mx_router_down` · `mx_router_status` ·
`mx_router_inspect` · `mx_infra_setup` · `mx_infra_list` · `mx_infra_link`

Run the subset of the ecosystem the task needs, tail its logs, restart one
service, check whether the [router](/docs/framework/router/) is up. The `make_*`
tools are the project's own verbs — the agent invokes the same targets you do,
so there is no second, agent-only code path to keep in sync.

### Understand — 6 tools

`project_analyze` · `project_detect` · `project_list` · `service_info` ·
`mx_doctor` · `mx_help`

Structural questions, answered from the contract rather than by reading files:
what services exist here, which one owns this path, what is this project's
shape, is the local environment healthy. This is the part that shortens the
exploration phase — an agent that can ask does not have to grep.

### Retrieve — 8 tools

`rag_context` · `rag_search` · `rag_search_category` ·
`rag_find_implementation` · `rag_get_guidance` · `rag_compare_approaches` ·
`rag_find_related` · `rag_health`

Hybrid search over the [techniques corpus](/docs/ai/rag-setup/).
`rag_context` is the one to reach for while working — you describe the task and
get back the relevant material; the others narrow by category, language,
comparison or relatedness. `rag_health` reports the backend, document and chunk
counts, embedding model and last ingest, which is how an agent finds out the
corpus is offline instead of concluding the corpus is empty.

Every tool answering from the corpus degrades honestly: with no database
reachable they return a message naming the config file and the container command
to start one, not an empty result.

### Also present

Two document-compilation tools (`mx_docs_compile`, `mx_docs_list`) and eight
`unyform_*` tools for [remote blueprints](/docs/framework/unyform/). The unyform
tools are inert without an account; local recipes need none.

## Deeper

- [`mx-mcp-usage`](/docs/corpus/process/mx-mcp-usage/) — the corpus document on
  driving these tools, retrievable by agents through `rag_context`
- [`mx-rust-cli-and-mcp-server`](/docs/corpus/architecture/mx-rust-cli-and-mcp-server/)
  — how the CLI and the server share `mx-lib`
