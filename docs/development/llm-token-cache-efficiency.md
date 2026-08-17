---
title: "LLM Token & Cache Efficiency Engineering for Agentic Coding"
category: process
complexity: advanced
use_cases:
  - keeping prompt-cache hit rates high in Claude Code / API agent sessions
  - correcting delegation cost math across model tiers (and the stale 15× claim)
  - session hygiene — what invalidates cache, what side questions really cost
  - instrumenting token/cache telemetry (OTel) and hard spend guardrails
  - evaluating local inference against API pricing honestly
summary: "Evidence-audited mechanics of prompt caching (Anthropic/OpenAI/Gemini, Aug 2026) and what follows for agentic coding shops: the real TTLs (5m/1h — the '90-minute cache' is false), the invalidation hierarchy and cache-safe vs cache-destroying actions, corrected delegation economics (Haiku is 5× cheaper than Opus, not 15×), fork-vs-subagent cache behavior, context-rot evidence, and prioritized recommendations with measured case studies (7%→74% hit rate, $37.9k miss incident)."
provenance: researched
researched: 2026-08-14
sources:
  - https://platform.claude.com/docs/en/build-with-claude/prompt-caching
  - https://platform.claude.com/docs/en/about-claude/pricing
  - https://platform.claude.com/docs/en/api/rate-limits
  - https://platform.claude.com/docs/en/build-with-claude/context-editing
  - https://code.claude.com/docs/en/prompt-caching
  - https://code.claude.com/docs/en/costs
  - https://code.claude.com/docs/en/sub-agents
  - https://claude.com/blog/lessons-from-building-claude-code-prompt-caching-is-everything
  - https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents
  - https://developers.openai.com/api/docs/guides/prompt-caching
  - https://developers.openai.com/api/docs/pricing
  - https://ai.google.dev/gemini-api/docs/caching
  - https://ai.google.dev/gemini-api/docs/pricing
  - https://arxiv.org/abs/2601.06007
  - https://www.trychroma.com/research/context-rot
  - https://docs.vllm.ai/en/stable/design/prefix_caching/
  - https://www.lmsys.org/blog/2024-01-17-sglang/
  - https://john-hodge.com/blog/opentelemetry-genai-semantic-conventions/
  - https://www.digitalocean.com/community/conceptual-articles/prompt-caching-in-practice-hit-rate
  - https://news.ycombinator.com/item?id=47933355
  - https://github.com/anthropics/claude-code/issues/51218
  - https://www.apple.com/mac-studio/specs/
  - https://news.ycombinator.com/item?id=49065752
  - https://arxiv.org/pdf/2406.18665
---

# LLM Token & Cache Efficiency Engineering

Cache and token mechanics as of 2026-08, from provider docs and measured case studies. Inline `[n]` cites key to `sources`. The one-sentence model: **cache warmth is governed by idle-gap-vs-TTL and prefix stability — not by elapsed session time.**

## 1. Cache mechanics, exactly

**Anthropic** [1][2][3]: up to 4 breakpoints per request; writes happen only at breakpoints; reads look back ≤20 blocks per breakpoint. Multipliers: 5-min write 1.25×, 1-hour write 2×, **read 0.1×** — 5-min caching pays for itself after one read. Default TTL **5 minutes, refreshed on every hit at no cost**; 1-hour via `ttl: "1h"` (longer-TTL entries must precede shorter ones). **The TTL clock starts at the request's start, not the response's end** — a 4-minute streamed response leaves ~1 minute to send the follow-up [1]. Minimum cacheable length varies by model: Opus 5/Fable 5 = **512** tokens, Sonnet 5 = 1,024, **Haiku 4.5 = 4,096** — tiny Haiku subagent prompts may silently not cache at all (both cache counters return 0, no error) [1]. Invalidation is hierarchical: `tools` → `system` → `messages`; a tool-definition change invalidates everything; toggling web search/citations edits the system prompt [1]. Cache reads don't count toward ITPM rate limits — 80% hit rate effectively 5×'s your throughput ceiling [3]. Usage-object trap: `input_tokens` counts only post-breakpoint tokens; real input = `cache_read + cache_creation + input_tokens` — dashboards reading `input_tokens` alone under-report catastrophically [1].

