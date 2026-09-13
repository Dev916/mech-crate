---
title: "MCP Server Tool Design: Workflow Tools, Progressive Discovery, and Code Mode"
category: api-design
languages: [typescript]
complexity: advanced
use_cases:
  - deciding how many tools an MCP server should expose and at what granularity
  - choosing between 1:1 endpoint mapping, server-side meta-tools, and workflow tools
  - writing tool names, descriptions, schemas, annotations, and error results that agents can use
  - scoping a large tool catalog (categories, grants, read-only) for clients that search tools
  - packaging a tool catalog once and adapting it to MCP, agent frameworks, or a client's code mode
summary: "How MCP server design changed once clients took over tool discovery (progressive discovery) and tool composition (programmatic tool calling / code mode): why 1:1 endpoint mapping bloated context and confused selection, why the server-side search/inspect/execute meta-tool layer is now the client's job, why workflow tools still beat raw endpoints (Neon's three-call branch creation as one createAndConnect tool), the API -> SDK -> tools-package -> adapters layering, and a spec-grounded checklist for names, schemas, outputSchema, annotations, errors, result size, and category scoping."
provenance: researched
researched: 2026-09-12
sources:
  - https://www.youtube.com/watch?v=BqRhBq-_kgE
  - https://modelcontextprotocol.io/docs/2026-07-28/develop/clients/client-best-practices
  - https://modelcontextprotocol.io/specification/2026-07-28/server/tools
  - https://platform.claude.com/docs/en/agents-and-tools/tool-use/tool-search-tool
  - https://platform.claude.com/docs/en/agents-and-tools/tool-use/programmatic-tool-calling
  - https://code.claude.com/docs/en/mcp
  - https://developers.openai.com/api/docs/guides/tools-tool-search
  - https://www.anthropic.com/engineering/writing-tools-for-agents
  - https://www.anthropic.com/engineering/code-execution-with-mcp
  - https://blog.cloudflare.com/code-mode/
  - https://github.com/neondatabase/neon-pkgs/tree/main/packages/tools
  - https://github.com/neondatabase/neon-pkgs/tree/main/packages/sdk
  - https://github.com/neondatabase/mcp-server-neon
---

# MCP Server Tool Design

State of practice as of 2026-09. The seed source is Neon's 2026-09-08 video "MCP Just Got a Whole Lot Better" [1], watched frame-by-frame; every claim it makes about client behaviour was checked against the MCP 2026-07-28 client best-practices page [2], the tools specification [3], and the Anthropic and OpenAI platform docs [4][5][6][7]. Inline `[n]` cites key to `sources`; `[1 @mm:ss]` is a timestamp in the video. Code is illustrative unless it is quoted from a linked README.

## 1. The shift: clients own discovery and composition now

**1:1 endpoint mapping was the first common design, and it has two measured failure modes.** Mapping every API endpoint to its own tool (`POST /projects` -> `create_project`, `GET /branches` -> `list_branches`, ...) gives an agent the whole API surface, but with hundreds of endpoints you get hundreds of tools loaded into the context window before the session starts, and near-duplicate endpoints make the agent pick the wrong tool [1 @00:31-01:10]. The numbers behind that: a typical five-server setup (GitHub, Slack, Sentry, Grafana, Splunk) costs ~55k tokens of definitions before any work happens, and selection accuracy degrades once 30-50 tools are available [4]; the MCP docs' own illustration is ~150,000 tokens of definitions up front versus ~2,000 with on-demand loading [2].

**The server-side fix was a layered meta-tool catalog.** Instead of hundreds of tools, a server exposes two or three: search the available endpoints, inspect one endpoint's definition, execute a request [1 @01:10-01:31]. The MCP docs describe the same three layers as catalog (`search_tools` returning names plus one-line descriptions), inspect (`get_tool_details` returning one full schema), execute [2].

**The big change: that layer moved into the client, under the name progressive tool discovery.** MCP's client best-practices page (protocol revision 2026-07-28) recommends that hosts fetch `tools/list` as usual but defer injecting definitions into context, expose a lightweight `search_tools` meta-tool, and load full definitions only as needed, switching to this mode once definitions cross a threshold of roughly 1-5% of the context window [2]. Shipped implementations:

