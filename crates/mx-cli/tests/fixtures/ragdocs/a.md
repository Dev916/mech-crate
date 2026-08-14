---
title: Alpha Fixture Technique
category: process
languages: [rust]
complexity: intermediate
use_cases:
  - exercising the rag ingest dry-run path
  - keeping the CLI surface test hermetic
summary: A tiny, fully-frontmattered fixture doc for the mx-cli dry-run test.
---

# Alpha Fixture Technique

Fixture body for the `mx rag ingest --dry-run` surface test. Kept deliberately
small so the chunk count stays stable and the scan needs no database.

## Why this exists

The dry-run path parses and chunks only. A doc with valid frontmatter must
produce zero warnings, which is what makes the `0 warnings` assertion mean
something.
