/**
 * `/llms.txt` — the LLM-facing index, per the llmstxt.org convention.
 *
 * A static endpoint rather than a build hook: the static build prerenders it
 * straight to `dist/llms.txt`, and it reads the same content collection the
 * pages do, so the index cannot drift from what actually shipped.
 *
 * Pagefind indexes HTML only, so this route is outside the search index by
 * construction — which is what the spec asks for.
 *
 * The "generated as of" line in the agent instructions is the newest date in the
 * build's page-date map — `researched:` frontmatter or a source file's last
 * commit, resolved by `src/loaders/lib/dates.ts`. Never `Date.now()`: a
 * build-stamped date would move on every rebuild of an unchanged tree, tell a
 * reader nothing about the content, and make `dist/` non-reproducible. Reached
 * through `src/lib/build-dates.ts`, which throws under `astro build` (a build
 * that cannot read git history is a bug) and returns `undefined` under
 * `astro dev` (the containerised dev loop mounts no `.git`) — where the line is
 * simply omitted.
 */

import type { APIRoute } from 'astro';

import { buildPageDates } from '../lib/build-dates.ts';
import { buildLlmsTxt } from '../loaders/lib/llms.ts';
import { collectLlmsPages } from '../loaders/llms.ts';

export const prerender = true;

/** Newest date in the page-date map — the last time any published page changed. */
function newestDate(dates: ReadonlyMap<string, string> | undefined): string | undefined {
  if (dates === undefined || dates.size === 0) return undefined;
  // ISO-8601 UTC timestamps sort lexicographically, which is why the map stores
  // them normalised rather than as `Date`s.
  const newest = [...dates.values()].reduce((a, b) => (a > b ? a : b));
  return newest.slice(0, 'YYYY-MM-DD'.length);
}

export const GET: APIRoute = async () => {
  const generatedAt = newestDate(await buildPageDates());

  const body = buildLlmsTxt({
    pages: await collectLlmsPages(),
    ...(generatedAt === undefined ? {} : { generatedAt }),
  });

  return new Response(body, {
    headers: {
      'Content-Type': 'text/plain; charset=utf-8',
      'Cache-Control': 'public, max-age=3600',
    },
  });
};
