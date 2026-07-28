---
title: "RAG Retrieval Quality: Hybrid Fusion and Chunking for Code Corpora"
category: ml
languages: [sql, rust]
complexity: advanced
use_cases:
  - choosing between RRF and weighted score blending for hybrid search
  - diagnosing a lexical arm that contributes nothing (score-scale mismatch)
  - chunking source code and technical markdown for embeddings
  - deciding whether contextual retrieval, late chunking, or reranking is worth it
summary: Evidence-based guidance on hybrid retrieval fusion (RRF vs convex combination, with the counter-narrative), pg_trgm's structural failure as a lexical arm over long chunks (empirically verified), AST-aware code chunking, token-based sizing, contextual retrieval economics, and how to evaluate any of it.
provenance: researched
researched: 2026-07-26
sources:
  - https://cormack.uwaterloo.ca/cormacksigir09-rrf.pdf
  - https://arxiv.org/abs/2210.11934
  - https://opensearch.org/blog/introducing-reciprocal-rank-fusion-hybrid-search/
  - https://weaviate.io/blog/hybrid-search-fusion-algorithms
  - https://learn.microsoft.com/en-us/azure/search/hybrid-search-ranking
  - https://www.elastic.co/search-labs/blog/linear-retriever-hybrid-search
  - https://jkatz05.com/post/postgres/hybrid-search-postgres-pgvector/
  - https://www.postgresql.org/docs/current/pgtrgm.html
  - https://www.paradedb.com/blog/hybrid-search-in-postgresql-the-missing-manual
  - https://www.anthropic.com/engineering/contextual-retrieval
  - https://arxiv.org/abs/2506.15655
  - https://www.trychroma.com/research/evaluating-chunking
  - https://arxiv.org/abs/2505.21700
  - https://arxiv.org/pdf/2409.04701
  - https://arxiv.org/abs/2504.19754
  - https://docs.llamaindex.ai/en/latest/examples/retrievers/auto_merging_retriever/
---

# RAG Retrieval Quality: Hybrid Fusion and Chunking

What actually improves retrieval in a hybrid (lexical + vector) RAG system, with the evidence — including one failure mode we verified empirically on our own corpus. Inline `[n]`-style cites key to `sources`.

## 1. Fusion: how to combine lexical and vector scores

**Reciprocal Rank Fusion (RRF)** [1]: `score(d) = Σ 1/(k + rank_r(d))` over each retriever's ranked list; `k` defaults to 60 (Elastic) or 50 (Katz's pgvector pattern) — higher k flattens toward equal voting. Rank-based fusion is the safe default because the score families are incomparable: BM25 is unbounded, cosine lives in [0.33, 1.0] — a fixed linear blend of the two has undefined tail behavior [3][5][6].

**The counter-narrative most blogs miss**: tuned convex combination (normalized weighted blend) *beats* RRF when you can tune the weight. Peer-reviewed (Bruch et al., TOIS 2023 [2]): CC outperforms RRF in- and out-of-domain, RRF is parameter-sensitive, the normalization function barely matters (min-max vs z-score are rank-equivalent under retuning) — **the weight α is what matters, and it's sample-efficient to tune** (50–100 labeled queries plausibly suffice). Two vendors corroborate against their own defaults: OpenSearch measured RRF at −3.86% avg nDCG@10 vs score normalization (worst −8.13% on FiQA) [3]; Weaviate measured ~6% better recall for normalized fusion over RRF and switched their default to `relativeScoreFusion` [4].

**Decision rule**: no labeled queries → RRF (k=60), immune to scale bugs. Have (or can build) a 50–100 query golden set → tune a min-max-normalized convex combination and ship whichever wins on YOUR corpus. Weighted RRF is the middle ground (Elastic/Azure support per-arm weights before fusion).

**pgvector pattern** [7]: no native hybrid — over-fetch ~40 per arm, `UNION ALL`, `GROUP BY id`, `SUM(rrf_score(rank))` with an `IMMUTABLE PARALLEL SAFE` SQL function; ~8.5ms on 50k rows with HNSW + GIN.

**Two-stage retrieval**: cross-encoder reranking is the largest single-step gain measured — Anthropic's numbers: 2.9% → 1.9% top-20 retrieval failure [10]. Self-host `bge-reranker-v2-m3` (278M, Apache-2.0) for small corpora; skip ColBERT/late-interaction below serious scale.

## 2. The pg_trgm trap: a lexical arm that does nothing (verified)

`pg_trgm similarity(a, b)` is **Jaccard over trigram sets** — the denominator includes every trigram of the long string [8]. Against a ~1200-char chunk, a 3-word query's score is ceilinged at roughly `|query trigrams| / |chunk trigrams|` ≈ 0.04–0.06. That's a length penalty, not relevance.

Measured on Postgres 16 (reproducible; two realistic heading-prefixed 1.2KB chunks):

| Signal | Relevant chunk | Irrelevant chunk | Separation |
|---|---|---|---|
| `similarity('reciprocal rank fusion', chunk)` | 0.0402 | 0.0210 | **0.019** |
| `ts_rank_cd` | 0.1006 | 0.0000 | clean |
| `strict_word_similarity(query, chunk)` | 1.000 | 0.107 | ~9× |