**OpenAI** [10][11]: automatic ≥1,024 tokens, exact prefix match, cached input 0.1×; in-memory prefixes live ~5–10 min idle (max 1h) with 24h retention opt-in (`prompt_cache_retention`); `prompt_cache_key` for routing; GPT-5.6+ adds explicit breakpoints (convergence toward Anthropic's model).

**Gemini** [12][13]: implicit caching default on 2.5+; explicit caching unavailable in the Interactions API; **the only provider charging hourly cache storage** ($1–4.50/MTok/hour) — a 200k explicit cache on 2.5 Pro costs $0.90/hour just existing. Gemini's model punishes idle caches; Anthropic/OpenAI punish churn.

**Current Anthropic pricing** (per MTok, in/out): Fable 5 $10/$50 · **Opus 5 $5/$25** · **Sonnet 5 $2/$10** (introductory price now permanent) · Haiku 4.5 $1/$5. Batch 50% off; fast mode (Opus) $10/$50 [2]. **Tokenizer trap: Claude ≥4.7 tokenizes ~30% more tokens for the same text than ≤4.6** — cross-boundary $/MTok comparisons are not apples-to-apples [2].

## 2. Fact-check: "Claude Code's cache goes cold after ~90 minutes" — FALSE

No provider documents a 90-minute TTL; only 5 minutes and 1 hour exist [1][5]. What's actually true [5]:

- On a **Claude subscription**, Claude Code requests the 1-hour TTL automatically. On an **API key / Bedrock / Vertex / Foundry**, the default is **5 minutes** — opt in with `ENABLE_PROMPT_CACHING_1H=1`.
- **The credit cliff**: when a subscription exceeds its plan limit and draws usage credits, Claude Code **silently drops 1h → 5m** (1h writes cost more). Same env var overrides it. Largely unknown, and real.
- **Every hit resets the timer** — an actively-worked session stays warm indefinitely; only idle gaps > TTL kill it. There is no cumulative session decay.
- **Subagents always use the 5-minute TTL**, even on a subscription.
- The "90 minutes" number traces to a LOW-confidence reverse-engineered description of a `time_based_microcompact` idle-compaction path — a **compaction** behavior, not cache expiry.

The complaint behind the folklore is real: one documented case shows a 200-token prompt after a ~45-min pause on a ~912k context consuming ~20% of a 5-hour usage window vs <1% back-to-back [21]. The four killers: idle gap > TTL; the credit cliff; any prefix edit; resuming after a Claude Code upgrade (full-history reprocess, zero hits) — "the first turn back into a long session can be the most expensive request you send" [5].

## 3. Cache-aware session practice

**The cache key includes model, effort level, and fast-mode** — switching model or `/effort` mid-session recomputes the whole conversation even byte-identical; `opusplan` makes every plan-mode toggle a model switch [5]. This is the highest-frequency accidental invalidation.

**Cache-safe (append-only)** [5]: editing repo files, editing CLAUDE.md mid-session (safe but inert until restart/clear), permission-mode changes, invoking skills, `/rewind`, spawning subagents, MCP servers connecting/disconnecting *when tool schemas are deferred* (the default). **Cache-destroying**: model/effort switch, first fast-mode turn, MCP flap when tools load into the prefix, adding a bare tool-name deny rule, `/compact`, upgrades. **`/rewind` beats `/compact` when abandoning a path** — it truncates back to an already-cached prefix instead of building a new one.

**Anthropic's own patterns** (they run SEVs on cache hit rate internally) [8]: state transitions via tool calls, not prompt edits (plan mode = `EnterPlanMode` tool, prefix survives); deferred tool loading (stubs in stable order, schemas on demand); compaction **forks the prefix** — the summarization request reuses the exact system prompt + history + one instruction, so warm `/compact` reads the parent's cache. Anti-patterns named: timestamps in static prompts, non-deterministic tool ordering, mid-session tool-parameter changes.

**Side questions are triple-taxed** [5][9]: (1) permanent context growth — the full conversation is re-sent every turn, so the side question is re-read at cache rate for the rest of the session; (2) invalidation risk if it triggers a model/effort/MCP change; (3) attention cost (context rot, §5). Open a second session instead.

**Cache scope is one machine + one directory.** The system prompt embeds cwd/platform/shell/memory paths — **two worktrees of the same repo miss each other's cache**; parallel sessions in the same directory share [5].

**Measured evidence**: the cross-provider academic study finds strategic cache control worth **41–80% cost reduction and 13–31% TTFT** (naive full-context caching can paradoxically increase latency) [14]. ProjectDiscovery moved one dynamic field from mid-template to the end: hit rate **7% → 74%, bill −59%** [19]. The $37,901 Bedrock incident: caching configured at every layer of a 5-layer stack, broken end-to-end — 6.47B uncached input tokens; the lesson is **hard spend caps and uncached-token rate limits, not budget alerts** [20]. A healthy agentic session shape: ~97% of input tokens from cache reads — the wins are in avoiding invalidation events, not adding breakpoints.

## 4. Delegation economics, corrected

**The "Haiku is 15× cheaper" claim is stale by 3×.** Opus 5 $5/$25 vs Haiku 4.5 $1/$5 = **5×**; **Sonnet 5 vs Haiku is only 2×** — often less than a subagent's context re-bootstrap cost. (15× was true against retired Opus 4.1 $15/$75. OpenAI's spread is still 25×.) Re-run any cascade ROI model built on the old numbers, and fold in the ≥4.7 tokenizer inflation [2][11].

