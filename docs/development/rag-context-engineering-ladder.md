---
title: "From Naive RAG to Context Lakes: A Maturity Ladder for LLM Context (Hybrid Retrieval, GraphRAG, Code Graphs, Agent Memory)"
category: ml
languages: [python, sql]
complexity: advanced
use_cases:
  - placing an existing RAG or agent-context system on a maturity ladder and picking the next cheapest improvement
  - deciding whether GraphRAG, a code graph, late interaction or agent memory is worth its cost for a given corpus and query mix
  - designing a governed retrieval surface ("context lake") over documents, data, code and conversation records for agents
  - assembling a token-budgeted, deduplicated, cited context block and gating retrieval changes with evals
summary: "A rung-by-rung ladder for LLM context as of 2026-10: no retrieval, naive RAG, hybrid, query-side and index-side enrichment, agentic retrieval, GraphRAG, code graphs, agent memory and context lakes, assembly and evaluation, each with sourced gains, costs and a climb signal."
provenance: researched
researched: 2026-10-05
sources:
  - https://simonwillison.net/2025/Jun/27/context-engineering/
  - https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents
  - https://claude.dev/blog/the-new-rules-of-context-engineering-for-claude-5-generation-models/
  - https://www.anthropic.com/engineering/contextual-retrieval
  - https://arxiv.org/abs/2407.16833
  - https://arxiv.org/abs/2502.05167
  - https://www.trychroma.com/research/context-rot
  - https://arxiv.org/abs/2401.05856
  - https://arxiv.org/abs/2104.08663
  - https://arxiv.org/abs/2212.10496
  - https://arxiv.org/abs/2303.07678
  - https://arxiv.org/abs/2305.03653
  - https://arxiv.org/abs/2505.12694
  - https://arxiv.org/abs/2305.14283
  - https://arxiv.org/abs/2402.03367
  - https://arxiv.org/abs/2310.06117
  - https://arxiv.org/abs/2212.10509
  - https://arxiv.org/abs/2403.14403
  - https://reference.langchain.com/python/langchain-classic/retrievers/self_query/base/SelfQueryRetriever
  - https://arxiv.org/abs/2112.01488
  - https://arxiv.org/abs/2407.01449
  - https://arxiv.org/abs/2312.06648
  - https://arxiv.org/abs/2401.18059
  - https://reference.langchain.com/python/langchain-classic/retrievers/parent_document_retriever/ParentDocumentRetriever
  - https://github.com/pgvector/pgvector
  - https://qdrant.tech/documentation/concepts/vectors/
  - https://arxiv.org/abs/2310.11511
  - https://arxiv.org/abs/2401.15884
  - https://arxiv.org/abs/2503.09516
  - https://news.ycombinator.com/item?id=43164253
  - https://cursor.com/blog/semsearch
  - https://aider.chat/2023/10/22/repomap.html
  - https://arxiv.org/abs/2404.16130
  - https://github.com/microsoft/graphrag
  - https://www.microsoft.com/en-us/research/blog/introducing-drift-search-combining-global-and-local-search-methods-to-improve-quality-and-efficiency/
  - https://www.microsoft.com/en-us/research/blog/lazygraphrag-setting-a-new-standard-for-quality-and-cost/
  - https://arxiv.org/abs/2410.05779
  - https://arxiv.org/abs/2405.14831
  - https://arxiv.org/abs/2502.14802
  - https://arxiv.org/abs/2502.11371
  - https://arxiv.org/abs/2506.05690
  - https://arxiv.org/abs/2501.13956
  - https://github.com/scip-code/scip
  - https://sourcegraph.com/blog/announcing-scip
  - https://github.blog/open-source/introducing-stack-graphs/
  - https://github.com/github/stack-graphs
  - https://docs.joern.io/code-property-graph/
  - https://code.claude.com/docs/en/plugins/code-intelligence
  - https://github.com/oraios/serena
  - https://arxiv.org/abs/2410.14684
  - https://arxiv.org/abs/2408.03910
  - https://arxiv.org/abs/2503.09089
  - https://cursor.com/blog/secure-codebase-indexing
  - https://arxiv.org/abs/2310.08560
  - https://arxiv.org/abs/2504.19413
  - https://arxiv.org/abs/2402.17753
  - https://arxiv.org/abs/2410.10813
  - https://blog.getzep.com/lies-damn-lies-statistics-is-mem0-really-sota-in-agent-memory/
  - https://www.letta.com/blog/benchmarking-ai-agent-memory
  - https://platform.claude.com/docs/en/agents-and-tools/tool-use/memory-tool
  - https://platform.claude.com/docs/en/build-with-claude/context-editing
  - https://tacnode.io/context-lake
  - https://contextlake.org/canonical
  - https://arxiv.org/abs/2601.17019
  - https://www.getzep.com/ai-agents/what-is-a-context-lake/
  - https://www.port.io/glossary/context-lake
  - https://altertable.ai/blog/2026-07-15-lakehouse-as-context-store
  - https://github.com/timescale/pgai
  - https://www.postgresql.org/docs/current/ddl-rowsecurity.html
  - https://aws.amazon.com/s3/features/vectors/
  - https://modelcontextprotocol.io/specification/latest
  - https://arxiv.org/abs/2307.03172
  - https://www.cs.cmu.edu/~jgc/publication/The_Use_MMR_Diversity_Based_LTMIR_1998.pdf
  - https://arxiv.org/abs/2310.05736
  - https://platform.claude.com/docs/en/build-with-claude/citations
  - https://manus.im/blog/Context-Engineering-for-AI-Agents-Lessons-from-Building-Manus
  - https://arxiv.org/abs/2309.15217
  - https://docs.ragas.io/en/stable/concepts/metrics/available_metrics/context_precision/
  - https://docs.ragas.io/en/stable/concepts/metrics/available_metrics/context_recall/
  - https://docs.ragas.io/en/stable/concepts/metrics/available_metrics/faithfulness/
  - https://www.trulens.org/getting_started/core_concepts/rag_triad/
  - https://arxiv.org/abs/2406.04744
  - https://arxiv.org/abs/2306.05685
---

# From Naive RAG to Context Lakes: A Maturity Ladder for LLM Context (Hybrid Retrieval, GraphRAG, Code Graphs, Agent Memory)

