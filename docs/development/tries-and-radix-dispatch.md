---
title: "Tries, Radix Trees, and Trie-Path Dispatch: Replacing Conditional Chains with Structural Lookup"
category: patterns
languages: [javascript, rust, go, sql]
complexity: advanced
use_cases:
  - deciding when a trie/radix structure beats a hash map, B-tree, or switch statement
  - replacing sprawling if/else chains with longest-prefix-match dispatch
  - designing hierarchical configuration with most-specific-wins and ancestor fallback
  - choosing between nearest-wins and merge-along-path config resolution
summary: Trie/radix fundamentals with the ART paper's actual numbers, the production deployments (HTTP routers, kernel FIB, DuckDB/Redis, MQTT, Lucene), and the trie-path dispatch pattern — most-specific-wins resolution with ancestor fallback — documented via convergent evidence (CSS cascade, nginx, gitignore) with forst as the worked example.
provenance: researched
researched: 2026-07-28
sources:
  - https://db.in.tum.de/~leis/papers/ART.pdf
  - https://github.com/julienschmidt/httprouter
  - https://github.com/ibraheemdev/matchit
  - https://docs.kernel.org/networking/fib_trie.html
  - https://duckdb.org/2022/07/27/art-storage
  - https://github.com/antirez/rax
  - https://github.com/surrealdb/vart
  - https://dotat.at/prog/qp/README.html
  - https://en.wikipedia.org/wiki/Hash_array_mapped_trie
  - https://blog.mikemccandless.com/2010/12/using-finite-state-transducers-in.html
  - https://www.hivemq.com/blog/mqtt-topic-tree-matching-challenges-best-practices-explained/
  - https://ethereum.org/developers/docs/data-structures-and-encoding/patricia-merkle-trie/
  - https://www.w3.org/TR/css-cascade-4/
  - https://git-scm.com/docs/gitignore
  - https://docs.editorconfig.org/en/master/editorconfig-format.html
  - https://github.com/cosmiconfig/cosmiconfig
  - https://docs.spring.io/spring-boot/reference/features/external-config.html
  - https://launchdarkly.com/docs/home/flags/target-rules
  - https://github.com/web-mech/forst
---

# Tries, Radix Trees, and Trie-Path Dispatch