- Claude Code enables tool search by default from v2.1.232; a request that needs a server's tools triggers a `ToolSearch` call, matching tools are fetched on demand, and servers can start from a discovery cache and connect on first use [6].
- The Claude API takes `defer_loading: true` per tool plus a `tool_search_tool` (regex or BM25 variant); deferred tools stay out of the prompt prefix so prompt caching survives, and the docs advise keeping the 3-5 most-used tools non-deferred [4].
- OpenAI's Responses API has the same `defer_loading` flag with hosted or client-executed search; the model initially sees only name and description, and the guidance is to keep each namespace under ten functions [7].

**Programmatic tool calling ("code mode") moved composition into the client too.** Instead of calling tools one at a time, the agent writes a script that chains tools, runs it in a sandbox, and only the final result returns to the model [1 @02:01-02:19]. The MCP docs formalise it: the host generates a typed API from each tool's arguments and `outputSchema`, the model writes one script against it, the sandbox intercepts calls and brokers them to servers, credentials never enter the sandbox, and only console output comes back [2]. Anthropic's measured example is a workflow that drops from 150,000 tokens to 2,000 (98.7%) when intermediate results stop flowing through the model [9]; the API feature is enabled per tool with `allowed_callers: ["code_execution_20260120"]` and runs in the code-execution container [5]. Cloudflare's Code Mode (2025-09-26) converts MCP schemas to a TypeScript API, runs generated code in disposable V8 isolates, and uses bindings so API keys are never visible to the model [10].

**Consequence:** between progressive discovery handling selection and code mode handling composition, clients now own the problems the server-side meta-tool layer was built to solve [1 @02:19-02:30].

## 2. Do not go back to 1:1: the SDK argument for workflow tools

The obvious inference, "clients handle discovery, so expose every endpoint again", is wrong for the same reason SDKs are not thin wrappers [1 @02:30-02:41]. Any real API has common tasks that require chaining several calls; if the agent has to rediscover that chain every time it wastes tokens and repeats the same mistakes [1 @02:51-03:11, @04:06-04:16]. SDKs give you the raw endpoints *and* bundle common workflows into single operations, and the same idea applies to the tools an MCP server exposes [1 @03:04-03:22].

The worked example [1 @03:22-03:46]:

```text
Branch creation (Neon)               one tool
  create the branch          \
  attach compute resources    >---->  createWithCompute   (video name)
  get the connection string  /        branches.createAndConnect (shipped name [11][12])
```

Anthropic's tool-writing guidance reaches the same design independently: build consolidated, workflow-oriented tools that handle several API calls under the hood (`schedule_event` rather than `list_users` + `list_events` + `create_event`; `search_logs` rather than `read_logs`; `get_customer_context` rather than three lookups), because "more tools don't always lead to better outcomes" [8].

Sizing rule from the source: if the API is simple, 1:1 mapping is fine; where there is provisioning-style complexity, ergonomic shortcuts save the agent from repeating the same mistakes [1 @04:00-04:16].

## 3. Reference architecture: API -> SDK -> tools package -> adapters

Neon's stack, as described in the video and confirmed in the package READMEs [1 @04:16-05:27][11][12][13]:

```text
        +-------------+   +-------------+   +----------------+
        | MCP server  |   | Eve adapter |   | Mastra adapter |
        +------^------+   +------^------+   +-------^--------+
               |                 |                  |
               +-----------------+------------------+
                                 |
                          +------+------+
                          | @neon/tools |   tool descriptors: id, title, description,
                          +------^------+   Zod inputSchema, annotations, execute()
                                 |
                   +-------------+-------------+
                   |          @neon/sdk        |
                   |  ergonomic layer          |   createAndConnect, resetFromParent, ...
                   |  raw endpoints (OpenAPI)  |   every endpoint, 1:1, tree-shakeable
                   +-------------+-------------+
                                 |
                           +-----+-----+
                           |    API    |
                           +-----------+
```