State of practice as of 2026-10. "Context engineering" is the name the field settled on in mid-2025 for everything that decides which tokens reach the model [1][2], and retrieval is one tool inside it, not the whole job [2][3]. This doc is a ladder: twelve rungs from putting the whole corpus in a cached prompt [4] to a governed context lake [62][65], each with what it adds, the best public evidence for what it buys (figures carry the benchmark they were measured on; vendor figures about their own products are labelled vendor-reported), what it costs, and the signal that says climb. The rule throughout is to stand on the lowest rung your evals say is enough [8][77].

The doc deliberately does not repeat the deep single-rung docs already in this corpus, and points at them instead: `appendix-rag.md` (a working naive-RAG quick start on SQLite vectors), `rag-retrieval-fusion-and-chunking.md` (hybrid fusion, the pg_trgm trap, AST-aware chunking, contextual retrieval, reranking, retrieval metrics), `pgvector-rust-batch-embedding.md` (pgvector indexing and embedding pipelines), `multi-agent-systems-in-practice.md` section 4 (grep versus code-graph tools for agents), `llm-token-cache-efficiency.md` section 5 (context rot, reduction levers, caching) and `mcp-server-tool-design.md` (exposing retrieval as tools). Everything outside `## Synthesis (inferred)` is cited; inline `[n]` keys to `sources`.

## Overview: the ladder on one screen

| Rung | What it adds | Best public evidence | Main cost | Climb when |
|---|---|---|---|---|
| 0. No retrieval | Whole corpus in a cached prompt | Under 200,000 tokens, "no need for RAG" (Anthropic, 2024) [4]; long context beats RAG when resourced [5] | Tokens per call; context rot as length grows [6][7] | Corpus outgrows the window or quality drops with length |
| 1. Naive RAG | Chunk, embed, top-k | Seven documented failure points [8] | An index to keep fresh | Eval shows misses on exact terms or rank order |
| 2. Hybrid + rerank + chunking | Lexical arm, fusion, cross-encoder, better chunks | Top-20 retrieval failure 5.7% to 1.9% with contextual embeddings, contextual BM25 and reranking [4] | A second index, a reranker call | Queries are vague, multi-part or filter-shaped |
| 3. Query-side | Rewrite, HyDE, multi-query, step-back, routing, self-query | Query2doc +3% to 15% BM25 [11]; step-back +27% TimeQA [16] | One or more LLM calls before retrieval | Answers need cross-chunk structure |
| 4. Index-side | Late interaction, page images, propositions, RAPTOR, parent docs | RAPTOR +20% absolute on QuALITY with GPT-4 [23] | Storage (multi-vector) or LLM indexing passes | Single-shot retrieval cannot reach the answer |
| 5. Corrective and agentic | Retrieval as a tool in a loop; graders | IRCoT up to +21 retrieval points [17]; Search-R1 +41% (7B) [29] | Latency, tool calls, token spend | Questions are corpus-global or multi-hop over entities |
| 6. Graph RAG | Entity graph, communities, PageRank | GraphRAG wins global sensemaking [33]; basic RAG matches it on simple facts [41] | Expensive LLM indexing [34]; heavy query prompts [41] | Corpus is a codebase |
| 7. Code graphs | Symbols, references, call graphs, LSP | RepoGraph 32.8% average relative gain on SWE-bench Lite [50]; Cursor +12.5% QA (vendor-reported) [31] | Indexer per language; freshness | Agents need state across sessions or sources |
| 8. Memory and context lakes | Persistent memory; one governed surface over all sources | Benchmarks disputed [58][59]; "context lake" is a vendor label [62][65][66] | Governance, ingestion, permissions | Always, once retrieval exists |
| 9. Context assembly | Budget, dedup, MMR, ordering, citations, compaction | Middle-of-context degradation [72]; up to 20x compression [74] | Engineering time | Always |
| 10. Evaluation | Component and end-to-end metrics in CI | RAGAS, RAG triad, CRAG benchmark [77][81][82] | Golden sets, judge calls | Before every climb |
| 11. Decision table | Maps corpus and query type to the lowest rung | Sections 1 to 10 | Not reported | Not reported |

## 0. Framing: context engineering and the no-retrieval baseline

**Where the term came from.** On 2025-06-27 Simon Willison noted that "context engineering" was gaining traction, quoting Shopify CEO Tobi Lutke ("I really like the term 'context engineering' over prompt engineering") and Andrej Karpathy ("+1 for 'context engineering' over 'prompt engineering'") [1]. Karpathy's gloss lists the ingredients: "task descriptions and explanations, few shot examples, RAG, related (possibly multimodal) data, tools, state and history, compacting" [1].

**The primary definition.** Anthropic's engineering post (2025-09-29) defines it as "the set of strategies for curating and maintaining the optimal set of tokens (information) during LLM inference, including all the other information that may land there outside of the prompts" [2]. Its operating principle is to find "the smallest set of high-signal tokens that maximize the likelihood of your desired outcome" [2]. Its July 2026 follow-up for Claude 5 models reports that Anthropic "removed over 80% of Claude Code's system prompt" with "no measurable loss on our coding evaluations", and replaces "put it all upfront" with progressive disclosure: skills and deferred tools loaded "at the right times" [3]. Manus's equally primary counterpart (2025-07-18) argues that "the KV-cache hit rate is the single most important metric for a production-stage AI agent" and reports an average input-to-output token ratio "around 100:1" [76].

**Rung 0: do not retrieve if you do not have to.** Anthropic's 2024-09 guidance: "If your knowledge base is smaller than 200,000 tokens (about 500 pages of material), you can just include the entire knowledge base in the prompt" [4]. A controlled study found that "when resourced sufficiently, LC consistently outperforms RAG in terms of average performance", while "RAG's significantly lower cost remains a distinct advantage"; its Self-Route method routes each query to RAG or long context by model self-reflection and "significantly reduces the computation cost while maintaining a comparable performance to LC" [5].

**But long context is not free attention.** NoLiMa removed literal overlap between question and needle: "At 32K, for instance, 11 models drop below 50% of their strong short-length baselines", and GPT-4o fell from 99.3% to 69.7% [6]. Chroma's 18-model study found performance "degrades as input length increases, often in surprising and non-uniform ways" [7]. Caching mechanics and the full context-rot discussion live in `llm-token-cache-efficiency.md` sections 1 and 5.

| Signal | Stay on rung 0 | Leave rung 0 |
|---|---|---|
| Corpus size | Under about 200,000 tokens [4] | Over it, or growing |
| Query mix | Few distinct questions over the same corpus [4] | Many users, many unrelated questions [5] |
| Accuracy versus length | Flat on your eval as the prompt grows | Drops as the prompt grows [6][7] |
| Cost | Cache hit rate high [76] | Re-sending the corpus dominates spend [5] |

