# Unyform Claude Code Local Plugin — Design

**Status:** Design draft (2026-05-28) · **Owner:** webmech · **Scope:** ADDITIVE — does not replace the SaaS gateway.

## Why this exists

Anthropic enforces a client-identity allowlist on the `claude-opus-* + sk-ant-oat01-*` (Max OAuth) combination. The official Claude Code binary passes; any proxy/HTTP client (our SaaS gateway, curl, raw Node fetch, raw Bun fetch — verified 2026-05-27) gets pre-quota 429s and the per-token throttle deepens with each attempt. See `~/dev/dev916/unyform.ai/.claude/projects/.../memory/anthropic_max_oauth_opus_lockout.md` for the full investigation.

PR #280 added a clean 400-with-actionable-message guard for the Opus+OAuth combo in the SaaS gateway — but the underlying user need remains:

> *"I'm a Max subscriber who uses Opus through Claude Code every day. I want unyform's blueprint context + policy + audit governance on top of my normal CC workflow, without giving up Opus."*

For these users a SaaS HTTP proxy can never work. A **local plugin** can — it runs on the customer's machine, alongside their own CC binary, applying unyform's governance via CC's documented hook system. The customer's CC makes the actual Anthropic API call → official client identity → Opus + Max works exactly as it does today.

## Goals

1. **Optional, not replacing.** Customers using the SaaS gateway today (with API keys, Bedrock/Vertex, or Sonnet/Haiku-via-OAuth) keep what they have. The plugin is a parallel path for customers who specifically need Opus-via-Max.
2. **Same product surface.** Blueprints, policies, audit log live in the same unyform.ai dashboard regardless of which path the customer chose. Authoring is unchanged.
3. **Owned by mech-crate.** Ships as a subcommand of the `mx` CLI, written in the same Rust workspace, sharing config + auth + telemetry primitives with the rest of the toolchain.
4. **Zero gateway hop on the inference path.** Customer's CC talks directly to api.anthropic.com. Plugin only injects context locally and ships audit events post-hoc.

## Non-goals

- Replacing the SaaS gateway. The gateway remains the right answer for Bedrock/Vertex, API-key tenants, multi-org governance, server-side LLM use cases, and any workflow where the customer doesn't run CC interactively.
- Multi-tenant CC hosting. Running CC processes on unyform-owned infra is architecturally and TOS-wise a non-starter (see `architecture-review-2026-03-07.md` if it gets raised again).
- Replacing the IDE plugins on the existing roadmap. The CC plugin is its own deployment surface, parallel to VS Code / JetBrains.

## Architecture sketch

```
┌────────────────────── customer's machine ──────────────────────┐
│                                                                │
│   Claude Code (official binary, customer's OAuth)              │
│        │                                                       │
│        │  CC hook events (SessionStart, UserPromptSubmit,      │
│        │  PostToolUse, Stop) call out via configured commands  │
│        ▼                                                       │
│   ┌─────────────────────────────────────────────────────┐      │
│   │  mx cc-plugin <subcommand>  (new mech-crate cmds)   │      │
│   │  • mx cc-plugin install     — configures hooks      │      │
│   │  • mx cc-plugin session     — SessionStart payload  │      │
│   │  • mx cc-plugin pre-prompt  — UserPromptSubmit     │      │
│   │  • mx cc-plugin post-tool   — PostToolUse payload   │      │
│   │  • mx cc-plugin stop        — Stop / audit flush    │      │
│   └─────────────────────────────────────────────────────┘      │
│        │                                                       │
│        │  HTTPS (long-lived, async, retry-on-failure)          │
│        ▼                                                       │
└────────│───────────────────────────────────────────────────────┘
         │
         ▼
┌──────────────────── unyform.ai SaaS (existing) ────────────────┐
│                                                                │
│   /api/v1/cc/session            (blueprint resolution per repo)│
│   /api/v1/cc/policy-check       (egress policy evaluation)     │
│   /api/v1/cc/audit              (audit log ingest)             │
│   /api/v1/cc/usage              (token-count rollups)          │
│                                                                │
│   Dashboard, blueprint authoring, policy mgmt — UNCHANGED      │
└────────────────────────────────────────────────────────────────┘
                        │
   customer's CC ───────┼────────────────► api.anthropic.com
                        │                  (direct, official client identity,
                        │                   Opus + Max OAuth works)
```

