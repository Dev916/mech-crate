/**
 * The IO half of the LLM surfaces: read what the site publishes, hand it to the
 * pure generators in `lib/llms.ts`.
 *
 * Same split as the corpus loader — `src/loaders/lib/**` is pure and unit
 * tested, `src/loaders/*.ts` touches Astro. Both endpoints
 * (`src/pages/llms.txt.ts`, `src/pages/llms-full.txt.ts`) call this so the two
 * files always describe the same set of pages.
 */

import { getCollection } from 'astro:content';

import { LANDING_DESCRIPTION, LANDING_TITLE } from '../site-meta.ts';
import type { LlmsPage } from './lib/llms.ts';
import {
  corpusCategoryCounts,
  generatedNavPages,
  pagesFromDocsEntries,
  type DocsEntryLike,
} from './lib/llms-pages.ts';

/**
 * Every published route, normalised: the `docs` collection (authored pages plus
 * the injected corpus documents) followed by the generated navigation pages
 * that have no collection entry of their own.
 */
export async function collectLlmsPages(): Promise<LlmsPage[]> {
  const entries = (await getCollection('docs')) as unknown as DocsEntryLike[];
  const docPages = pagesFromDocsEntries(entries);

  return [
    ...docPages,
    ...generatedNavPages(
      { title: LANDING_TITLE, description: LANDING_DESCRIPTION },
      corpusCategoryCounts(docPages)
    ),
  ];
}
