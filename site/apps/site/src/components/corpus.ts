/**
 * Shared helpers for the corpus page chrome.
 *
 * The loader (`src/loaders/corpus.ts`) stashes every corpus field on the page's
 * `data.corpus` (schema in `src/content.config.ts`). Starlight's own route-data
 * types don't know about that extension, so `corpusMetaOf()` is the single
 * narrowing point rather than casting at each call site.
 */

import type { CollectionEntry } from 'astro:content';

/** Corpus metadata as it reaches a rendered page. Mirrors `content.config.ts`. */
export interface CorpusMeta {
  category: string;
  slug: string;
  repoPath: string;
  sourceUrl: string;
  complexity?: string;
  languages?: string[];
  useCases?: string[];
  provenance?: string;
  researched?: string;
  sources?: string[];
  inferredTitle: boolean;
  inferredCategory: boolean;
}

/** A `docs` entry that came from the corpus loader rather than an authored page. */
export type CorpusEntry = CollectionEntry<'docs'> & {
  data: { title: string; description?: string; corpus: CorpusMeta };
};

/**
 * Corpus metadata for an entry, or `undefined` for authored pages
 * (Start / Framework / AI Layer / Project) which must not get corpus chrome.
 */
export function corpusMetaOf(entry: { data?: unknown } | undefined): CorpusMeta | undefined {
  const corpus = (entry?.data as { corpus?: CorpusMeta } | undefined)?.corpus;
  return corpus && typeof corpus.category === 'string' ? corpus : undefined;
}

/** Type guard form of {@link corpusMetaOf} for filtering `getCollection()` results. */
export function isCorpusEntry(entry: CollectionEntry<'docs'>): entry is CorpusEntry {
  return corpusMetaOf(entry) !== undefined;
}

/**
 * The provenance line, e.g. `Researched 2026-08-14 · 24 sources`.
 * Returns `undefined` when the doc declares no provenance — most corpus docs
 * predate the research pipeline and simply have nothing to attest.
 */
export function provenanceSummary(corpus: CorpusMeta): string | undefined {
  if (!corpus.provenance) return undefined;

  const label = corpus.provenance.charAt(0).toUpperCase() + corpus.provenance.slice(1);
  const parts = [corpus.researched ? `${label} ${corpus.researched}` : label];

  const count = corpus.sources?.length ?? 0;
  if (count > 0) parts.push(`${count} source${count === 1 ? '' : 's'}`);

  return parts.join(' · ');
}

/** Category words that are acronyms, so title-casing would read wrong ("Ml"). */
const ACRONYMS = new Set(['ai', 'api', 'cli', 'llm', 'ml', 'rag', 'ui', 'ux']);

/** Human label for a category slug, e.g. `framework-guides` → `Framework Guides`. */
export function categoryLabel(category: string): string {
  return category
    .split('-')
    .filter((word) => word.length > 0)
    .map((word) =>
      ACRONYMS.has(word.toLowerCase())
        ? word.toUpperCase()
        : word.charAt(0).toUpperCase() + word.slice(1)
    )
    .join(' ');
}
