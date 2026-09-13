---
title: Agent execution rules
description: Consult the corpus before deciding, evidence before assertion. The rules agents work under in this repository, and where they are published.
sidebar:
  order: 5
---

Giving an agent tools is the easy half. The rules for using them are the half
that decides whether the output is worth reading.

Two of them carry most of the weight here:

**Consult before deciding.** Before choosing a pattern, picking a technology,
designing an API or implementing a non-obvious algorithm, query the corpus with
`rag_context` and a description of the task, or with the narrower `rag_*`
[tools](/docs/ai/mcp-server/) when you know what you are after. The corpus
exists so that architectural decisions are made against written-down experience
instead of a plausible-sounding guess. Weak results are information too: they
mean the corpus does not cover this, which is what `mx rag gaps` collects and the
[research pipeline](/docs/ai/research-pipeline/) acts on.

**Evidence before assertion.** No claiming a fix without running it, no
"should work", no marking a task complete without the command output that proves
it. This is the same standard the [testing](/docs/framework/testing/) page holds
CI to, and the reason defects live in public in the
[known-broken lane](/docs/project/known-broken/) rather than getting quietly
written around.

The full rule set (RAG tool routing, functional design foundations, error
handling strategy, the procedure agents follow) is a corpus document, so agents
retrieve it the same way they retrieve everything else:

**→ [Codex Execution Rules](/docs/corpus/process/instructions/)**

It is long, and deliberately so: it doubles as the index into the theory and
pattern material the rest of the corpus holds.
