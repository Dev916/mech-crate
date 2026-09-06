/**
 * `/llms-guides.txt` — the full text of the authored guides, without the corpus.
 *
 * One of the sixteen files that partition `/llms-full.txt`: this one plus the
 * fifteen `/llms-corpus-<category>.txt` files carry exactly the same documents.
 * The monolith stays published for agents that want everything; this is the half
 * that describes mx itself, small enough to fit a context window whole.
 *
 * Prerendered to `dist/llms-guides.txt`, and invisible to Pagefind, which
 * indexes HTML only.
 */

import type { APIRoute } from 'astro';

import { buildLlmsGuidesTxt } from '../loaders/lib/llms.ts';
import { collectLlmsPages } from '../loaders/llms.ts';

export const prerender = true;

export const GET: APIRoute = async () =>
  new Response(buildLlmsGuidesTxt({ pages: await collectLlmsPages() }), {
    headers: {
      'Content-Type': 'text/plain; charset=utf-8',
      'Cache-Control': 'public, max-age=3600',
    },
  });
