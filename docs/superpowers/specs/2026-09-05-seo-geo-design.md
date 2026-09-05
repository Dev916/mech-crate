# mechcrate.dev SEO + GEO Layer — Design Spec

**Date:** 2026-09-05
**Status:** Approved (design approved in-session; spec pending user review)
**Repo:** mech-crate · site at `site/apps/site` (Astro 5.18 + Starlight 0.37.7, static, Cloudflare Workers static assets)

## Overview

Make mechcrate.dev visible: rank on Google and get cited by AI answer engines (ChatGPT search, Perplexity, Claude, AI Overviews). Built from a four-lens audit (live site, codebase, 2026 best-practice research, 10 peer dev-docs sites). The site's foundations are already strong — 110/110 unique titles/descriptions/canonicals, conformant llms.txt, AI crawlers verified served, brotli/HTTP2, full alt coverage — so this layer closes the four P0 holes, adds the evidence-backed P1s, and skips everything the audit's folklore list killed.

## Decisions (settled during brainstorming)

| Decision | Choice |
|---|---|
| AI crawler stance | **Allow all** (citation + training bots) — OSS-adoption goal; `Content-Signal: search=yes, ai-input=yes, ai-train=yes` (Stripe's explicit-opt-in pattern) |
| Analytics | **Cloudflare Web Analytics** — cookie-less beacon, no consent banner; referrers are the only way to observe AI citations landing |
| Scope | Full layer (P0s + P1s + cheap P2s) in one build; folklore items excluded |
| OG images | Build-time per-page via **astro-og-canvas 0.13.1** (only OG lib with an `astro ^5` peer range; runtime generation impossible on Workers — no sharp) |
| lastmod source | `researched:` frontmatter where present, else **git commit date of the source file** (verifiably accurate — Google's condition for trusting lastmod). Never build time. |
| Structured data | Scoped: TechArticle + BreadcrumbList (corpus), BreadcrumbList (docs), Organization + WebSite + SoftwareApplication (home). No FAQ/HowTo/SearchAction (deprecated/retired). |

## 1. Edge & transport

**`site/apps/site/public/robots.txt`** (new):
```
User-agent: *
Allow: /

Content-Signal: search=yes, ai-input=yes, ai-train=yes

Sitemap: https://mechcrate.dev/sitemap-index.xml
```
Cloudflare currently injects a managed robots.txt (Content-Signals preamble, zero directives) — it PREPENDS to origin files, so post-deploy verification must check the **merged live output**, and the owner checklist includes reviewing that toggle.

**`site/apps/site/public/_headers`** (new — verified supported by wrangler 4.x Workers static assets; the contrary comments in `wrangler.jsonc` and `site.yml` are wrong and get corrected):
```
/_astro/*
  Cache-Control: public, max-age=31536000, immutable
/*
  Strict-Transport-Security: max-age=31536000; includeSubDomains
  X-Content-Type-Options: nosniff
  Referrer-Policy: strict-origin-when-cross-origin
/llms.txt
  Content-Type: text/plain; charset=utf-8
  Cache-Control: public, max-age=3600
/llms-full.txt
  Content-Type: text/plain; charset=utf-8
  Cache-Control: public, max-age=3600
/404
  X-Robots-Tag: noindex
```
(Exact matcher syntax per Workers assets `_headers` rules; llms twins and section files included once they exist. HSTS ships without `preload` until the 301 has been live.)

## 2. Social cards (og:image)

- `astro-og-canvas` build-time endpoint (`src/pages/og/[...route].ts` pattern) generating 1200×630 PNGs for every docs + corpus page from frontmatter title/description; landing gets its own card.
- Template: site dark palette + raccoon mark + page title + section/category label. Reuse `src/site-meta.ts` and the existing logo asset.
- Injection: Starlight `head:` config array (docs pages) + hand-edited landing head. Absolute URLs, `og:image:width`/`og:image:height`, `og:image:alt`. `twitter:card` stays `summary_large_image` — now truthfully.

## 3. Freshness

- Add `@astrojs/sitemap` as an explicit dependency + integration (Starlight then skips its own injection) with a `serialize` hook setting `lastmod`:
  - corpus pages: `researched:` frontmatter when present;
  - all other pages (and corpus without `researched:`): last git commit date of the page's source file, resolved at build time from a git-log map keyed by source path.
- `site.yml` build/deploy jobs get `fetch-depth: 0`.
- Same date map feeds `article:published_time`/`article:modified_time` metas and JSON-LD `datePublished`/`dateModified`.

## 4. Structured data

`src/components/Head.astro` Starlight override (pattern proven by the existing `MarkdownContent` override via `Astro.locals.starlightRoute.entry`):
- **Corpus pages**: `TechArticle` — headline, description, `datePublished`/`dateModified`, `articleSection` (category), `citation[]` (the sources list), `isBasedOn` (GitHub source URL), publisher Organization. Plus `BreadcrumbList`.
- **All docs pages**: `BreadcrumbList` (Home → group → page).
- **Homepage** (inline in `index.astro`): `Organization` (MechCrate, logo, sameAs GitHub), `WebSite`, `SoftwareApplication` (mx CLI: OS, license MIT/Apache-2.0, free, codeRepository).
- JSON-LD emitted as one `application/ld+json` block per page; unit-tested shape (ajv against schema snippets), Playwright-parsed on the built site.

## 5. Agent surface

- **llms.txt**: prepend `## Instructions for LLM agents` (honest contract: install from source — no npm package; canonical commands; don't invent flags; where the corpus + .md twins live; generated-at date passed in from build) and add `## Optional` demoting the 15 per-category corpus sections per llmstxt.org semantics.
- **Markdown twins**: prerendered `[...slug].md.ts` endpoint serving each docs/corpus page's processed markdown at `<url>.md` (or `/md/<slug>` if route collision with trailing-slash pages demands it — implementer verifies; twins linked via `<link rel="alternate" type="text/markdown">` in Head).
- **Section splits**: `llms-corpus-<category>.txt` + `llms-guides.txt` generated alongside `llms-full.txt` (kept), all advertised from llms.txt. Solves the 502k-token monolith truncation.

## 6. Polish

- Corpus H1 dedup: strip the body's leading `#` heading when it duplicates (fuzzy: normalized-prefix) the frontmatter title — in `src/loaders/lib/` with tests.
- Landing head parity: `og:site_name`, `og:locale`, `twitter:title`/`twitter:description`, `<link rel="sitemap">`.
- Icon set from the logo: `favicon.ico` (multi-size), `apple-touch-icon.png` (180×180), `site.webmanifest`, `theme-color` meta. Existing `favicon.svg` stays.
- `og:type`: `website` on section/category index pages, `article` only on content pages.
- Cloudflare Web Analytics beacon `<script>` on all pages (token from owner; ships as a config value — build works with it absent).

## 7. Owner checklist (dashboard/account actions)

1. Cloudflare → SSL/TLS → **Always Use HTTPS** ON (fixes the http:// duplicate-200 P0; not fixable from the repo).
2. Cloudflare → Bot traffic → review the **managed robots.txt / Content Signals** toggle so the merged output matches intent (repo file + optional CF signals, no empty preamble).
3. Cloudflare → **Crawler Hints** + **IndexNow** ON; create a **Web Analytics** site → hand the beacon token over (repo secret or config value).
4. **Google Search Console**: DNS-TXT *domain* property for mechcrate.dev (one TXT record in Cloudflare DNS); submit `https://mechcrate.dev/sitemap-index.xml`.
5. **Bing Webmaster Tools**: import from Search Console (feeds Copilot and contributes to ChatGPT's blended index).

## 8. Verification

- **Vitest**: robots/_headers content assertions; OG endpoint route coverage (every published page has a card route); lastmod resolution (frontmatter beats git date; git date correct for a known file); JSON-LD shape per page type; llms.txt sections (Instructions + Optional present, absolute URLs); md-twin content matches page body; H1 dedup.
- **Playwright** (built site): og:image + twitter:card tags present and image URL returns 200 with image/png; exactly one H1 on corpus pages; JSON-LD parses on one page per type; beacon present when token configured.
- **Post-deploy live checks** (in the dogfood/acceptance task): merged robots.txt contains our directives + Sitemap line; `curl http://` → 301; `_headers` effects visible (immutable on /_astro, HSTS, llms charset); OG validator-equivalent fetch of a card.
- CI: all inside the existing `site.yml` build job; content-type assertion extended to run against the deployed origin after deploy (soft check step).

## Out of scope (folklore-killed by the audit)

FAQPage/HowTo schema (deprecated May-2026/Sept-2023) · WebSite SearchAction (retired) · Google Indexing API (JobPosting/BroadcastEvent only) · GA4 · keyword-density/content changes · llms.txt-as-ranking claims (900-domain log study: zero major-bot fetches — ours is for agents, not SEO) · per-page OG as a *ranking* lever (share-CTR only) · Starlight upgrade (0.37.7 supports every lever used here).