## 1. Naive RAG: chunk, embed, top-k

**What it is.** The seven-failure-points paper describes the pattern: "finding documents that semantically match a query and then passing the documents to a large language model (LLM)" [8]. The how-to for this corpus is `appendix-rag.md`.

**Where it breaks.** Barnett et al. (2024-01) derived seven failure points from three deployed systems in research, education and biomedicine [8]:

| FP | Name [8] | What you see | Usual fix (rung) |
|---|---|---|---|
| 1 | Missing Content | Answer not in the corpus; model guesses | Abstention prompt, coverage audit (9, 10) |
| 2 | Missed the Top Ranked Documents | Right chunk exists but ranks below k | Hybrid and reranking (2) |
| 3 | Not in Context, consolidation strategy limitations | Retrieved but dropped during assembly | Budget and ordering (9) |
| 4 | Not Extracted | In context, model fails to use it | Less noise, ordering (9) |
| 5 | Wrong Format | Ignores the requested table or list | Prompt and output schema |
| 6 | Incorrect Specificity | Too general or too detailed | Query rewriting (3) |
| 7 | Incomplete | Partial answer across chunks | Decomposition, agentic loops (3, 5) |

Their two takeaways are the reason this ladder ends in evaluation: "validation of a RAG system is only feasible during operation", and "the robustness of a RAG system evolves rather than designed in at the start" [8].

**Climb signal.** Failure point 2 on queries full of exact identifiers or rare terms is the classic sign that dense retrieval alone is not enough; BEIR found "BM25 is a robust baseline" across 18 zero-shot datasets, with dense models that "often underperform" out of domain [9].

## 2. Hybrid retrieval, reranking and better chunking

This rung is owned by `rag-retrieval-fusion-and-chunking.md`, which covers RRF versus tuned convex combination, why a pg_trgm lexical arm can contribute nothing over long chunks, AST-aware code chunking, token-based sizing, contextual retrieval economics, late chunking and cross-encoder reranking, and which metric to use for each change. Read it before this section's neighbours.

The headline numbers it relies on, re-verified here: Anthropic's contextual retrieval "can reduce the number of failed retrievals by 49% and, when combined with reranking, by 67%" on their top-20-chunk evaluation [4]. BEIR's zero-shot result that "re-ranking and late-interaction-based models on average achieve the best zero-shot performances, however, at high computational costs" is the reason reranking sits on this rung and late interaction on rung 4 [9].

| Move | Buys | Costs | Source |
|---|---|---|---|
| Working lexical arm (BM25 class) | Rare-term and identifier recall | Second index | [9] |
| Fusion (RRF, then tuned blend) | Robust merge of arms | A golden set to tune | see sibling doc |
| Contextual chunk prefixes | 49% fewer top-20 failures | One LLM pass at ingest | [4] |
| Cross-encoder rerank | 67% fewer failures with the above | One model call per query | [4] |

This pass found no newer primary source that changes those recommendations since the sibling doc's 2026-07-26 research date, so it remains the authority for rung 2 [4][9].

**Climb signal.** Rung 2 fixes ranking of chunks that match the query; it cannot fix a query that does not match the chunk, the "gap between the input text and the needed knowledge in retrieval" [14]. When misses survive hybrid plus reranking and the failing queries are vague, multi-part or carry filters (*2024 incidents in the billing service*), climb to query-side techniques [14][19].

## 3. Query-side techniques

Every technique here spends an LLM call before retrieval to make the query look more like the answer, so each one adds a model round trip to latency [14][18].

| Technique | Mechanism | Evidence | Cost and caveat |
|---|---|---|---|
| Rewrite-Retrieve-Read | LLM rewrites the query; a small trainable rewriter can be tuned with RL from reader feedback [14] | "consistent performance improvement" on open-domain and multiple-choice QA [14] | One call; training optional |
| HyDE | Generate a hypothetical answer document, embed it, retrieve real neighbours [10] | "significantly outperforms" Contriever and is "comparable to fine-tuned retrievers" zero-shot [10] | The fake document "may contain false details" [10] |
| Query2doc | Append a few-shot pseudo-document to the query [11] | BM25 up "3% to 15%" on MS-MARCO and TREC DL, no fine-tuning [11] | One call per query |
| LLM query expansion | Prompted expansion; CoT prompts work best [12] | Beats classical expansion on MS-MARCO and BEIR [12] | Fails when the model lacks the knowledge or the query is ambiguous [13] |
| Multi-query / RAG-Fusion | Several generated queries, merged with RRF [15] | Manual evaluation at Infineon: "accurate and comprehensive answers" [15] | Some answers "strayed off topic" [15]; n retrievals |
| Step-back | Ask a more abstract question first, reason from principles [16] | PaLM-2L: MMLU Physics +7%, Chemistry +11%, TimeQA +27%, MuSiQue +7% [16] | Extra call; reasoning-heavy tasks |
| Decomposition (interleaved) | Retrieve per reasoning step [17] | Covered under rung 5 [17] | Several calls |
| Routing by complexity | A small classifier picks no-retrieval, single-step or iterative [18] | Better efficiency and accuracy than adaptive baselines on open-domain QA [18] | A trained classifier |
| Self-query | LLM turns the question into a structured query plus metadata filter [19] | Framework feature, no benchmark published [19] | Needs clean metadata |

**The evidence is uneven.** RAG-Fusion's paper is a single-company manual evaluation without a controlled baseline [15]. The strongest negative result is recent: LLM-based expansion "can significantly degrade the retrieval effectiveness when knowledge in the LLM is insufficient or query ambiguity is high" [13]. That is exactly the private-corpus case (internal service names, project jargon), so test expansion on your own golden set before adopting it [13].

**Self-query in practice.** LangChain's `SelfQueryRetriever` is described as a "Retriever that uses a vector store and an LLM to generate the vector store queries", with a translator "for turning internal query language into VectorStore search params" [19]. It only works if the filter fields exist and are populated at ingest [19].

**Decision rule.** Routing first (cheap, and it can choose no retrieval at all) [18]; self-query when queries carry filters [19]; HyDE or Query2doc for short queries against long documents in a domain the model knows [10][11]; skip expansion for unfamiliar jargon [13]; step-back for reasoning-heavy questions [16].

**Climb signal.** When the right answer is spread over a long document or many small facts, rewriting the query does not help; change what is indexed (rung 4) [22][23].

## 4. Index-side enrichment beyond rung 2

