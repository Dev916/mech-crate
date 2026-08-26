---
title: Research log & backlog
description: The append-only record of every technique-research run, and the queue of topics waiting for one — both published as corpus documents.
sidebar:
  order: 3
---

The techniques corpus grows through a [research
pipeline](/docs/ai/research-pipeline/) that opens pull requests. Two documents
keep it accountable, and both are published as corpus pages rather than
summarised here — they are the record, not a report about the record.

## The log

**→ [Research log](/docs/corpus/process/research-log/)**

Append-only, one row per run, newest first: date, topic, coverage verdict,
number of sources, outcome. Written by the research skill at the end of every
run — *including* the runs that produced nothing.

That last part is what makes it worth reading. A run that finds the corpus
already covers a topic logs a FRESH row and stops without a pull request; a run
that cannot find enough sources logs "insufficient sources" and stops. Both
appear in the log next to the runs that authored documents, so the ratio is
visible instead of implied.

The outcome column is where the corrections live. Research has, on the record,
falsified a cached-prompt TTL figure the corpus had been repeating, corrected a
stale delegation multiplier from 15× to 5×, and established by measurement
that this project's own lexical retrieval arm was inert — which became
[`mech-crate-4jw`](/docs/project/known-broken/) with a red test against it.

## The backlog

**→ [Research backlog](/docs/corpus/process/research-backlog/)**

The queue. Autonomous runs pop the top unchecked entry; anyone — human or
agent — may append one. Entries carry a one-line reason and who added it:

```
- [ ] <topic> — <one-line why> (added YYYY-MM-DD by <who>)
```

Three things fill it. Humans adding topics they want covered. The techniques
skill appending when a `rag_context` lookup comes back weak — the corpus
noticing its own gap mid-task. And the research pipeline itself, which proposes
topics back into the queue when the ladder reaches its tech-radar sweep.

Some entries are deliberately parked as watch items rather than research tasks:
a wire protocol to re-check when its redesign ships, a Postgres major release to
verify pgvector against before upgrading. Those sit unchecked on purpose.

## Why they are corpus documents

Both files live in `docs/development/` with the same frontmatter as everything
else there, which means they are ingested, chunked and retrievable — an agent
can ask what the project has already researched, and get an answer, before
proposing to research it again. Publishing them here is the human-facing half of
the same fact.
