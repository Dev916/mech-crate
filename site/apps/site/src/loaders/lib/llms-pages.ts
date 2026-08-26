/**
 * Turning what the site actually publishes into the flat page list the
 * `llms.txt` / `llms-full.txt` generators consume.
 *
 * Two inputs, because the site publishes two kinds of route:
 *   - `docs` collection entries — authored pages (Start / Framework / AI Layer /
 *     Project) and the injected corpus documents. Carry a markdown body, so they
 *     appear in both files.
 *   - generated Astro pages — the landing page, the corpus overview and the
 *     per-category indexes. Navigation with no markdown source, so they are
 *     indexed in `llms.txt` and skipped by `llms-full.txt`.
 *
 * Pure: `astro:content` is imported for types only (erased at transform), so
 * this is unit tested without a build. The endpoints supply the entries.
 */

import { categoryLabel, corpusMetaOf } from '../../components/corpus.ts';
import type { LlmsPage } from './llms.ts';
import { classifyDocId, routeFromDocId } from './llms.ts';

/**
 * Repo-relative path of this Astro app. Authored pages report a project-relative
 * `filePath` (`src/content/docs/…`); `llms-full.txt` quotes repo-relative paths
 * so a reader can find the file in the repository.
 */
export const APP_REPO_PREFIX = 'site/apps/site';

/** The shape of a `docs` collection entry that these generators depend on. */
export interface DocsEntryLike {
  id: string;
  body?: string;
  filePath?: string;
  data: {
    title: string;
    description?: string;
    sidebar?: { order?: number; hidden?: boolean };
    corpus?: unknown;
  };
}

/**
 * Normalise `docs` collection entries into pages.
 *
 * Entries that are not published content pages — the 404 route, anything
 * outside the known navigation groups, anything hidden from the sidebar — are
 * dropped, so `llms.txt` never advertises a route the navigation does not.
 */
export function pagesFromDocsEntries(entries: readonly DocsEntryLike[]): LlmsPage[] {
  const pages: LlmsPage[] = [];

  for (const entry of entries) {
    if (entry.data.sidebar?.hidden === true) continue;

    const corpus = corpusMetaOf(entry);
    const kind = corpus ? 'corpus' : classifyDocId(entry.id);
    if (kind === null) continue;

    const sourcePath = corpus?.repoPath ?? repoPathFor(entry.filePath);

    pages.push({
      title: entry.data.title,
      route: routeFromDocId(entry.id),
      ...(entry.data.description === undefined ? {} : { description: entry.data.description }),
      ...(entry.body === undefined ? {} : { body: entry.body }),
      ...(sourcePath === undefined ? {} : { sourcePath }),
      kind,
      ...(corpus === undefined ? {} : { category: corpus.category }),
      ...(entry.data.sidebar?.order === undefined ? {} : { order: entry.data.sidebar.order }),
    });
  }

  return pages;
}

/** `src/content/docs/…` → `site/apps/site/src/content/docs/…`. */
function repoPathFor(filePath: string | undefined): string | undefined {
  if (!filePath) return undefined;
  const normalized = filePath.replace(/\\/g, '/').replace(/^\.\//, '').replace(/^\/+/, '');
  return normalized.startsWith(`${APP_REPO_PREFIX}/`)
    ? normalized
    : `${APP_REPO_PREFIX}/${normalized}`;
}

export interface LandingPageInput {
  title: string;
  description: string;
}

/**
 * The generated navigation routes: the landing page, `/docs/corpus/`, and one
 * index per corpus category. Titles and descriptions mirror what those pages
 * actually render (src/pages/index.astro, src/pages/docs/corpus/**) so the index
 * does not describe them differently from the site.
 *
 * `counts` maps a category slug to how many corpus documents it holds — the
 * per-category index pages state that number, so the summaries state it too.
 */
export function generatedNavPages(
  landing: LandingPageInput,
  counts: ReadonlyMap<string, number>
): LlmsPage[] {
  const pages: LlmsPage[] = [
    {
      title: landing.title,
      route: '/',
      description: landing.description,
      kind: 'overview',
      order: -1,
    },
    {
      title: 'Techniques Corpus',
      route: '/docs/corpus/',
      description: `Every publishable document in mx's agent corpus, grouped by category. Agents retrieve these pages via the MCP \`rag_context\` tool.`,
      kind: 'corpus',
      order: -1,
    },
  ];

  const categories = [...counts.entries()]
    .map(([category, count]) => ({ category, count, label: categoryLabel(category) }))
    .sort((a, b) => a.label.localeCompare(b.label));

  for (const { category, count, label } of categories) {
    pages.push({
      title: `${label} — Techniques Corpus`,
      route: `/docs/corpus/${category}/`,
      description: `The ${count} ${label.toLowerCase()} document${count === 1 ? '' : 's'} in mx's agent corpus.`,
      kind: 'corpus',
      order: 0,
    });
  }

  return pages;
}

/** How many corpus documents each category holds, keyed by category slug. */
export function corpusCategoryCounts(pages: readonly LlmsPage[]): Map<string, number> {
  const counts = new Map<string, number>();
  for (const page of pages) {
    if (page.kind !== 'corpus' || page.category === undefined) continue;
    counts.set(page.category, (counts.get(page.category) ?? 0) + 1);
  }
  return counts;
}