These techniques change the unit and shape of what is stored, which Dense X shows "significantly impacts the performance of both retrieval and downstream tasks" [22]. Each has a vector-store requirement worth checking before adopting it [25][26].

| Technique | What is indexed | Evidence | Store requirement |
|---|---|---|---|
| Late interaction (ColBERTv2) | One vector per token; token-level scoring at query time [20] | "state-of-the-art quality within and outside the training domain" [20] | Multi-vector; footprint "an order of magnitude" larger before compression, which ColBERTv2 cuts 6 to 10 times [20] |
| Page-image retrieval (ColPali) | VLM multi-vector embeddings of page images [21] | "largely outperforms modern document retrieval pipelines" on the ViDoRe benchmark [21] | Multi-vector; embeds page images directly instead of extracting text [21] |
| Propositions (Dense X) | Atomic, self-contained factoids [22] | Proposition indexing "significantly outperforms passage-level units in retrieval tasks" [22] | Many more rows; one LLM pass to extract |
| Hierarchical summaries (RAPTOR) | Recursive cluster-and-summarise tree [23] | With GPT-4, +20% absolute accuracy on QuALITY [23] | Summary nodes alongside leaves; LLM indexing cost |
| Parent-document retrieval | Small chunks for matching, larger parents returned [24] | Framework feature, no benchmark published [24] | Vector store plus a document store keyed by parent ID [24] |

**Why the store matters.** Qdrant supports multivectors "as of v1.10.0" with a `max_sim` comparator, "a sum of maximum similarities between each pair of vectors" [26]. pgvector's documented types are `vector` (indexable "up to 2,000 dimensions"), `halfvec` (4,000), `bit` and `sparsevec`, and its README lists no multi-vector comparator [25]. So late interaction and ColPali on Postgres mean either a rerank stage outside the database or a different store [25][26].

**Parent documents, explained by the implementer.** "You may want to have small documents, so that their embeddings can most accurately reflect their meaning", but also long enough that "the context of each chunk is retained"; the retriever "first fetches the small chunks but then looks up the parent IDs for those chunks and returns those larger documents" [24]. It is a context-shaping tool, which is why it pairs with rung 9 budgets [24].

**Decision rule.** Scanned PDFs, slides and tables: page-image retrieval [21]. Long narrative documents with whole-document questions: RAPTOR-style summaries [23]. Fact-dense corpora with short answers: propositions [22]. Precise matches but thin answers: parent documents [24]. Late interaction only when you can afford the storage and your store supports it [20][26].

**Climb signal.** If a single retrieval pass, however good, cannot gather everything an answer needs (multi-hop questions, *what changed since*, contradictory sources), the model has to retrieve more than once [17][27].

## 5. Corrective and agentic retrieval

**Retrieval that checks itself.** Self-RAG trains one model to retrieve "on-demand" and to critique passages and its own output with "reflection tokens"; at 7B and 13B it outperformed ChatGPT and retrieval-augmented Llama2-chat on open-domain QA, reasoning and fact verification [27]. CRAG adds "a lightweight retrieval evaluator" that scores retrieved documents and triggers different actions, including "large-scale web searches", and a "decompose-then-recompose" filter; it is "plug-and-play" over other RAG methods [28]. Adaptive-RAG chooses between no retrieval, single-step and iterative retrieval with a classifier trained on query complexity [18].

**Retrieval as a tool in a loop.** IRCoT interleaves retrieval with chain-of-thought steps, because in multi-step QA "what to retrieve depends on what has already been derived"; with GPT3 it improved retrieval by up to 21 points and QA by up to 15 points across HotpotQA, 2WikiMultihopQA, MuSiQue and IIRC [17]. Search-R1 trains the search behaviour with RL and reports +41% (Qwen2.5-7B) and +20% (Qwen2.5-3B) over RAG baselines on seven QA datasets [29]. Anthropic's framing is "just in time" context: agents "maintain lightweight identifiers" such as file paths, stored queries and web links, and load data through tools at run time, while "the most effective agents might employ a hybrid strategy, retrieving some data up front for speed" [2].

**The code-search debate, with primary statements.**

| Position | Primary statement | Evidence offered | Date |
|---|---|---|---|
| No index: agentic grep and file reads | Claude Code's Boris Cherny: "In our testing we found that agentic search out-performed RAG for the kinds of things people use Code for." [30] | Internal testing; no numbers published [30] | 2025-02-24 |
| Same, with the reasoning | Claude Code drops CLAUDE.md in up front while "primitives like glob and grep" retrieve files just in time, "effectively bypassing the issues of stale indexing and complex syntax trees" [2] | Design rationale [2] | 2025-09-29 |
| Keep a semantic index | Cursor: semantic search gave "on average 12.5% higher accuracy in answering questions", 6.5% to 23.5% by model; code retention +0.3%, and +2.6% "on large codebases with 1,000 files or more" [31] | Offline eval plus online A/B (vendor-reported) [31] | 2025-11-06 |
| Ranked structural map | Aider builds a tree-sitter map of definitions and ranks it with "a graph ranking algorithm" on a file-dependency graph, fitted to `--map-tokens`, which "defaults to 1k tokens" [32] | Design; no ablation published [32] | 2023-10-22 |

The honest reading, which `multi-agent-systems-in-practice.md` section 4 documents in more detail, is that the only published A/B is vendor-run and modest on retention, and no one has published the same harness with grep versus a code graph [31]. Both sides agree on a hybrid: Anthropic calls Claude Code's own design "this hybrid model" [2], and Cursor writes that "the combination of these two leads to the best outcomes" [31].

**Cost.** Every loop iteration is another model call and another set of tool results in the window; HippoRAG's comparison puts iterative retrieval like IRCoT at 10 to 30 times the cost of its single-step method [38]. Section 9's budget and compaction rules apply with more force here [2][61].

**Climb signal.** When questions are about the corpus as a whole (*what are the main themes*, *which teams touch billing*) or require hopping across named entities, an agent grepping chunks will burn calls without converging; that is the graph rung's territory [33][38].

## 6. Graph RAG

**Microsoft GraphRAG.** An LLM builds "an entity knowledge graph from the source documents", then pregenerates "community summaries for all groups of closely related entities"; global questions are answered by map-reduce over those summaries [33]. On "global sensemaking questions over datasets in the 1 million token range" it gave "substantial improvements over a conventional RAG baseline for both the comprehensiveness and diversity of generated answers" [33]. The repository warns: "GraphRAG indexing can be an expensive operation, please read all of the documentation to understand the process and costs involved, and start small" [34]. DRIFT search (2024-10-31) adds community information to local search; it beat local search on comprehensiveness "78% of the time" and on diversity "81% of the time" [35].