- **SDK**: generated from the OpenAPI spec so it covers every endpoint (`raw`), with a thinner ergonomic layer on top (`createNeonClient`) that adds auth-once, `{ data, error }` results, retries, readiness polling, auto-pagination, and the workflow methods [12].
- **Tools package**: converts both layers into tool descriptors. Selectors are SDK paths (`"branches.createAndConnect"`); the published id is snake_case last-segment-then-resource (`projects.list` -> `list_projects`); every tool carries a Zod input schema, title, description, safety annotations, stability metadata and an `execute()` that strictly validates input, rejects unknown fields, and returns JSON-safe `{ data }` [11].
- **MCP server** imports the package to surface the tools; **adapters** wrap the same package for agent frameworks (Eve, Mastra) [1 @04:56-05:18][11].
- **Categories** let a host scope which tools the agent gets [1 @05:18-05:27]; on the hosted server that is a URL query (`?category=projects&category=branches`, `readonly=true`), previewable via `/api/list-tools`, and `scope` is category metadata, not a read/write switch [13].

The one-line framing: **the server controls what tools are available; the client handles discovering and executing them however it sees fit** [1 @05:27-05:38].

Details in the tools package worth copying [11]:

- Paginated list methods become `.all()` tools with the cursor removed from the schema and a `limit` cap, so the agent never pages by hand.
- Write tools wait for readiness (default five minutes); set that below the host's tool-call timeout or the host gives up first. An abort stops the poll, not the create: list before retrying.
- Some client methods are deliberately not tools (`operations.waitFor`, `credentials.reveal`, `storage.objects.get`): waiting is built into the write tools, and secrets should not be a tool result.
- Hosts can override `descriptions` per tool, rewrite ids (`names`, `name: (id) => \`neon_${id}\``), and inject `project_id`/`branch_id` with `omitFromSchema: true` so the injector is the only source.
- The MCP adapter returns text content and `structuredContent`; failures use `isError: true` with a structured `{ error: { message, kind, status, code, requestId, ... } }`.
- Approval is carried as `neon/requiresApproval` in `_meta` and every non-read operation is marked as requiring it, because MCP annotations are advisory and the protocol does not enforce approval.

## 4. Server-side checklist (spec-grounded)