Three layers: the data structure (when it wins), the production record (who ships it and why), and the pattern (turning conditional sprawl into structural lookup — with [forst](https://github.com/web-mech/forst) as the worked example). Inline `[n]` cites key to `sources`.

## 1. Fundamentals — what the numbers actually say

A radix tree's height depends on key length k, not element count n — O(k) lookups vs O(k·log n) for comparison trees, no rebalancing, keys kept in lexicographic order [1]. The span (bits consumed per level) is the tuning knob: height falls linearly with span while node space grows exponentially.

**ART (Adaptive Radix Tree, Leis 2013)** resolves that tension with fixed 8-bit span and *adaptive node sizes* — Node4/16/48/256, grown in place, each with its own search strategy (linear scan → SSE parallel compare → index array → direct 256-slot lookup) [1]. The paper's measured story, worth internalizing rather than the folklore version:
- ART ≈ ties a chained hash table on lookups (both far ahead of FAST/CSB+/red-black); the hash table needs ~4× fewer instructions (26 vs ~100), ART wins on cache misses and branch prediction for *dense* keys.
- Skew (Zipf) widens ART's lead; **scarce cache inverts it** — shrinking effective cache 64× cut ART to ⅓ throughput while the hash table barely moved. "Tries are cache-friendly" is conditional, not axiomatic.
- Space: 8.1–52 bytes/key with path compression + lazy expansion (5× height reduction on long string keys); the Linux-kernel-style uncompressed radix tree is the 2048-bytes/key cautionary tale in the same table.
- **The decisive argument vs hash indexes isn't speed — it's order**: range scans, prefix enumeration, min/max come free; hash tables give you none of it [1]. Non-string keys need binary-comparable transforms (sign-bit flips, float rank transforms, collation keys) — a real correctness tax.

Relatives, one line each: **crit-bit/PATRICIA** = binary path-compressed endpoint; **qp-tries** widen to 4-5 bits with HAMT-style popcount bitmaps (~⅓ less memory, ~30% faster than crit-bit, explicit child prefetching) [8]; **HAMT** applies trie machinery to *hashed* keys — the backbone of Clojure/Scala immutable maps, structural sharing gives O(nodes-per-level) writes, but deliberately discards ordering [9]; **FSTs** compress suffixes too but are build-once (Lucene's terms index: 9.8M terms → 69MB FST) [10].

## 2. The production record

| System | Structure | Why |
|---|---|---|
| Go httprouter/gin, Rust matchit (axum) | radix trie per HTTP method | URL paths = hierarchical, small alphabet; zero-allocation matching; matchit benches ~2.4µs vs ~422µs for regex scans (~170×) [2][3]. Match priority = most-specific-first; ambiguous routes rejected **at insert time** |
| Linux IPv4 FIB | LC-trie (level + path compression, dynamic inflate/halve) | longest-prefix match at line rate; skipped bits mean the leaf key must be re-verified — the optimistic path-compression obligation [4] |
| DuckDB, HyPer | ART | PK/FK/UNIQUE enforcement + selective range queries; persistence cut index reload 7.75s → 0.06s [5] |
| Redis | rax radix tree | cluster slot tracking, Streams IDs. antirez's caveat is load-bearing: a robust radix tree is *hard* — "a lot of things can go wrong in node splitting, merging, and various edge cases" (he later found a read overflow in his own) [6] |
| SurrealDB | VART (versioned ART) | snapshot isolation via copy-on-write structural sharing [7] |
| MQTT brokers | topic tree | wildcard subscription matching without scanning millions of subscriptions; shared topic levels stored once [11] |
| Ethereum | Merkle-Patricia trie | same family + cryptographic overlay; extension nodes ARE path compression; every state change re-roots the hash [12] |
| Telecom rating | E.164 prefix trie | longest matching phone prefix determines rate/carrier |

Common thread: **keys that are paths over a small alphabet with heavy shared prefixes**. That's the single strongest predictor a trie is the right call.

## 3. The pattern: trie-path dispatch (if/else chains → structural lookup)

There is no canonical paper for this — the refactoring catalog has no "Replace Conditional with Dispatch Table" entry (verified). What exists is **convergent evolution**: independent systems repeatedly arriving at the same shape. The structural preconditions, distilled from all of them:

1. The condition space is a **path** over bounded segments (env/tenant/feature, URL, topic, prefix).
2. Many conditions **share prefixes**.
3. Resolution is **most-specific-wins**.
4. Unmatched paths need **ancestor or default fallback**.

When all four hold, the dispatch table's shape matches the domain's shape — and three properties become *structural* instead of enforced-by-code-review: **order independence** (like IP routing: match length decides, not declaration order), **fallback** (walk up the trie), and **insert-time ambiguity detection** (like matchit rejecting conflicting routes).

The exhibits and their exact tiebreak rules — worth studying because every hand-rolled resolver gets some of this wrong:
- **CSS cascade** [13]: origin/importance → context → specificity → source order; and a fully-specified fallback chain (inherited properties take the parent's computed value, non-inherited take initial values). Design lesson: *inherit-from-ancestor*, *fall-back-to-default*, and *roll-back-a-layer* are three distinct concepts.
- **nginx location matching**: exact match short-circuits → longest prefix remembered (by length, not config order) → regex tier in config order → fall back to the remembered prefix. The honest hybrid: LPM for the structured majority, an ordered list for the irregular tail, one keyword (`^~`) to stop the tail overriding the structure.
- **.gitignore** [14]: nearest file wins, then last-pattern-wins within a file — two stacked tiebreaks, named explicitly.
- **cosmiconfig vs rc** [16]: the ecosystem's fork in the road — stop-at-nearest vs merge-up-the-tree. Cosmiconfig documents choosing nearest-wins *deliberately*.
- **Counter-examples that prove the boundary**: Spring Boot resolves by *source rank* (flat ordered list), not path specificity [17]; LaunchDarkly targeting rules are *manually ordered* because "this rule beats that one" is author intent, not structure [18]. When ordering IS the semantics, use an ordered rule list.

**When NOT to use trie-path dispatch**: flat unrelated conditions (no shared prefixes → a slow hash map); few branches (a 5-arm switch beats everything on speed and readability — a hash lookup is ~26 instructions, ART ~100 [1]); predicates that aren't prefixes (ranges, regex, arbitrary booleans — do what nginx does); ordering-as-intent (above); and hand-rolling when correctness matters — use matchit/rax/libart/vart before writing your own [6].

## 4. Worked example: forst

[forst](https://github.com/web-mech/forst) (`npm i forst`) is a radix-trie hierarchical configuration resolver where **the filesystem is the trie**: directories are inner nodes, `<path>.json` files are values at nodes. Two operations, ~90 lines total:

- `forst('test/foo', './conf')` — **lookup with ancestor fallback**: if `conf/test/foo.json` doesn't exist, strip the last segment and retry (`conf/test.json`), recursively to the root; missing everything yields `{}`. This is longest-prefix-match resolution over config paths — the IP-routing/CSS-inheritance semantic, implemented in one recursive function.
- `forst(['test', 'test/foo'], './conf')` — **merge-along-path**: resolve each path (each with its own fallback) and deep-merge left→right, so later/more-specific paths override earlier/base ones per-property. This is the `rc`/EditorConfig school (merge, not replace) — and note it composes BOTH resolution schools: nearest-wins fallback *within* a path, merge *across* the path list.
- `forstMap({db: 'test', api: ['test','test/bar']})` — batch expansion of named lookups.

What it replaces: the `if (env === 'prod' && region === 'eu' && tenant === 'acme') {...} else if (env === 'prod' && region === 'eu') {...} else if (env === 'prod') {...}` pyramid becomes a directory tree + one lookup — adding a context is adding a file, not editing branch logic. Every property from §3's checklist holds: paths over bounded segments (env/region/tenant), shared prefixes, most-specific-wins (deep-merge order), ancestor fallback (the recursive parent walk).

Use-cases this fits: environment/tenant/feature-scoped app config; per-context defaults with overrides (pricing tiers, regional settings, white-label theming); any "settings cascade" where contexts nest. Boundaries per §3: don't route non-hierarchical conditions through it, and keep an escape hatch for the irregular tail.

## Synthesis (inferred)

- **The forst pattern, generalized**: "filesystem as radix trie" is a deliberate architectural trade — you get zero data-structure code (the OS maintains the trie), human-inspectable state (`tree conf/`), and git-diffable dispatch tables, at the cost of per-lookup IO. The same resolution semantics can be lifted to an in-memory trie (or a flat map keyed by full paths with computed fallback chains — at config-scale, n is tiny and the *semantics*, not the structure, are the point). This mirrors claim-of-record from §1: below serious scale, choose the trie for its *semantics* (fallback, specificity, order-independence), never for speed.
- **A resolver designed today should answer four questions explicitly**, learned from the exhibits' divergence: (1) nearest-wins or merge-along-path? (2) per-property or whole-file override? (3) what are the stacked tiebreaks, in order? (4) is there an escape hatch for non-hierarchical rules? forst answers: merge-along-path, per-property, path-order-then-specificity, none.
- **forst modernization notes** (if reviving the package): `fs.exists` is deprecated (race-prone check-then-read — read and catch `ENOENT` instead); lookups are sequential awaits (batch with `Promise.all` across `forstMap` keys); add memoization for hot paths (config rarely changes mid-process); and consider publishing the *pattern* (path-fallback + merge resolution over any KV backing) with the fs implementation as one adapter — the semantics generalize to S3 prefixes, Consul KV, or an in-memory map unchanged.