**Cheaper variants.** LazyGraphRAG (2024-11-25) defers LLM work to query time: "LazyGraphRAG data indexing costs are identical to vector RAG and 0.1% of the costs of full GraphRAG", with "comparable answer quality to GraphRAG Global Search for global queries, but more than 700 times lower query cost" (vendor-reported) [36]. LightRAG combines graph structure with vectors in "a dual-level retrieval system" and adds "an incremental update algorithm" for new data [37].

**Associative memory variants.** HippoRAG runs Personalized PageRank over an LLM-built knowledge graph and reports multi-hop QA gains "by up to 20%", with single-step retrieval comparable to IRCoT at 10 to 30 times lower cost and 6 to 13 times faster [38]. Its successor names graph RAG's main regression: structure-augmented methods' "performance on more basic factual memory tasks drops considerably below standard RAG"; HippoRAG 2 claims to fix this and reports "a 7% improvement in associative memory tasks over the state-of-the-art embedding model" [39]. Temporal graphs for agents (Graphiti inside Zep) are covered under rung 8 [42].

**When graph beats vector, and when it does not.**

| Finding | Source |
|---|---|
| "recent studies report that GraphRAG frequently underperforms vanilla RAG on many real-world tasks" | GraphRAG-Bench [41] |
| "basic RAG is comparable to or outperforms GraphRAG in simple fact retrieval tasks" | GraphRAG-Bench v3, observation 1 [41] |
| GraphRAG models "show a clear advantage in complex reasoning, Contextual Summarize, and creative generation" | GraphRAG-Bench v3, observation 2 [41] |
| Average tokens per query on its Novel dataset: vanilla RAG 879, MS-GraphRAG local 38,707, global 331,375, HippoRAG2 1,008 | GraphRAG-Bench v3, Figure 9 [41] |
| RAG and GraphRAG show "distinct strengths" by task; combining them gives "consistent performance improvements" | RAG vs. GraphRAG [40] |

The same GraphRAG-Bench text also puts global search prompts at up to about 40,000 tokens, which does not match its own Figure 9 average; treat the token numbers as order-of-magnitude [41].

**Construction cost and quality.** Every LLM-built graph pays an extraction pass over the whole corpus [33][34], HippoRAG 2 confirms the factual-recall regression [39], and GraphRAG-Bench attributes it to graph processing that "may introduce redundant or noisy information for simpler queries" [41].

**Decision rule.** Global, thematic questions over a large narrative corpus: GraphRAG global or LazyGraphRAG [33][36]. Multi-hop entity questions: HippoRAG 2 class, which keeps token cost near vanilla RAG [39][41]. Simple fact lookup: stay on rungs 2 to 4 [41]. Route between them rather than replacing vector RAG [40].

**Climb signal.** If the corpus is source code, the entities and edges already exist precisely in the language toolchain; do not ask an LLM to guess them [43][48].

## 7. Code graphs and code intelligence as context

**Two kinds of code navigation.** Sourcegraph distinguishes "search-based" navigation, "powered by tools like ctags and tree-sitter", from "precise" navigation from compiler-grade indexers [44]. SCIP is "a language-agnostic protocol for indexing source code, which can be used to power code navigation functionality such as Go to definition, Find references, and Find implementations" [43]; it was announced on 2022-06-08 as "a better code indexing format than LSIF" after Sourcegraph had built "dozens of LSIF indexers" [44]. Indexers exist per language (for example `scip-typescript`, `scip-python`, `scip-java`, `scip-clang`) [43].

**Live language servers.** Claude Code's code-intelligence plugins connect "to a language server for one language through the Language Server Protocol (LSP)" so Claude "catches type errors and missing imports that its own edits introduce" and "finds definitions and references by symbol instead of by text search" [48]. Caveat from the same page: "In cloud sessions, Claude Code doesn't start plugin language servers" [48]. Serena exposes language servers to agents over MCP and claims support "for over 40 programming languages" [49].

**Other structures.** GitHub's stack graphs did name resolution incrementally, looking "at each file completely in isolation" so unchanged files are reused [45], but the repository "was archived by the owner on Sep 9, 2025" and is "no longer supported or updated by GitHub" [46]. Code property graphs, "first introduced in the paper Modeling and Discovering Vulnerabilities with Code Property Graphs" for vulnerability discovery in C code, combine three low-level code representations in a graph database queried through a graph-traversal DSL; Joern's docs describe the structure as "designed to mine large codebases for instances of programming patterns", a security-analysis lineage rather than agent context [47].

**Repo-level graph retrieval research.**

| System | Graph | Measured result | Setting |
|---|---|---|---|
| RepoGraph | Repository-level code graph as a plug-in module [50] | "average relative improvement of 32.8%" in success rate across four methods [50] | SWE-bench Lite [50] |
| CodexGraph | Code graph database queried by the agent in a graph query language [51] | "competitive performance" [51] | CrossCodeEval, SWE-bench, EvoCodeBench [51] |
| LocAgent | Heterogeneous graph of files, classes, functions with imports, invocations, inheritance [52] | Up to 92.7% file-level localization; about 86% cost reduction with fine-tuned Qwen-2.5-Coder-Instruct-32B; +12% Pass@10 issue resolution [52] | Code localization benchmarks [52] |

**Freshness is the hard part.** Claude Code's stated reason for grep is avoiding "stale indexing" [2]. Cursor's answer is a Merkle tree "which lets it detect exactly which files and directories have changed without reprocessing everything", plus reuse of a teammate's index: time-to-first-query fell from 7.87 seconds to 525 milliseconds at the median and from 4.03 hours to 21 seconds at the 99th percentile (vendor-reported) [53]. Stack graphs made incrementality a design constraint for the same reason [45].

**How to expose it to an agent.** The useful surface is small and verb-shaped: symbol search, definition, references (callers), callees, and impact of a change, returning short ranked lists rather than files [43][48][52]. Tool design and naming are covered in `mcp-server-tool-design.md`.

**Climb signal.** When the agent's problem is not finding code but remembering decisions, preferences and facts across sessions and sources, the next rung is memory [54][60].

## 8. Agent memory, and what "context lake" means

**Memory systems.**