## Hook responsibilities (matched to CC's actual events)

| CC event | What `mx cc-plugin` does | Why |
|----------|-------------------------|-----|
| `SessionStart` | Resolve which blueprints apply (by `git remote get-url origin` + workspace path + org config). Emit a `system-reminder`-tagged context block to stdout — CC injects it into the session preamble exactly the way our SaaS gateway prepends the system block today. | This is the moment to inject blueprint context, equivalent to where `inject_blueprints_anthropic` runs in the gateway. |
| `UserPromptSubmit` | (Optional, configurable) Run input-side policy checks against the user message. Block with a `denyMessage` or pass through. | Mirrors the gateway's input policy pipeline. Off by default — most users will only want blueprint injection + audit. |
| `PostToolUse` | (Optional) Record tool-call metadata for the audit log; redact secrets per egress-scan rules. | Mirrors `egress_scan_enabled` on the gateway. |
| `Stop` | Flush this turn's audit event to `/api/v1/cc/audit`. Compute usage rollup (input/output tokens from CC's transcript) and POST to `/api/v1/cc/usage`. | Centralizes audit + usage tracking with the SaaS dashboard, matching the gateway's audit log. |
| `SessionEnd` | Flush any pending events; mark the session closed. | Cleanup hook. |
| `PreCompact` | (Optional) Snapshot the conversation before CC compacts so audit history isn't lost when context is summarized. | Nice-to-have for compliance customers; off by default. |

