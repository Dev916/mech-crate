# Source Providers — Registry & Contract

Phase 2 of technique-research runs every **active** provider whose **Use when** matches the topic. To add a provider: append an entry satisfying the contract, flip Status to active. Log new provider ideas as GitHub issues on Dev916/mech-crate.

## Contract

Every entry defines:
- **Status:** active | planned
- **Use when:** topic conditions that make this provider worth querying
- **Query:** the exact tool/command/skill an agent runs
- **Returns:** claims with citations (URL or source id), each tagged with confidence (high/medium/low)
- **Cost note:** token/API cost characteristics

## Providers

### web
- **Status:** active
- **Use when:** always (default provider)
- **Query:** dispatch ONE research subagent (Agent tool) instructed to: run 8-15 varied WebSearch queries, WebFetch the strongest primary sources, corroborate each key claim with 2+ independent sources or an authoritative primary, and return numbered claims with citation URLs + HIGH/MEDIUM/LOW confidence tags plus a recommendations section. (The `deep-research` skill is user-invocable only — `disable-model-invocation` — so agents cannot call it; if the USER launches the run via /deep-research, use that instead.)
- **Returns:** cited, confidence-tagged claims from web sources (docs, papers, engineering blogs)
- **Cost note:** token-heavy (~150k subagent tokens); at most one invocation per run

### x
- **Status:** active
- **Use when:** innovation/project-discovery topics — new tools, emerging patterns, "what are practitioners adopting"; also the tech-radar sweep
- **Query:** `mcp__x__search_recent` with 2-3 topic keyword variants (e.g. `<topic> -is:retweet lang:en`); pull threads from high-signal hits via `mcp__x__get_user`/timeline when an author looks authoritative
- **Returns:** discovery-grade claims cited as tweet/thread URLs, confidence LOW-MEDIUM — must be corroborated by a primary source (repo, docs, post) or placed under Synthesis (inferred)
- **Cost note:** cheap API calls; recent-search window ~7 days, so it finds what's NEW, not what's established

### hackernews
- **Status:** active
- **Use when:** innovation/project-discovery and "how do experienced engineers argue about this" topics; also the tech-radar sweep
- **Query:** no-auth Algolia HN API via curl: `curl -s "https://hn.algolia.com/api/v1/search?query=<url-encoded topic>&tags=story&hitsPerPage=15"` (add `&numericFilters=created_at_i><epoch>` for freshness); fetch linked articles for top relevant hits, and comment threads via `tags=comment` when the discussion itself is the signal
- **Returns:** story/article claims cited by URL (+ HN discussion link), confidence MEDIUM for linked primary sources, LOW for comment claims — comment-only claims must be corroborated or marked inferred
- **Cost note:** free, no auth, fast; rank by points/num_comments for signal

### context7
- **Status:** planned
- **Use when:** topic names a specific library/framework/SDK
- **Query:** `mcp__plugin_context7_context7__resolve-library-id` then `query-docs` for current API documentation
- **Returns:** official-doc claims, citation = library id + doc section, confidence high
- **Cost note:** cheap per query

### cross-model
- **Status:** planned (needs API key config — see GitHub issue)
- **Use when:** contested/judgment-heavy topics where a second model's blind spots differ
- **Query:** OpenAI-compatible chat call (key/base_url from env) asking the model for its approach + reasoning
- **Returns:** claims cited as "model consultation: <model>", confidence medium; must be corroborated or marked inferred
- **Cost note:** one API call per consulted model

### hq-corpus
- **Status:** planned
- **Use when:** topic may overlap internal knowledge (business/ops/prior research)
- **Query:** `mcp__hq__hq_corpus_search` with the topic
- **Returns:** internal-doc claims, citation = corpus doc path/id
- **Cost note:** cheap

### reddit
- **Status:** planned (needs OAuth app registration — see GitHub issue)
- **Use when:** practitioner-experience and tooling-adoption topics (r/rust, r/programming, r/ExperiencedDevs, topic-specific subs)
- **Query:** Reddit API search (OAuth client credentials from env) across relevant subreddits, sorted by top/relevance; fetch high-score threads
- **Returns:** discovery-grade claims cited as thread URLs, confidence LOW-MEDIUM — corroborate with primary sources or mark inferred
- **Cost note:** free tier rate limits; requires registered app credentials

### rss
- **Status:** planned (see GitHub issue)
- **Use when:** freshness-driven topics; curated feed list TBD in the issue
- **Query:** fetch + parse configured feeds, filter by topic keywords
- **Returns:** article claims with URLs, confidence per source reputation
- **Cost note:** cheap fetches, noisy signal

### medium-api
- **Status:** planned (see GitHub issue — https://mediumapi.com/)
- **Use when:** practitioner-experience topics where engineering blogs dominate
- **Query:** unofficial Medium API (key TBD) topic/tag search, fetch top articles
- **Returns:** article claims with URLs, confidence medium
- **Cost note:** third-party API key + rate limits