| System | Idea | Claim | Status of the claim |
|---|---|---|---|
| MemGPT (now Letta) | "virtual context management" modelled on OS memory tiers, moving data between fast and slow memory [54] | Handles documents and multi-session chat beyond the window [54] | Qualitative [54] |
| Mem0 | Extract, consolidate and retrieve salient facts; optional graph memory [55] | 26% relative LLM-as-a-Judge gain over the OpenAI baseline; 91% lower p95 latency and over 90% token savings versus full context on LoCoMo [55] | Disputed [58][59] |
| Zep / Graphiti | "temporally-aware knowledge graph engine" over conversations and business data [42] | DMR 94.8% vs 93.4% for MemGPT; LongMemEval accuracy up to +18.5% with 90% lower latency [42] | Vendor paper [42] |
| Anthropic memory tool | Claude "stores what it learns in files under /memories"; the tool "operates client-side" and your handler maps the path to storage [60] | Not a benchmark claim [60] | Product docs [60] |

**The benchmarks.** LoCoMo dialogues average "300 turns and 9K tokens" over "up to 35 sessions" [56]. LongMemEval has 500 questions testing five abilities (extraction, multi-session reasoning, temporal reasoning, knowledge updates, abstention) and found "a 30% accuracy drop" for commercial assistants and long-context LLMs [57].

**The disputes.** Zep's rebuttal to Mem0 (2025-05-06) argues Mem0's paper rests on "a flawed benchmark (LoCoMo) and a demonstrably incorrect implementation of a competitor system (Zep)", notes LoCoMo's Category 5 "was unusable due to missing ground truth answers", and later corrected its own figure to "75.14% +/- 0.17" [58]. Letta (2025-08-12) reported that an agent "simply storing conversation histories in files" scored 74.0% on LoCoMo with GPT-4o mini, above "Mem0's reported 68.5%", and concluded it is "much more important to consider whether an agent will be able to effectively use a retrieval tool" than the retrieval mechanism [59]. Treat every memory leaderboard number as vendor-run until reproduced [58][59].

**Context management primitives from the model vendor.** Anthropic's context editing clears stale tool results with `clear_tool_uses_20250919` once context crosses a threshold, clears thinking blocks with `clear_thinking_20251015`, and offers SDK compaction, noting "server-side compaction is generally preferred" [61].

**"Context lake": a vendor label, not a standard.** The term has at least three incompatible public definitions:

| Who | Definition (verbatim) | Emphasis |
|---|---|---|
| Tacnode (Xiaowei Jiang, CEO, 2025-08-15) | "a unified infrastructure layer designed from first principles to deliver live, multi-modal context at the speed and scale of AI decision-making" [62] | Transactional freshness: "Not eventual consistency, not micro-batches, not replication lag" [62] |
| contextlake.org and arXiv 2601.17019 (same author, 2026-01-15) | "A Context Lake is the system class defined by Decision Coherence" [63]; requires "semantic operations as native capabilities", "transactional consistency over all decision-relevant state" and "operational envelopes bounding staleness and degradation under load" [64] | A formal system class |
| Zep (page updated 2026-05-31) | "a governed system of context graphs" for agent memory at enterprise scale [65] | Temporal graphs, ABAC, retention, provenance [65] |
| Port (glossary) | "an aggregated, structured repository where all the information an AI agent needs to operate is stored, correlated, and governed" [66] | Engineering metadata: services, ownership, incidents [66] |

