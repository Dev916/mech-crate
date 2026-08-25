/**
 * `/llms.txt` — the LLM-facing index, per the llmstxt.org convention.
 *
 * A static endpoint rather than a build hook: the static build prerenders it
 * straight to `dist/llms.txt`, and it reads the same content collection the
 * pages do, so the index cannot drift from what actually shipped.
 *
 * Pagefind indexes HTML only, so this route is outside the search index by
 * construction — which is what the spec asks for.
 */

import type { APIRoute } from 'astro';

import { buildLlmsTxt } from '../loaders/lib/llms.ts';
import { collectLlmsPages } from '../loaders/llms.ts';

export const prerender = true;

export const GET: APIRoute = async () => {
  const body = buildLlmsTxt({ pages: await collectLlmsPages() });

  return new Response(body, {
    headers: {
      'Content-Type': 'text/plain; charset=utf-8',
      'Cache-Control': 'public, max-age=3600',
    },
  });
};
