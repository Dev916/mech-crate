---
title: Testing
description: The gates that block a release, the evidence that they actually block, and the lane where open defects live in public.
sidebar:
  order: 7
---

Three jobs block every release: **lint**, **test**, **coverage**. A release tag
cannot ship unless its commit passes all three, because the publish jobs depend
on them.

## Proven, not assumed

A CI gate that has never failed is a gate you are *hoping* works. Each of the
three was demonstrated failing on a deliberately-broken branch before the
baseline landed:

| Deliberate break | Job that failed | Evidence |
|---|---|---|
| a red unit test | `test` | [run 31819711713](https://github.com/Dev916/mech-crate/actions/runs/31819711713) |
| a `clippy::useless_format` warning | `lint` | [run 31819749340](https://github.com/Dev916/mech-crate/actions/runs/31819749340) |
| `.coverage-floor` raised 49.5 → 54.5 | `coverage` | [run 31819764638](https://github.com/Dev916/mech-crate/actions/runs/31819764638) |

Those links go to real, red CI runs. That is the standard the rest of this page
is written to.

## The targets

```bash
make test               # the gate: cargo-nextest across the workspace + doc-tests
make lint               # clippy with -D warnings
make coverage           # line coverage against a ratcheting floor
make check              # fmt-check + lint + test
```

| Target | What it is | Blocking |
|---|---|---|
| `make test` | `cargo nextest run --workspace --profile ci` plus `cargo test --doc` | yes, on every PR and push to `main` |
| `make lint` | clippy, warnings denied | yes |
| `make coverage` | ratchet script against `.coverage-floor` (currently **49.5%**). `BUMP=1` raises the floor; a drop fails CI | yes |
| `make test-known-broken` | the TDD lane below: a report, never a gate | no |
| `make test-e2e` | scaffold → `make dev` → live router URL → teardown, with real Docker | dispatched workflow |
| `make test-mutants` | `cargo-mutants` over `mx-lib` | weekly cron; missed mutants are backlog, not failure |
| `make test-unit` | fast unit-only loop, no database | local |
| `make test-int` | integration against a local pgvector container | local |

Database-backed tests skip when `MX_RAG_TEST_DATABASE_URL` is unset, so a laptop
without Docker still runs green. CI always supplies the container. Skipping is an
env-var early return, never `#[ignore]`. That attribute is reserved for one
thing only.

## The known-broken lane

Every tracked, testable defect in this repository has a test that asserts its
**fixed** behaviour *today*, annotated `#[ignore = "bd:mech-crate-<id> …"]`. The
gate never runs them; `make test-known-broken` runs only them and scoreboards the
result.

**Expected state: all red.** A lane test that turns green is the signal that a
fix landed without bookkeeping. Surfaced, not silently absorbed.

The fix workflow is the definition of done for each issue: claim it, make the
lane test pass, delete its `#[ignore]` so the test joins the gate, remove its row
from the index, close the issue.

Two house rules keep the lane honest:

- Every lane test's *arrange* half must succeed today. A lane test that dies on a
  missing fixture tells you nothing about the defect, so setup assertions carry a
  `setup:` prefix and the two failure classes stay distinguishable.
- Lane tests live beside the suite that owns their subject, not in one central
  file. The test is the first thing whoever fixes it should read.

The two numbers partition the workspace: the lane reports 14 tests, 14 red; the
gate suite in the same tree reports 189 passed, 14 skipped. If they stop summing,
either a lane test lost its `#[ignore]` or a gate test grew one.

Several pages in these docs point at this lane, because pointing at it is the
alternative to quietly writing around a defect:
[`mech-crate-z5i`](/docs/framework/upgrade/) for `mx upgrade`,
`mech-crate-vxq`, `mech-crate-wd9` and `mech-crate-066` for the
[infra credential path](/docs/framework/infra-credentials/).

**→ [The lane index on GitHub](https://github.com/Dev916/mech-crate/blob/main/tests/KNOWN_BROKEN.md)**
· **→ [Project notes](/docs/project/)** for the research log and backlog

## This site

The documentation site is an mx project too, and it has its own gates: a vitest
suite over the corpus loader (frontmatter mapping, `publish: false` filtering,
slug sanitisation, link rewriting, the secret lint) and an `astro build` that
fails the PR when a document breaks. Docs are the product, so a broken document
breaking the build is the correct outcome.

```bash
cd site/apps/site
npm test          # vitest
npx astro build   # the build gate
```

## Agent execution rules

Agents working in this repository are held to the same standard: evidence before
assertion, no claiming a fix without running it. Those rules are published in the
corpus: [instructions](/docs/corpus/process/instructions/).