| Concern | Do this | Why / source |
|---|---|---|
| Names | 1-128 chars, `[A-Za-z0-9_.-]`, unique per server; expect clients to prefix with a server id when aggregating | Spec SHOULDs; server `name` is not unique across servers [3] |
| Namespacing | Prefix by service or resource (`asana_projects_search`); unambiguous parameter names (`user_id`, not `user`) | Search matches whole groups; prefix vs suffix placement measurably changes accuracy [4][8] |
| Descriptions | Write as if onboarding a new hire: make query formats and terminology explicit; include the words users say | Description tuning was the single most effective improvement (15-20% accuracy in Anthropic's Slack/Asana tests); tool search indexes names, descriptions, argument names and argument descriptions [4][8] |
| Input schema | Valid JSON Schema object; no-parameter tools use `{ "type": "object", "additionalProperties": false }`; strict, typed inputs | Spec MUST; strict data models avoid ambiguity [3][8] |
| Output schema | Provide `outputSchema` and return `structuredContent` plus the serialised JSON as text | Servers MUST conform when declared; code-mode hosts generate typed stubs from it, and without it the fallback is `any` [2][3] |
| Annotations | Set `readOnlyHint`/`destructiveHint`/`idempotentHint`/`openWorldHint`, but never rely on them for safety | Clients MUST treat annotations as untrusted; enforce approval in the host policy (Neon: `_meta` flag) [3][11] |
| Errors | Execution errors as `isError: true` with actionable text ("date must be in the future, today is ..."); protocol errors only for unknown tool / malformed request | Clients SHOULD feed execution errors back for self-correction; opaque codes do not help [3][8] |
| Result size | Paginate, filter, truncate; offer `DETAILED` vs `CONCISE` response formats; prefer semantic fields (`name`) over identifiers (`uuid`) | Claude Code caps MCP results at 25k tokens by default (`MAX_MCP_OUTPUT_TOKENS`, per-tool `anthropic/maxResultSizeChars` in `_meta`); concise Slack responses used ~1/3 of the tokens; resolving UUIDs to names improved precision [6][8] |
| List stability | Deterministic `tools/list` order; declare `listChanged` and send `notifications/tools/list_changed`; honour `ttlMs`/`cacheScope` | Stable order improves prompt-cache hits; hosts re-index their search catalog on `list_changed` [2][3] |
| State | No protocol session: return an explicit handle from a creation tool, validate authorization against it every call, state the retention policy in the description, return an execution error on expiry | Spec's stateful-tools guidance [3] |
| Security | Validate all inputs, implement access control, rate-limit, sanitise outputs; never mark secrets with `x-mcp-header` | Spec MUSTs for servers [3] |
| Catalog size | Keep the always-on set small (3-5 non-deferred tools; OpenAI: fewer than ten per namespace) and expose categories so hosts can scope grants | Selection degrades past 30-50 tools; category URLs are how Neon scopes [4][7][13] |

## 5. Evaluate before you ship

Anthropic's loop, which produced the accuracy gains above [8]: generate dozens of realistic prompt/response pairs that need many tool calls; run them and track accuracy, runtime, tool-call count, token consumption and tool errors; read the agent's reasoning and raw transcripts for where it stalled; let an agent rewrite descriptions from the transcripts; keep a held-out set so you are not tuning to the test.

## Agreed vs folklore (compressed)

- **Agreed:** definition bloat and selection degradation are measured, not anecdotal [2][4]; progressive discovery is a client responsibility with three shipped implementations [4][6][7]; code mode is standardised guidance with vendor implementations [2][5][10]; workflow tools remain necessary and vendors converge on it independently [1][8]; annotations are advisory [3][11].
- **Folklore, rebutted by the source:** "clients do discovery now, so expose every endpoint" [1 @02:30-02:41]; "put a search/inspect/execute layer in the server" is now redundant for tool-search clients [1 @01:31-01:50][2].
- **Overstated in the video:** it names Claude Code and Codex as implementing code mode [1 @01:57-02:05]. As of 2026-09-12 Claude Code's MCP docs document tool search but no code-mode execution of MCP tools [6]; programmatic tool calling is a Claude API feature [5]. Treat the Codex half as the video's claim until a primary Codex source is added here.

## Synthesis (inferred)

- **Decision rule.** Count the tools a client will see across all its servers, not just yours. Under ~10 tools with small schemas, 1:1 mapping and upfront loading are fine. Past that, do not add a meta-tool layer; instead ship (a) workflow tools for every multi-call task you see agents repeat, (b) `outputSchema` on everything so code-mode stubs are typed, (c) categories so the host can scope the grant. The server's job is a well-described, scoped catalog; the client's job is retrieval and composition.
- **Keep a server-side meta-tool layer only as a fallback** for clients that cannot search: Claude Code disables tool search under a custom `ANTHROPIC_BASE_URL`, `ENABLE_TOOL_SEARCH=false`, or pre-4.5 models [6]. Gate it behind a flag rather than making it the default surface.
- **`outputSchema` is the cheapest high-leverage change** for existing servers: it costs nothing at call time and is what turns a code-mode stub from `Promise<any>` into a typed function the model can chain without a round trip [2].
- **Applied to this corpus's own servers.** The mx MCP server publishes 47 tools in one flat list; with two or three other servers attached that is inside the 30-50 band where selection degrades [4], so category scoping (as in Neon's `?category=` grants) and a handful of workflow tools (for example one "scaffold and start a service" tool instead of add-service + build + dev + ps) are the two changes this document argues for. vidwatch's seven tools with ranked inline frames are already in the "small catalog, rich results" shape; its remaining gap is `outputSchema` on `watch`/`frames` so a code-mode host can iterate over frames without re-reading the timeline.
