---
title: Project
description: MechCrate's open books — the known-broken lane, the research log and backlog, and the license.
sidebar:
  order: 1
---

Most projects publish the parts that flatter them. This section is the rest: the
defects that are open right now, the record of what the research pipeline
learned and what it got wrong, and the terms the whole thing ships under.

| Page | What it is |
|---|---|
| [Known-broken lane](/docs/project/known-broken/) | Every open, testable defect — rendered from `tests/KNOWN_BROKEN.md` at build time |
| [Research log &amp; backlog](/docs/project/research/) | What the research pipeline has done, and what is queued |
| [License](/docs/project/license/) | Dual MIT / Apache-2.0 |

The reasoning is simple enough. A project that indexes its own defects can be
checked; a project that does not is asking to be taken on faith. The lane page
is built from the repository's own index rather than transcribed, so it cannot
drift into flattery — if a row is on this page, the defect is open today.

The same posture runs through the docs: the [upgrade](/docs/framework/upgrade/)
page says `mx upgrade` is broken, the [RAG setup](/docs/ai/rag-setup/) page says
the lexical retrieval arm underperforms by a measured factor, and
[testing](/docs/framework/testing/) links the CI runs where each gate was proven
to fail rather than asserting that it would.