**What delegation actually buys is context isolation, not tier price** [9][7]: the subagent burns tens of thousands of tokens and returns a 1,000–2,000-token distillation; the parent's cached prefix just appends. The saving is *avoided permanent supervisor context growth compounded over every remaining turn* — usually larger than the tier delta. What it costs: the subagent re-bootstraps (zero cache hits from the parent, own system prompt, **5-minute TTL always**), and Haiku's 4,096-token cache floor means tiny Haiku subagents may cache nothing — small-task Haiku routing can be **net negative**.

**Fork vs subagent is the key lever** [5]: a fork inherits the parent's prefix exactly and its first request *reads the parent's cache*; a subagent doesn't. Fork for same-context parallelism (cheap); subagent for genuine isolation (pays bootstrap, buys a clean window). Multipliers to budget: agents ~4× chat; multi-agent research ~15×; Claude Code agent teams ~7× a normal session [6][7]. Tier-per-role consensus: Haiku for read-only exploration/search/test execution; Sonnet for implementation and teammates (Anthropic's own guidance); Opus/Fable for architecture and review. Router/cascade papers (RouteLLM 85% cost cut at 95% quality [24]) are benchmark ceilings; production reports cluster at 40–70%.

## 5. Context management and context rot

Anthropic's cost anchor: **~$13/dev/active-day, $150–250/dev/month, <$30/day for 90% of users** [6]. Their long-session usage diagnosis: long context re-sent every turn, cache misses after breaks, scheduled tasks and cross-session messages each sending full context, teammates, and compaction itself; `/usage` flags any behavior ≥10% of recent usage [6].

**Context rot is a gradient, not a cliff** (Chroma, 18 models): performance declines with input length even on trivial tasks, non-uniformly; distractors compound; position bias (best near the beginning, worsening with length) [15]. Circulating harder numbers ("99% below advertised windows") are not credible; the defensible claim is Anthropic's "attention budget stretched thin" framing [9]. Reduction levers in leverage order [6]: `/clear` between unrelated tasks; CLAUDE.md under ~200 lines with workflow detail moved to on-demand skills; CLIs (`gh`, `aws`) over MCP servers; hooks that filter verbose tool output ("tens of thousands of tokens to hundreds"); delegate verbose ops to subagents; cap thinking budgets; `/rewind` on wrong turns. Context editing (`clear_tool_uses`) **invalidates the prefix at the clearing point** — use it as a chunky periodic operation with `clear_at_least`, never per-turn [4].

