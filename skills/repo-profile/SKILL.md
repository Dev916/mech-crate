---
name: repo-profile
description: 'Use when authoring or refreshing a repo profile for the techniques corpus — a docs/development/repos/<slug>.md doc that tells agents what one of our repositories does, what it is capable of, how it was built, and how it relates to the others. Triggers: a beads task titled "Profile: <repo>", the technique-research stale-profile rung, or "profile <repo> for the corpus".'
---

# Repo Profile — Author or Refresh One Repository's Corpus Profile

A repo profile is a retrieval document. The chunker splits on `##` and prefixes every chunk with `Doc Title > Heading`, so the **fixed eleven-section shape is the contract**: identical `##` headings across all profiles make "what can X do" hit `… > Capabilities` for every repo. Your instinct will be to write long self-descriptive `##` headings — that instinct is right about retrieval and wrong about the contract. Put the descriptive richness in `###` subheadings *inside* the fixed sections; the `##` set never changes.

One profile = one file (`docs/development/repos/<slug>.md`) = one branch (`corpus/repo-<slug>`). Spec: `docs/superpowers/specs/2026-09-01-repo-profiles-corpus-design.md`.

## Phase 0 — Identify

Collect the Identity facts before reading code:

```bash
git -C <repo> remote get-url origin && git -C <repo> log -1 --format='%h %cs' \
  && git -C <repo> rev-list --count HEAD && git -C <repo> branch --show-current
gh api repos/<org>/<repo> --jq '{language,visibility:.private,pushed_at,default_branch,license:.license.spdx_id}'
```

Language mix by file count (skip node_modules/target/.git/vendor/dist). Record the **real local path even when the directory name differs from the repo name** (e.g. `~/dev/dev916/gnn2` is nyvorin/nexus-tokyo, `~/dev/dev916/stack` is devmesh-traefik) — record both names.

## Phase 1 — Dedup

`mcp__mx__rag_search` for the repo name (fall back to `grep -rl <name> docs/development/*.md`).
- **NEW** — nothing meaningful → author from the template.
- **IMPROVE** — a profile exists → surgical edits only: update stale sections, bump `researched:` and the profiled sha, keep structure and prior content unless it is wrong.
- Corpus offline → proceed as NEW and note "dedup skipped" in your report.

Existing *technique* docs about the repo (e.g. `tries-and-radix-dispatch.md` for forst, the five `mx-*` docs for mech-crate) are not duplicates — **link them from Notable Techniques instead of re-deriving their content**.

## Phase 2 — Read (read-only, breadth first)

Priority order: README / CLAUDE.md / AGENTS.md → `docs/` + `docs/superpowers/specs/` → **surface files** (CLI arg parser, MCP tool registry, HTTP route table, SKILL.md files, Makefile, compose files, CI workflows, manifest/package files) → entry points. Capabilities come from surfaces, not from reading every module — for large repos prefer breadth over completeness and say what you sampled.

The target repo is **READ-ONLY**: no installs, no builds, no file writes, nothing with side effects. `git -C <repo> status --porcelain` must be identical before and after.

## Phase 3 — Author

Copy `skills/repo-profile/TEMPLATE.md`, replace every `{{placeholder}}` and `tpl:` comment. Non-negotiables the checker enforces:

- The eleven `##` sections, exact names, exact order (write "none found" under a heading rather than dropping it).
- Every capability bullet ends with a repo-relative path. README claims you could not verify in code are labeled "README claims, not verified". A doc that links a URL or repo you could not confirm exists: record both facts, assert neither.
- Your own conclusions live **only** under `### Synthesis (inferred)`.
- Config and env vars by **name and purpose only — never values**. Nothing secret-shaped, ever.
- `category: repos` is a sanctioned category (INDEX.md and the MCP tool descriptions list it) — not a typo to fix.
- 120–500 lines (600 only where the task says so). `publish: false` always.
- Multiple copies/forks of the thing you're profiling (installed skills, vendored trees, overlays)? Profile the repo the task names; document the divergence in Relationships with what you diffed.

## Phase 4 — Verify

```bash
python3 skills/repo-profile/scripts/check_profile.py docs/development/repos/<slug>.md --target <repo-path>
```

Must print `PASS`. It checks frontmatter keys, section set/order, size, secret patterns, leftover template markers, the profiled sha in Identity, target-repo cleanliness, and runs `mx rag ingest --dry-run` (0 warnings). Fix the profile, not the checker.

## Phase 5 — Ship

```bash
git checkout -b corpus/repo-<slug>
git add docs/development/repos/<slug>.md
git commit --no-verify -m "docs(corpus): repo profile — <name> (mech-crate-965.<n>)"
git show --stat HEAD   # MUST list exactly one file
```

`--no-verify` is required: the beads pre-commit hook stages `.beads/issues.jsonl` into your commit otherwise (observed, not hypothetical). After committing, `git show --stat HEAD` listing anything besides your profile means redo the commit. Push the branch only if your task says to.

Never edit `INDEX.md`, `RESEARCH_LOG.md`, or `RESEARCH_BACKLOG.md` (the wave gate and wrap-up tasks own them — list backlog-worthy topics in your report instead). Never open a PR (the wave gate batches profiles into one PR).

## If blocked

Never stall on an unknowable: write `undetermined — <what you tried>` in the profile and continue. Reserve questions for cases where a wrong guess poisons the whole profile (e.g. you cannot tell which of two repos the task means).

## Report back

Checker output (the PASS block), NEW vs IMPROVE verdict, line count, 3–5 headline findings (drift, contradictions, canonical-copy decisions), backlog-candidate topics, and anything undetermined.
