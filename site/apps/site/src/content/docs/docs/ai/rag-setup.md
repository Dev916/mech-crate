---
title: Techniques corpus & RAG
description: Set up the techniques corpus — rag.toml, a Neon project or a local pgvector container, OpenAI or any OpenAI-compatible embeddings endpoint — and know exactly where your text goes.
sidebar:
  order: 3
---

The techniques corpus is the retrieval half of the [AI layer](/docs/ai/): the
markdown under `docs/development/` chunked, embedded and stored in Postgres with
pgvector, queried by the `rag_*` [MCP tools](/docs/ai/mcp-server/). Everything
in it is also published here under [Techniques Corpus](/docs/corpus/) — same
source files, two renderings.

## What you need

1. **A Postgres with pgvector.** A [Neon](https://neon.tech) project or a local
   container — mx does not care which.
2. **An embeddings endpoint.** OpenAI by default; any OpenAI-compatible
   `/embeddings` endpoint works.

A local container is one command:

```bash
docker run -d --name mx-rag -p 5432:5432 \
  -e POSTGRES_DB=mx_rag -e POSTGRES_HOST_AUTH_METHOD=trust \
  pgvector/pgvector:pg17
```

## Configuration

`~/.mech-crate/config/rag.toml`. Every key has a default, and every key has an
environment-variable override that wins over the file.

```toml
# Primary store. Unset → the fallback is used.
database_url = "postgres://…"          # env: MX_RAG_DATABASE_URL

# Used when database_url is unset or unreachable.
# Default: postgres://postgres@localhost:5432/mx_rag
fallback_database_url = "postgres://postgres@localhost:5432/mx_rag"
                                        # env: MX_RAG_FALLBACK_DATABASE_URL

# Any OpenAI-compatible /embeddings endpoint.
# Default: https://api.openai.com/v1
embedding_base_url = "https://api.openai.com/v1"
                                        # env: MX_RAG_EMBEDDING_BASE_URL

# Default: text-embedding-3-small
embedding_model = "text-embedding-3-small"
                                        # env: MX_RAG_EMBEDDING_MODEL
```

The API key comes from `MX_RAG_EMBEDDING_API_KEY`, falling back to
`OPENAI_API_KEY`, or an `embedding_api_key` entry in the file.

## Where your text actually goes

This is worth being precise about, because "local-first" is easy to overclaim.

**The store is yours.** Documents, chunks and embedding vectors live in the
Postgres you pointed `database_url` at — your Neon project or your container.
There is no MechCrate service in the path, no shared index, and no vendor
holding your corpus. Retrieval at query time is a SQL query against your own
database.

**Embeddings are computed wherever you point them, and the default is OpenAI.**
Out of the box, `mx rag ingest` sends chunk text to `api.openai.com` to be
embedded, and each `rag_*` query sends the query string the same way. That is a
real egress, and it is on by default.

**The escape hatch is one line.** Point `embedding_base_url` at a local
OpenAI-compatible server — Ollama, LM Studio — and set `embedding_model` to
whatever it serves; ingestion and queries then stay on your machine end to end:

```toml
embedding_base_url = "http://localhost:11434/v1"
embedding_model = "nomic-embed-text"
```

Switching providers changes the vector space, so re-embed what is already
stored:

```bash
mx rag ingest --reembed
```

## The commands

```bash
mx rag ingest     # chunk + embed docs/development into the corpus
mx rag status     # backend, doc/chunk counts, embedding model, last ingest
mx rag gaps       # mine weak-scoring queries for research topics
```

`mx rag ingest` takes `--path <dir>` (defaults to `<mech-crate root>/docs/development`),
`--clear` (drop existing docs and chunks first), `--force` (re-ingest unchanged
docs), `--reembed`, and `--dry-run` — the last of which parses and chunks with no
database and no embeddings at all, which is what the research pipeline uses as a
pre-flight check.

`mx rag gaps` takes `--days <n>` (default 30) and `--min-count <n>` (default 2).
It reads the query log for searches that scored badly and clusters them into
themes — the corpus telling you what it does not know, which feeds the
[research pipeline](/docs/ai/research-pipeline/).

`mx rag status` is the same picture `rag_health` gives an agent:

```
Techniques Corpus Status
  • Backend: neon
  • Docs: 66
  • Chunks: 2148
  • Embedding model: text-embedding-3-small
  • Last ingest: 2026-08-18T14:16:55Z
```

## Honest limits

- **The lexical arm is weak.** Retrieval is meant to be hybrid — vector plus a
  pg_trgm lexical arm — and the lexical half currently contributes far less
  separation than it should. It is measured, not guessed: with the vector arm
  held equal, the lexical arm separates a relevant from an irrelevant chunk by
  **2.18× / 0.0062 of final score**, against a target of ≥5× / ≥0.05. Tracked as
  [`mech-crate-4jw`](/docs/project/known-broken/) with a red test.
- **The corpus is only what got merged.** 66 documents. Ask outside that surface
  and you get weak hits — visible in `mx rag gaps`, which is the intended
  feedback loop rather than an embarrassment to hide.
- **Ingest is not automatic.** Merging a document does not put it in your store;
  `mx rag ingest` does.

## Deeper

- [`appendix-rag`](/docs/corpus/ml/appendix-rag/) — retrieval-augmented
  generation from first principles
- [`rag-retrieval-fusion-and-chunking`](/docs/corpus/ml/rag-retrieval-fusion-and-chunking/)
  — hybrid fusion and chunking for code corpora; the document that measured the
  lexical arm
- [`pgvector-rust-batch-embedding`](/docs/corpus/database/pgvector-rust-batch-embedding/)
  — pgvector in Rust and concurrent batch embedding
