/**
 * `<page-url>.md` — one markdown twin per published page with a markdown source.
 *
 * `/docs/start/install/` serves HTML from `dist/docs/start/install/index.html`;
 * this route serves the same document's markdown from
 * `dist/docs/start/install.md`. Sibling names on disk, so nothing collides — the
 * `/md/<slug>` fallback the spec allowed for is unnecessary.
 *
 * The IO half; the predicate and the file's shape live in `src/lib/md-twin.ts`.
 * Prerendered, like every other route on this site: the deploy is an assets-only
 * Worker, so a twin is a file, not a handler. The page list is
 * `collectLlmsPages()` — the same function `/llms.txt`, `/llms-full.txt` and the
 * social cards use — so the set of twins is by construction the set of documents
 * the site says it publishes.
 *
 * Endpoints are not pages: these routes stay out of the sitemap, out of the
 * page-date gate in `src/loaders/lib/dates.ts`, and out of the Pagefind index,
 * which reads HTML only.
 *
 * See docs/superpowers/specs/2026-09-05-seo-geo-design.md → "5. Agent surface".
 */

import type { APIRoute, GetStaticPaths } from 'astro';

import { MD_TWIN_TYPE, buildMdTwin, mdTwins } from '../lib/md-twin.ts';
import type { LlmsPage } from '../loaders/lib/llms.ts';
import { collectLlmsPages } from '../loaders/llms.ts';

export const prerender = true;

export const getStaticPaths: GetStaticPaths = async () =>
  [...mdTwins(await collectLlmsPages())].map(([slug, page]) => ({
    params: { slug },
    props: { page },
  }));

export const GET: APIRoute<{ page: LlmsPage }> = ({ props }) =>
  new Response(buildMdTwin(props.page), {
    headers: {
      // Advisory only for a prerendered file — Cloudflare's asset server derives
      // the type from the `.md` extension, and `public/_headers` pins it. This
      // is what `astro dev` and `astro preview` serve, so the dev loop and the
      // edge agree.
      'Content-Type': `${MD_TWIN_TYPE}; charset=utf-8`,
      'Cache-Control': 'public, max-age=3600',
    },
  });