## 6. Telemetry and self-hosted notes

**OTel GenAI semantic conventions are still "Development" — no stable release, and no cost attributes** [18]: cost must be computed downstream from token counts × your own price table, which is therefore production config to version (see the price corrections above). The practical answer for a Claude shop is **Claude Code's OTel exporter** — per-user/per-session cache-read and cache-creation tokens, works on every procurement path [6]. The live per-turn signal: `cache_creation_input_tokens` vs `cache_read_input_tokens` on each response — "if creation stays high turn after turn, something is changing in your prefix" [1].

**Self-hosting obeys the same physics**: vLLM prefix caching is exact-prefix, block-aligned (partial blocks don't hit), LRU-evicted [16]; SGLang's RadixAttention generalizes prefix sharing via a radix tree [17]. Prompt discipline designed for hosted caches transfers verbatim — and vice versa. Expect <10% hit rates on unique-file code-completion workloads.

**Local inference reality (Aug 2026)**: Mac Studio now caps at **96GB** (512GB pulled Mar 2026, 256GB by May, DRAM shortage) [22]; Kimi K2.6 needs ~600GB at Q4; K3 needs ~1.4TB with **zero public local runs** in a 544-comment HN thread [23]. The realistic local ceiling is Qwen3-Coder-Next-class (80B MoE/3B active, ~35–40GB Q4). Against the $150–250/dev/month API baseline, **local is a data-sovereignty play, not a cost play**.

## Priorities (for any agentic coding shop)

**P0**: instrument cache hit rate as an SLO (alert when read:creation drops; healthy ≈ 97% reads); hard spend caps + uncached-token rate limits (not alerts); freeze prefix ordering with all dynamic content at the tail; set TTL explicitly (`ENABLE_PROMPT_CACHING_1H=1`) and know the credit cliff; correct the price table (5×, not 15×).
**P1**: session hygiene protocol (model+effort at start, `/clear` between tasks, `/rewind` over `/compact` for abandoned paths, side questions in a second session); tier-per-role with the Haiku cache-floor caveat; fork-vs-subagent deliberately; shrink always-on context.
**P2**: cost attribution on OTel with a versioned price table; solve the worktree cache split for fleets (Agent SDK can suppress per-machine prompt sections); don't plan on local frontier inference; quote 41–80% (measured) not 85–98% (benchmark ceilings) to finance.

## Synthesis (inferred)

Applied to our setup (Claude Code subscription, devloop's subagent-heavy runs, git worktrees, mx on Neon):

1. **Our worktree habit splits the cache.** Every `EnterWorktree`/parallel-worktree flow starts cache-cold and stays segregated from the main checkout's prefix. Acceptable for isolation-critical work; wasteful for read-mostly tasks — prefer same-directory sessions or forks when the work doesn't mutate files.
2. **Devloop's economics are context-isolation economics.** Its per-task subagents each re-bootstrap on a 5-minute TTL — the win is that verbose build/test output never lands in the supervisor. Keep task prompts lean, keep the 1–2k-token return-summary discipline, and don't route tiny tasks to Haiku subagents (4,096-token cache floor + bootstrap ≈ net negative below a few thousand tokens of work).
3. **Session hygiene rules worth adopting as team practice**: pick model/effort at session start; side questions go in a second session; `/rewind` over `/compact` when abandoning an approach; expect the first turn after a long break or an upgrade to be the expensive one — batch your return.
4. **If we add usage telemetry** (the meeting's OTel-collector thread), build on the Claude Code OTel exporter + our own versioned price table — the GenAI semconv has no cost attributes and no stable release to target; watch `cache_creation` vs `cache_read` per turn as the one health signal.
5. **The local-inference question is settled for now**: nothing we can buy runs frontier-class models locally (96GB ceiling vs 600GB–1.4TB needs); revisit only if the rumored high-memory M5 Ultra actually ships (backlogged as a watch item).
