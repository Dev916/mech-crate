---
title: "MechCrate MCP Server Usage"
category: process
summary: Slug sanitization plus DSN placeholders that must stay publishable.
---

# Usage

```bash
export MX_RAG_FALLBACK_DATABASE_URL=postgres://postgres@localhost:5432/mx_rag
export MX_RAG_DATABASE_URL=postgres://...neon.tech/mx_rag
```

Risk-free, task-lifecycle and ./models/risk-model.json must not trip the lint.
