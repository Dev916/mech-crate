---
title: "Doc That Leaks A Credential"
category: process
summary: Fixture only — the values below are fabricated and must fail the build.
---

# Leak

```bash
# FIXTURE DATA — fabricated. Underscores keep it outside the shape
# real-secret scanners look for, while still tripping our own rule.
export OPENAI_API_KEY=sk-FAKE_not_a_real_key_00000000000000000000000000000
```
