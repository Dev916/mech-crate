/**
 * `/llms-full.txt` — the full markdown text of every published guide and corpus
 * document, concatenated behind separator blocks.
 *
 * Size is unbounded by design (agents want everything); `llms.txt` is the small
 * indexed companion. Like that file, this is a static endpoint prerendered to
 * `dist/llms-full.txt` and invisible to Pagefind, which indexes HTML only.
 */

import type { APIRoute } from 'astro';

import { buildLlmsFullTxt } from '../loaders/lib/llms.ts';
import { collectLlmsPages } from '../loaders/llms.ts';

export const prerender = true;

export const GET: APIRoute = async () => {
  const body = buildLlmsFullTxt({ pages: await collectLlmsPages() });

  return new Response(body, {
    headers: {
      'Content-Type': 'text/plain; charset=utf-8',
      'Cache-Control': 'public, max-age=3600',
    },
  });
};
