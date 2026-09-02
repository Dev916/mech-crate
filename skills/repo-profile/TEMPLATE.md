<!-- tpl: Copy this file to docs/development/repos/{{slug}}.md and replace every
     {{placeholder}} and every `tpl:` comment. check_profile.py fails the profile
     if any marker survives. Spec: docs/superpowers/specs/2026-09-01-repo-profiles-corpus-design.md -->
---
title: "{{name}}: {{one-line what it is}} (Repo Profile)"
category: repos
languages: [{{lowercase languages, most code first}}]
complexity: intermediate
use_cases:
  - "understanding what {{name}} does and where its code lives"
  - "finding {{name}}'s {{CLI / MCP / API / skill}} surface before extending it"
  - "answering 'which repo {{does the thing this repo does}}'"
  - "resuming work on {{name}} in a fresh session"
summary: "{{One paragraph: what it is, what it can do, the stack, and its status — written so a retrieval hit on the summary alone answers the question.}}"
provenance: researched
researched: {{YYYY-MM-DD}}
publish: false
repo: https://github.com/{{org}}/{{repo}}
local_path: {{~/dev/... — the real checkout path, even when it differs from the repo name}}
status: {{active | maintained | dormant | archived | template}}
visibility: {{public | private}}
owner: {{PriceLove LLC (Dev916) | Personal (nyvorin) | Unyform}}
hq_project: {{hq registry slug — omit the key entirely when none exists}}
sources:
  - {{README.md (target repo)}}
  - {{every other repo-relative path you actually read}}
---

# {{name}}

> {{Elevator pitch, one paragraph: what it is, who or what uses it, the problem
> it solves, and what state it is in. This preamble becomes the first retrieval
> chunk — make it self-sufficient.}}

## Identity
| Field | Value |
|---|---|
| Repository | `{{org}}/{{repo}}` ({{visibility}}) — default branch `{{branch}}` |
| Local path | `{{local path}}`{{ note when the directory name differs from the repo name}} |
| Owner / org | {{owner}}{{ · hq project `slug` when one exists}} |
| Status | {{status}} — last commit {{YYYY-MM-DD}}, {{N}} commits, profiled at `{{short sha}}` |
| Languages (by file count) | {{Rust 105 · Markdown 56 · …}} |
| Build system | {{cargo workspace / npm / pyproject / make / compose / anchor / wrangler …}} |
| Runtime deps | {{databases, daemons, OS requirements — names only}} |
| License | {{license or "none declared"}} |
| CI / release | {{workflows, release process, or "none"}} |

## What It Does
<!-- tpl: Problem → solution → who uses it (humans, agents, other repos). What
     "done" looks like for a user. Verified against code, not the README's hopes. -->

## Capabilities
<!-- tpl: The verified inventory, grouped by surface; keep only the groups that
     exist. EVERY bullet ends with a repo-relative path in parentheses. Mark
     README promises the code does not keep under "Not (yet) implemented". -->
### CLI
- `{{cmd}}` — {{what it does}} (`{{path}}`)
### MCP tools
### HTTP API / UI / Skills / Libraries / Background jobs
### Not (yet) implemented

## Architecture
<!-- tpl: Stack with versions; component map (crates/packages/apps); data flow
     (short ASCII diagram welcome); storage (databases, files, formats); external
     integrations; process/concurrency model; security model (auth, where secrets
     live — NAMES only, never values). -->

## Repository Layout
```
{{dir/}}    {{one-line purpose}}
```
<!-- tpl: List entry points explicitly (main.rs, index.ts, SKILL.md, …). -->

## How It Was Built
<!-- tpl: Toolchain and versions; build/run/test/lint commands AS THEY REALLY
     ARE (run nothing with side effects); dev loop (make targets, mx usage,
     router URL rule if applicable); CI/CD and deploy path; configuration and
     env-var NAMES with purpose; provenance — design specs, beads usage,
     agent-built history. -->

## Relationships
<!-- tpl: Depends on (our repos) · used by · shares code or patterns with ·
     supersedes / superseded by · canonical-copy notes. Link sibling profiles by
     path, e.g. docs/development/repos/meetnotes.md — links to profiles that do
     not exist yet are fine. -->

## Notable Techniques
<!-- tpl: Patterns worth knowing or extracting. Link existing corpus docs where
     they exist (docs/development/…). List topics that DESERVE a technique doc as
     "backlog candidates" — but do NOT edit RESEARCH_BACKLOG.md yourself. -->

## State, Gaps and Drift
<!-- tpl: Maturity; README-vs-code drift; TODO/FIXME density; open beads count
     if the repo has .beads; dead code; risks. -->
### Synthesis (inferred)
<!-- tpl: Your own conclusions and connections — ONLY here. Everything outside
     this subsection must trace to a file you read. -->

## Quick Reference
| Task | Command / path |
|---|---|
| Build | {{…}} |
| Run | {{…}} |
| Tests | {{…}} |
| {{URL / install / logs}} | {{…}} |

## Sources
<!-- tpl: The repo-relative paths you read for this profile (mirror of the
     frontmatter `sources:` list, with a word on what each contributed). -->
