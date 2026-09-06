/**
 * `/llms-corpus-<category>.txt` — one corpus category's full text.
 *
 * Fifteen files today, one per category the pipeline publishes, generated from
 * the same page list as everything else so a new category ships its file without
 * anyone editing this route. Together with `/llms-guides.txt` they partition
 * `/llms-full.txt` exactly: every published document appears in one of them and
 * no document appears in two.
 *
 * The category is a *partial* path parameter — the emitted names are
 * `llms-corpus-theory.txt`, not `llms-corpus/theory.txt` — which keeps every
 * agent-facing text file under the single `/llms-*.txt` rule in
 * `public/_headers`.
 *
 * Prerendered, and invisible to Pagefind, which indexes HTML only.
 */

import type { APIRoute, GetStaticPaths } from 'astro';

import { buildLlmsCorpusTxt, splitDocuments } from '../loaders/lib/llms.ts';
import type { LlmsPage } from '../loaders/lib/llms.ts';
import { collectLlmsPages } from '../loaders/llms.ts';

export const prerender = true;

export const getStaticPaths: GetStaticPaths = async () => {
  const pages = await collectLlmsPages();

  return [...splitDocuments(pages).corpus.keys()].sort().map((category) => ({
    params: { category },
    props: { category, pages },
  }));
};

export const GET: APIRoute<{ category: string; pages: LlmsPage[] }> = ({ props }) =>
  new Response(buildLlmsCorpusTxt({ pages: props.pages, category: props.category }), {
    headers: {
      'Content-Type': 'text/plain; charset=utf-8',
      'Cache-Control': 'public, max-age=3600',
    },
  });
