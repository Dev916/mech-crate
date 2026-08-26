---
title: Research pipeline
description: How the techniques corpus grows — a weekly autonomous run that researches a topic, corroborates discovery-grade claims against primary sources, and opens a pull request a human merges.
sidebar:
  order: 4
---

A corpus that never grows goes stale; a corpus that grows itself becomes a
laundering machine for whatever the model believed that week. The pipeline here
is the compromise: research runs on a schedule, but **its only output is a pull
request**, and a human merges it.

This is a process this project runs on its own repository, not a feature mx
ships to you. It is documented because the corpus you retrieve from is the
output of it, and you should know how the text got there.

## The run

A user crontab entry fires
[`scripts/research-weekly.sh`](https://github.com/Dev916/mech-crate/blob/main/scripts/research-weekly.sh)
every Monday morning (`3 9 * * 1`). It invokes headless Claude Code against the
[`technique-research`](https://github.com/Dev916/mech-crate/tree/main/skills/technique-research)
skill in autonomous mode, with an explicit tool allowlist rather than blanket
permissions — the run ingests untrusted web content, so what it can touch is
enumerated. Logs land in `~/.mech-crate/research-cron.log`; pausing it is
commenting out one crontab line.

## Picking a topic

Autonomous mode walks a ladder and stops at the first hit:

1. The top unchecked entry in
   [`RESEARCH_BACKLOG.md`](/docs/corpus/process/research-backlog/) — anyone,
   human or agent, can append to it.
2. The stalest document: oldest `researched:` frontmatter date, with documents
   carrying no date ranking stalest.
3. The top theme from `mx rag gaps --days 30 --min-count 2` — the corpus
   reporting what it answered badly.
4. Failing all of that, a tech-radar sweep that proposes three to five topics
   back into the backlog and takes the first.

## Coverage verdict, then research

Before researching anything the run asks the corpus what it already knows and
commits to a verdict: **NEW** (nothing meaningful — author a document),
**IMPROVE** (a document covers it — enumerate its concrete gaps against current
practice), or **FRESH** (covered and current — log a no-op row and stop, no pull
request). Of the ten runs logged so far one stopped dead at FRESH with no pull
request, and another left one of the two documents it examined untouched. That
is the point: a run is not supposed to always produce a document.

Research then runs through source providers matched to the topic. The rule that
does the load-bearing work:

> Discovery-grade claims — anything sourced from social or aggregator feeds —
> must be corroborated by a primary source before being stated as fact.
> Otherwise they go under a `## Synthesis (inferred)` heading, or they are
> dropped.

Everything the model contributes itself — inferred patterns, connections it
drew — is confined to that `Synthesis (inferred)` heading. Every other claim
traces to a citation, and the citation list ships in the document's frontmatter,
which is what renders as the provenance footer on a corpus page.

Disagreements between sources get reconciled explicitly rather than averaged.
If the sources are too thin to support a document, the run logs "insufficient
sources" and stops without a pull request.

## The gate

1. `mx rag ingest --dry-run` must report zero warnings — the document has to
   parse and chunk cleanly before it can be proposed.
2. Code examples compile or type-check where a toolchain exists; otherwise they
   are marked illustrative.
3. The run commits to a `research/<slug>` branch, pushes, and opens a pull
   request carrying the coverage verdict, what changed and why, the full source
   list, and an inventory of the inferred sections.
4. It appends a row to [`RESEARCH_LOG.md`](/docs/corpus/process/research-log/)
   and checks off the backlog entry in the same pull request.
5. **It does not merge.** A human does.

Post-merge, the next `mx rag ingest` picks up the delta. Nothing reaches a
corpus store — yours or this project's — that was not reviewed by a person
first.

## Reading the trail

The log is append-only and public, one row per run: date, topic, verdict, source
count, outcome. It records the no-ops as readily as the wins, and it records
what the research *falsified* — one run corrected a cached-prompt TTL claim the
corpus had been repeating, another found that this project's own lexical
retrieval arm was inert, which is now
[`mech-crate-4jw`](/docs/project/known-broken/) with a red test against it.

**→ [Research log](/docs/corpus/process/research-log/)** ·
**[Research backlog](/docs/corpus/process/research-backlog/)** ·
**[what these are, and how to read them](/docs/project/research/)**