Each definition is published by a vendor (Tacnode's CEO wrote both the Tacnode page and the arXiv paper), and none of them points to a standards body or an open specification for the term [62][64][65][66]. The neighbouring idea of a lakehouse as context store (Altertable, 2026-07-15) says "A useful context store has two parts": a map of definitions and relationships, and evidence such as records, events, logs and traces [67].

**The architecture pattern underneath.** Stripping the labels, the shared pattern is one governed retrieval surface over documents, structured data, code, events and conversation records, with these properties [62][65][66][67]:

| Property | What it means | A public primitive for it |
|---|---|---|
| Freshness | Embeddings and indexes follow source changes | pgai Vectorizer keeps "the embeddings synced with the underlying data as it changes" [68]; Merkle-tree sync for code [53] |
| Permission-aware retrieval | A user only retrieves what they may read | Postgres row security policies "restrict, on a per-user basis, which rows can be returned by normal queries" [69]; Cursor drops results when "the client can't prove it has a file" [53] |
| Provenance | Every chunk carries its source; answers cite | Claude citations "track and verify the sources behind each response" [75]; Zep governance lists provenance and audit [65] |
| Access protocol | Agents reach it through one interface | MCP servers offer "Resources", "Prompts" and "Tools"; the 2026-07-28 revision has "Stateless, self-contained requests" [71] |
| Storage | Where vectors and records live | pgvector-class extensions [25]; object-store-native vectors such as S3 Vectors, "up to 90%" cheaper and up to 2B vectors per index (vendor-reported) [70]; lakehouse tables [67] |

The SQL below is illustrative (not run): permission-aware vector retrieval with row security, so the database rather than the prompt enforces who sees what [25][69].

```sql
-- Illustrative: each chunk row carries its ACL and provenance.
ALTER TABLE chunks ENABLE ROW LEVEL SECURITY;
CREATE POLICY chunk_read ON chunks FOR SELECT
  USING (acl_groups && string_to_array(current_setting('app.groups'), ','));

-- Per request, the service sets the caller's groups, then queries.
SET app.groups = 'eng,billing';
SELECT source_uri, chunk_no, updated_at, body
FROM chunks
ORDER BY embedding <=> $1
LIMIT 20;
```

**Climb signal.** None: rung 9 is not optional once anything is retrieved [2][72].

## 9. Context assembly and efficiency

Retrieval decides what is eligible; assembly decides what the model actually sees, in what order, with what labels [2][72].

| Lever | Evidence | Rule |
|---|---|---|
| Token budget | Anthropic: the "smallest set of high-signal tokens" [2] | Fix a budget per call; fill by marginal value, not top-k |
| Ordering | Performance "is often highest when relevant information occurs at the beginning or end of the input context" and degrades in the middle [72] | Strongest evidence at the edges |
| Diversity | MMR "strives to reduce redundancy while maintaining query relevance"; λ=1 is plain relevance ranking, λ=0 maximal diversity [73] | Penalise similarity to already-chosen chunks |
| Deduplication | Near-duplicates waste budget and add noise [7][73] | Hash plus near-duplicate threshold before MMR |
| Compression | LLMLingua allows "up to 20x compression with little performance loss" on GSM8K, BBH, ShareGPT and Arxiv-March23 [74] | Compress long, low-precision context, not instructions |
| Provenance | Citations let users "track and verify the sources" [75] | Tag every chunk with a stable source ID |
| Just-in-time vs preload | Agents keep "lightweight identifiers" and load on demand; hybrid preloads some data [2] | Preload stable, small, always-needed context; fetch the rest |
| Compaction and notes | Compaction, structured note-taking, and sub-agents that return "a condensed, distilled summary", often 1,000 to 2,000 tokens [2]; Manus's `todo.md` "recitation" [76] | Summarise and restart long loops; keep plans in files |
| Tool-result clearing | `clear_tool_uses_20250919` drops stale results past a threshold [61] | Clear in chunks; see the caching doc for prefix effects |

Caching interactions (what invalidates a prefix, why clearing should be chunky) are in `llm-token-cache-efficiency.md`. Manus's cache-first stance is the counterweight to aggressive per-turn editing: stable prefixes are worth protecting [76].

The example below is dependency-free and runs on Python 3.10 or later; it applies a relevance floor, drops near-duplicates, selects with MMR under a token budget, places the two strongest chunks at the edges, and tags provenance [72][73][75].

```python
import math
import re
from collections import Counter

# Candidates as a retriever would return them: (source, chunk_id, score, text).
CANDIDATES = [
    ("rag.md", 3, 0.91, "RRF merges ranked lists by summing 1/(k+rank)."),
    ("rag.md", 4, 0.90, "RRF merges ranked lists by summing 1/(k + rank)!"),
    ("fusion.md", 1, 0.88, "RRF sums 1/(k+rank) over retrievers; k is often 60."),
    ("pgvector.md", 7, 0.80, "HNSW trades build time for recall via ef_construction."),
    ("eval.md", 2, 0.78, "Use nDCG@10 for reordering and recall@k for chunking."),
    ("chunking.md", 5, 0.74, "Never split inside a code fence; size chunks in tokens."),
    ("misc.md", 9, 0.40, "Unrelated note about office plants and watering."),
]


def tokens(text):
    return re.findall(r"[a-z0-9@]+", text.lower())


def est_tokens(text):
    return max(1, len(text) // 4)  # rough 4 chars per token for prose


def cosine(a, b):
    dot = sum(a[t] * b[t] for t in a)
    na = math.sqrt(sum(v * v for v in a.values()))
    nb = math.sqrt(sum(v * v for v in b.values()))
    return dot / (na * nb) if na and nb else 0.0


def assemble(cands, budget=60, lam=0.7, min_score=0.5, near_dup=0.9):
    # 1. Drop low-relevance candidates, then exact and near duplicates.
    kept, vecs = [], []
    for c in sorted(cands, key=lambda c: -c[2]):
        if c[2] < min_score:
            continue
        v = Counter(tokens(c[3]))
        if any(cosine(v, u) >= near_dup for u in vecs):
            continue
        kept.append(c)
        vecs.append(v)
    # 2. MMR: lam * relevance - (1 - lam) * max similarity to what is chosen.
    chosen, used = [], 0
    pool = list(zip(kept, vecs, strict=True))
    while pool:
        def mmr(item):
            (c, v) = item
            red = max((cosine(v, cv) for _, cv in chosen), default=0.0)
            return lam * c[2] - (1 - lam) * red
        best = max(pool, key=mmr)
        pool.remove(best)
        cost = est_tokens(best[0][3])
        if used + cost > budget:
            continue  # skip what does not fit; a smaller one may
        chosen.append(best)
        used += cost
    # 3. Order: strongest first and second strongest last (edges of context).
    ranked = [c for c, _ in chosen]
    ordered = ranked[::2] + ranked[1::2][::-1]
    # 4. Provenance tags the model can cite.
    lines = [f"[S{i}] ({c[0]}#{c[1]}) {c[3]}" for i, c in enumerate(ordered, 1)]
    return "\n".join(lines), used


if __name__ == "__main__":
    block, used = assemble(CANDIDATES)
    print(block)
    print(f"tokens used: {used} of 60")
```

Observed output: the near-duplicate `rag.md#4` and the low-score `misc.md#9` are dropped, MMR defers the redundant `fusion.md#1` until the budget is spent, and the second-strongest chunk (`pgvector.md#7`) lands last:

```text
[S1] (rag.md#3) RRF merges ranked lists by summing 1/(k+rank).
[S2] (chunking.md#5) Never split inside a code fence; size chunks in tokens.
[S3] (eval.md#2) Use nDCG@10 for reordering and recall@k for chunking.
[S4] (pgvector.md#7) HNSW trades build time for recall via ef_construction.
tokens used: 50 of 60
```

## 10. Evaluation

**Component metrics.** Retrieval metrics (nDCG@10 for reordering changes, recall@k for chunking changes, token-level IoU for size and overlap) and how to build a golden set are in `rag-retrieval-fusion-and-chunking.md` section 5. BEIR is the usual zero-shot retrieval benchmark to report: 18 datasets, 10 retrieval systems evaluated [9].

**End-to-end metrics.**

| Metric | Definition | Source |
|---|---|---|
| Context precision | "the retriever's ability to rank relevant chunks higher than irrelevant ones" | Ragas [78] |
| Context recall | "how many of the relevant documents (or pieces of information) were successfully retrieved" | Ragas [79] |
| Faithfulness | "how factually consistent a response is with the retrieved context" | Ragas [80] |
| RAG triad | Context relevance, groundedness, answer relevance | TruLens [81] |

Ragas positions these as usable "without having to rely on ground truth human annotations" [77]. TruLens's triad checks each hop: that "each chunk of context is relevant to the input query", that the answer is grounded in it, and that the response does "helpfully answer the original question" [81].

**Public benchmarks worth knowing.**

| Benchmark | Measures | Headline | Source |
|---|---|---|---|
| CRAG | Factual QA with mock web and KG search; 4,409 pairs, five domains | Advanced LLMs "<=34%"; straightforward RAG "only to 44%"; best industry RAG answers "63% of questions without any hallucination" | [82] |
| LoCoMo | Very long-term conversational memory | 300 turns, up to 35 sessions | [56] |
| LongMemEval | Five memory abilities, 500 questions | 30% accuracy drop for assistants | [57] |
| GraphRAG-Bench | When graphs help, by task difficulty | Basic RAG matches GraphRAG on simple facts | [41] |
| NoLiMa | Long-context retrieval without literal matches | 11 of 13 models below half their baseline at 32K | [6] |
| SWE-bench Lite | Repo-level issue resolution (used for code-graph claims) | RepoGraph +32.8% relative | [50] |

**LLM-judge caveats.** MT-Bench found strong judges reach "over 80% agreement" with humans, and also documented "position, verbosity, and self-enhancement biases" [83]. Mem0 versus Zep is a live example of judge-scored benchmarks producing contested results [55][58].

**Gating retrieval changes in CI.** Seven-failure-points' lesson that validation "is only feasible during operation" means the golden set must keep growing from production misses [8]. A workable gate: run component metrics on every retrieval change, end-to-end triad metrics on a fixed sample, fail the build on regressions beyond a tolerance, and report judge scores with the judge model pinned [77][81][83].

## 11. Decision table: the lowest rung that suffices

| Corpus | Query type | Budget | Lowest rung | Evidence |
|---|---|---|---|---|
| Under about 200,000 tokens, stable | Any | Cache-friendly | 0: full context, cached | [4][5] |
| Docs, any size | Keyword and identifier heavy | Low latency | 2: hybrid plus rerank | [4][9] |
| Docs with metadata | Filters, such as *2025 incidents in service X* | One extra call OK | 3: self-query | [19] |
| Docs, model knows the domain | Short, vague | One extra call OK | 3: HyDE or Query2doc | [10][11] |
| Internal jargon | Short, vague | Any | 2, plus synonyms in the index; avoid LLM expansion | [13] |
| Scanned PDFs, slides, tables | Visual layout matters | Storage OK | 4: page-image multi-vector | [21] |
| Long narrative docs | Whole-document questions | Indexing spend OK | 4: RAPTOR-style summaries | [23] |
| Any | Multi-hop | Seconds of latency OK | 5: iterative or agentic retrieval | [17][29] |
| Large narrative corpus | Global themes | Indexing spend OK | 6: GraphRAG global or LazyGraphRAG | [33][36] |
| Entity-rich corpus | Multi-hop entity questions | Moderate | 6: HippoRAG 2 class | [39][41] |
| Source code, agent | Navigation and edits | Interactive | 5: grep plus file reads, then 7: LSP or index | [2][31][48] |
| Conversations and user facts | Recall across sessions | Moderate | 8: file or tool memory before a memory product | [59][60] |
| Many sources, many users | Governed access | Platform budget | 8: governed surface with ACLs and provenance | [53][69][75] |

Climb only when your eval says so: every rung above 2 adds latency, cost or an index to keep fresh [8][34][53].

## Agreed vs folklore (compressed)

**Agreed** (multiple primary sources): context is a budget to curate, not a bucket to fill [2][3] · long context degrades with length even when it fits [6][7][72] · below roughly 200,000 tokens, try no retrieval first [4][5] · a lexical arm and a reranker are large, well-measured wins [4][9] · graph RAG helps global and multi-hop questions and can hurt simple lookups [33][39][41] · code agents work best with a hybrid of on-demand search and some precomputed structure [2][31] · memory benchmarks are contested and vendor-run [58][59] · validation has to continue in production [8].

**Folklore** (believed, repeated, contradicted or unproven):

| Claim | What the evidence says |
|---|---|
| *RAG means a vector database* | BM25 is "a robust baseline" [9]; Claude Code retrieves with grep and no index [2][30]; a file-based agent scored 74.0% on LoCoMo [59] |
| *Long context windows killed RAG* | Long context wins on quality when resourced, RAG on cost [5]; 11 of 13 models fall below half their baseline at 32K without literal matches [6] |
| *GraphRAG is strictly better* | Basic RAG matches or beats it on simple facts at a fraction of the tokens [41]; indexing is "expensive" [34] |
| *Agents do not need an index* | Cursor measured +12.5% QA accuracy with semantic search (vendor-reported) [31]; LSP gives symbol navigation grep cannot [48] |
| *More context is better* | Performance degrades in the middle [72] and with length [7]; Anthropic cut over 80% of a system prompt with no measured loss [3] |
| *Embeddings are enough for code* | Precise navigation comes from compiler-grade indexers and language servers [44][48]; graph-guided localization reaches 92.7% file-level accuracy [52] |
| *Context lake is an industry standard* | Three vendors' incompatible definitions, no open spec [62][64][65][66] |

## Synthesis (inferred)

**The ladder as a decision rule.** Measure first, then climb one rung at a time, and only on the failure your eval shows. Rung 0 and rung 2 cover most document corpora; rung 3 is a per-query-type patch, not a default; rung 4 is chosen by document shape; rung 5 by question shape; rung 6 only for global or multi-hop entity questions; rung 7 is the native structure for code and should come from the toolchain, not an LLM; rung 8 is governance more than retrieval; rungs 9 and 10 apply at every level. A system that skipped rung 10 is not on any rung; it is guessing.

**Reference architecture for a small team's context lake.** One Postgres (pgvector plus a real BM25 extension) as the system of record for chunks, with columns for `source_uri`, `chunk_no`, `updated_at`, `acl_groups` and a content hash; row security for permissions; a change-driven vectorizer (pgai-style or a queue fed by source webhooks) for freshness; a code-intelligence service (LSP or SCIP index) kept separate because its freshness model is per-commit; file-based agent memory with explicit write tools before any memory product; one MCP server exposing a handful of verb-shaped tools (search, fetch by ID, symbol lookup, references) that return IDs plus short excerpts, so the agent fetches just in time; an assembly layer like the example above; and a golden set wired into CI. Move vectors to an object-store-native index only when Postgres storage cost, not query latency, becomes the constraint. Calling this a "context lake" is optional; the properties are what matter.

**Application note for this corpus (inferred).** The mx techniques corpus is 82 docs and 2,857 chunks in Postgres with pgvector and `text-embedding-3-small`, served through MCP `rag_*` tools, alongside a separate code-graph MCP server. At roughly 1,200 characters per chunk that is on the order of 3.4 million characters, roughly 850,000 tokens at 4 characters per token, well above the rung 0 threshold, so retrieval is justified. Exposing retrieval as tools the agent calls on demand, plus a code graph, puts the stack at rung 5 for access pattern and rung 7 for code, but its retrieval core sits at rung 1 to 2: whether the lexical arm actually contributes is the open question `rag-retrieval-fusion-and-chunking.md` raised. The next cheapest move is not a graph or a memory layer. It is rung 10 then rung 2: build the 50 to 100 query golden set, confirm the lexical arm moves rankings (or replace it with BM25 and RRF), and add a reranker; after that, the assembly rules in section 9 (dedup, MMR, provenance IDs in `rag_*` results) are cheap and directly reduce wasted tokens in agent sessions. GraphRAG is not indicated: the corpus is modest in size, and its questions are mostly "how do I do X", which are local lookups.