At weight 0.15, that 0.019 separation contributes ~**0.003** to the final score while a 0.85-weighted vector arm swings 0.17–0.26 — the lexical arm is arithmetically incapable of reordering anything. **A blend like `0.85·cosine + 0.15·similarity()` is vector-only search paying for a trigram scan.** Fix ladder: (a) one-liner — `strict_word_similarity(query, chunk)` (short needle FIRST; extent-matching, no length penalty) [8]; (b) right — `tsvector` + `ts_rank_cd` as the lexical arm (but note: no IDF — can't tell rare terms from common ones [9]); (c) best — real BM25 via pg_search/VectorChord-bm25/pg_textsearch, which matters for technique/code corpora full of rare discriminating identifiers. After ANY fix, the old weight is stale — retune or switch to RRF.

Secondary pg_trgm hazards: GIN trigram indexes can degrade to near-full scans on common trigrams; no stemming/stopwords [8].

## 3. Chunking code and technical markdown

- **Never split inside a fenced code block.** For oversized blocks, use AST-aware split-then-merge (tree-sitter): recursively split oversized nodes, greedily merge small adjacent siblings under a budget — cAST (EMNLP 2025) measured +4.3 Recall@5 (RepoEval) and +2.67 Pass@1 (SWE-bench) over fixed-size line chunking [11]; Sweep AI ran the same shape in production at 2M files/day.
- **Size in tokens, not characters.** Prose ≈ 4 chars/token, code ≈ 3 — a fixed char budget makes code chunks ~33% bigger than prose chunks in the encoder's units. cAST sizes by non-whitespace chars for the same reason [11].
- **Optimal size is dataset- and model-dependent** [13]: 64–128 tokens for factoid retrieval, 512–1024 for analytical; the practitioner convergence band is 256–512 with modest overlap. Chroma's token-level eval [12]: up to 9% recall spread across strategies, and the widely-copied 800/400 default wastes ~5× context tokens for equal recall (IoU 1.4% vs 7-8% for tighter strategies).
- **Overlap is contested**: it inflates recall while degrading token-efficiency — evaluate with token-level IoU, not recall alone, or you'll always add more overlap and always be wrong [12].

## 4. Context enrichment: what's worth it

- **Contextual retrieval (Anthropic, primary source [10])**: prepend a 50–100-token LLM-generated situating sentence per chunk before embedding. Measured top-20 failure: 5.7% → 3.7% (embeddings arm) → 2.9% (+contextual BM25) → 1.9% (+rerank). Cost: **$1.02 per million document tokens** one-time with prompt caching. Heading-path prefixes are the explicitly-sanctioned zero-cost variant ("document title, section headers") — the cheap 80%.
- **Late chunking (Jina [14])**: embed the whole doc with a long-context encoder, mean-pool per chunk. Beats naive chunking, but **requires token-level encoder output — architecturally unavailable behind pooled-embedding APIs like OpenAI's** [14]. Head-to-head: contextual retrieval wins quality, late chunking wins cost [15]. Behind an API: skip it.
- **Small-to-big / auto-merging**: legitimate for context-window shaping; LlamaIndex's own eval found gains "roughly the same" as base retrieval — don't adopt it for a recall claim [16].
- **The 200k-token sanity check**: Anthropic's threshold — below ~200k tokens of corpus, put everything in the prompt and skip retrieval [10]. Measure your corpus before optimizing its retrieval.

## 5. Evaluating any of this

Fusion changes only reorder → primary metric **nDCG@10**. Chunking changes affect whether the right span exists → **recall@5/10**. Size/overlap changes → **token-level IoU** alongside recall (IoU penalizes redundancy; recall rewards it) [12][17]. Build a 50–100 golden-query set (LLM-generated + human-reviewed) before shipping retrieval changes; the same set powers convex-combination α tuning (§1).

Embedding model note: `text-embedding-3-small` = 1536 dims, 8192-token context, Matryoshka-truncatable; general encoders trail code-specialized ones (CoIR benchmark) on code-to-code retrieval — a working lexical arm with IDF covers much of that gap, since code queries are dominated by rare exact identifiers.

## Synthesis (inferred)

Applying the above to the mx techniques corpus (pgvector, `0.85·cosine + 0.15·pg_trgm similarity`, 1200-char heading-aware chunks, text-embedding-3-small) — priority-ordered, each traceable to sections above:

1. **Measure corpus size first** (§4 sanity check): if under ~200k tokens, retrieval is a scale hedge, not a necessity — right-size the effort.
2. **Build the golden-query eval harness** (§5) before touching anything: ~50-100 queries, nDCG@10 + recall@5 + IoU.
3. **Fix the inert lexical arm** (§2): minimum `strict_word_similarity`, better `ts_rank_cd`, best BM25 — then retune or move to RRF; the 0.85/0.15 weights are dead on arrival with a working lexical arm.
4. **Adopt the Katz RRF pattern** (§1) as the scale-bug-proof default; A/B a tuned convex combination once the eval set exists.
5. **Chunker upgrades** (§3): token budgets, fence-atomic code blocks with AST split-then-merge for oversized ones.
6. **Contextual enrichment** (§4): generated situating sentences on top of the existing heading paths — sub-$1 for this corpus size.
7. **Reranking last** (§1): after the arms are healthy; bge-reranker-v2-m3 self-hosted.