Each subcommand is a fast Rust binary call (the same `mx` already on the customer's PATH), so adding the hooks doesn't introduce a Python/Node dependency. Per CC docs, hook commands should return in <1s for `SessionStart`/`Stop` and <300ms for `Pre*ToolUse`, which is well within budget for an HTTPS round-trip + JSON deserialize.

## Auth model

- `mx login` (already exists in mech-crate) signs the developer in to unyform.ai and stores a token in the OS keychain (mirrors `mech-crate/crates/mx-lib/src/auth/*`).
- `mx cc-plugin install` writes the hook entries into `~/.claude/settings.json` (per-user) or `<repo>/.claude/settings.json` (per-repo) and registers the workspace with unyform.ai for blueprint scoping.
- Hook subcommands use the stored token to call the unyform.ai SaaS endpoints. No new auth surface, no per-call OAuth.

## Coexistence with the SaaS gateway

A customer (org) can have **both** the SaaS gateway and the CC local plugin attached to the same unyform.ai account simultaneously:

- The same blueprint, edited once in the dashboard, is served to whichever path is in use.
- Audit log entries from both paths flow into the same audit-log table; they're tagged `source: "gateway"` vs `source: "cc-plugin"` so dashboards can filter or aggregate.
- Org-level rate limits, quotas, and policies apply equally to both.

Customer decision tree:

```
Do you use Claude Code locally with a Max subscription, and need Opus?
  ├── Yes → install the local plugin (this design)
  └── No  → SaaS gateway is the right answer
            ├── If using API keys     → keep using Sonnet/Haiku/Opus
            ├── If using Bedrock      → keep using as-is
            └── If using Max OAuth    → Sonnet/Haiku still work via gateway
```

Some customers will run both — e.g. CC plugin for individual developer use, SaaS gateway for CI/automated agents that don't run CC. That's supported and not weird.

## Where the code lives

- **mech-crate** (this repo): adds the `mx cc-plugin` subcommand and the `mx-lib` shared types for blueprint resolution, policy checks, audit ingest. Aligns with existing `mx-cli`/`mx-lib`/`mx-mcp-server` structure.
- **unyform.ai**: adds the SaaS endpoints (`/api/v1/cc/session`, `/audit`, `/usage`, `/policy-check`) — mostly thin wrappers over existing gateway functions (`load_gateway_config_by_id`, `inject_blueprints_anthropic`'s scoring half, etc.). Significant code reuse.
- **mech-crate as submodule of unyform.ai**: per request, vendor `mech-crate` into `unyform.ai/vendor/mech-crate` (or `crates/mech-crate`) as a git submodule so both repos can be developed together without cross-clone friction. The submodule is consumed only for shared types / test fixtures, not as a runtime dependency of the gateway.

## Migration / rollout (no migration needed)

This is purely additive. No existing customer flow changes.

1. **Phase 0 (this doc):** Design, alignment, decide we want to do this.
2. **Phase 1 — mx subcommand skeleton:** `mx cc-plugin install/uninstall/status`, writes/reads `~/.claude/settings.json`. No SaaS dependency yet.
3. **Phase 2 — SessionStart blueprint injection (read path):** `mx cc-plugin session` calls a new `/api/v1/cc/session` endpoint that resolves blueprints exactly the way the gateway does today. Reuses `inject_blueprints_anthropic`'s scoring + trim logic.
4. **Phase 3 — audit ingest (write path):** `mx cc-plugin stop` posts to `/api/v1/cc/audit`. Dashboard adds a filter for `source: "cc-plugin"`.
5. **Phase 4 — policy + tool egress:** `mx cc-plugin pre-prompt` + `post-tool` for customers who want the full governance surface.
6. **Phase 5 — distribution:** documented install command, mention on docs/onboarding/dashboard for users who hit the new Opus+OAuth 400 from PR #280.

## Open questions

- **How does the plugin know which blueprints apply when the customer has multiple orgs / repos?** Initial approach: `mx cc-plugin install` registers the workspace path + git remote with unyform.ai; the SaaS endpoint resolves from that. Customers on personal forks may need an env var override.
- **Does the SessionStart hook need to be idempotent across CC reloads?** Probably yes — CC fires SessionStart on `/clear` too. Should we re-inject every time, or cache the last injection and skip if blueprint hasn't changed? Lean re-inject — it's cheap and ensures the customer sees blueprint updates immediately.
- **Should the plugin work offline?** First version: no, hook subcommands fail-soft (log a warning to stderr, let CC continue without injection). Customers in air-gapped envs won't be on Max OAuth anyway.
- **TOS clarity.** Reach out to Anthropic informally to confirm this pattern is fine — we're literally just using documented hooks. Should be uncontroversial but worth a sanity check given today's context.
- **MCP server angle.** mech-crate already has an `mx-mcp-server` crate. The CC plugin and the MCP server are different integration surfaces, but both consume `mx-lib`. Worth confirming the layering is clean before Phase 2.

## What this does NOT do

- Does not replace the SaaS gateway. The gateway remains the right answer for non-local workflows.
- Does not change anything about how blueprints/policies/audit log are authored or stored.
- Does not require any changes to today's gateway codebase, including PR #280's Opus+OAuth guard (which becomes the "you should install the plugin" trigger for affected users).
- Does not violate Anthropic's Max TOS (the customer's CC remains the official client; we're using documented extensibility).

## Related docs

- `mech-crate/docs/unyform/ROADMAP.md` — existing IDE plugin direction (this is the same architecture for a different IDE).
- `mech-crate/docs/unyform/TECHNICAL_ARCHITECTURE.md` — hub model and lightweight-client framing.
- `unyform.ai/.claude/projects/.../memory/anthropic_max_oauth_opus_lockout.md` — why we need this at all.
- `unyform.ai/PR #280` — the SaaS-side companion that surfaces "install the plugin" to affected users.
