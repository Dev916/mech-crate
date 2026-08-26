/**
 * Generation of the two LLM-facing surfaces: `/llms.txt` and `/llms-full.txt`.
 *
 * `llms.txt` is the index — an H1, a one-line blockquote description, then one
 * `##` section per navigation group, each a bullet list of
 * `- [Title](https://mechcrate.dev/route): one-line summary`, per the
 * llmstxt.org convention. It stays small: summaries are collapsed to a single
 * line and truncated.
 *
 * `llms-full.txt` is the payload — the markdown body of every published guide
 * and corpus document, each prefaced by a separator block carrying the title,
 * the canonical URL and the repo-relative source path. Its size is unbounded by
 * design; agents want everything (see the spec's "Error handling & policies").
 *
 * Everything here is pure: the Astro endpoints in `src/pages/llms*.txt.ts` do
 * the `getCollection()` call and hand the entries in, so the whole contract —
 * grouping, URL construction, summary fallback, separator format — is unit
 * testable without a build.
 *
 * See docs/superpowers/specs/2026-08-20-mechcrate-site-design.md → "Content pipeline".
 */

import { categoryLabel } from '../../components/corpus.ts';

/** Canonical origin. Matches `site:` in astro.config.mjs. */
export const SITE_ORIGIN = 'https://mechcrate.dev';

/** Site title used as the `llms.txt` H1. */
export const SITE_TITLE = 'MechCrate';

/** One-line description used as the `llms.txt` blockquote. */
export const SITE_DESCRIPTION =
  'mx — a Rust CLI that runs a subset of your service ecosystem locally, routes every project through one Traefik instance, scaffolds services that carry their wisdom, and feeds agents a techniques corpus you host yourself.';

/** Longest a generated one-line summary may be before it is elided. */
export const SUMMARY_MAX = 200;

/**
 * Navigation group a page belongs to. Mirrors the Starlight sidebar
 * (astro.config.mjs) so the two orderings never diverge.
 */
export type SectionKind = 'overview' | 'start' | 'framework' | 'ai' | 'corpus' | 'project';

/** Section order, top to bottom. Corpus categories are interleaved by `groupPages`. */
const SECTION_ORDER: SectionKind[] = ['overview', 'start', 'framework', 'ai', 'corpus', 'project'];

const SECTION_HEADING: Record<SectionKind, string> = {
  overview: 'Overview',
  start: 'Start',
  framework: 'Framework',
  ai: 'AI Layer',
  corpus: 'Techniques Corpus',
  project: 'Project',
};

/** One published page, normalised for both generators. */
export interface LlmsPage {
  title: string;
  /** Public route with leading and trailing slash, e.g. `/docs/start/install/`. */
  route: string;
  /** Frontmatter description/summary. Absent for docs that declare none. */
  description?: string;
  /**
   * Markdown body. Absent for generated navigation pages (the landing page, the
   * corpus overview, the per-category indexes) which have no markdown source —
   * those appear in `llms.txt` only.
   */
  body?: string;
  /** Repo-relative source path, e.g. `docs/development/appendix-fsm.md`. */
  sourcePath?: string;
  kind: SectionKind;
  /**
   * Corpus category slug. Set only on corpus documents; corpus navigation pages
   * carry `kind: 'corpus'` with no category and list under "Techniques Corpus".
   */
  category?: string;
  /** Sidebar order, when the page declares one. Sorts before title. */
  order?: number;
}

export interface LlmsSection {
  heading: string;
  pages: LlmsPage[];
}

/** Absolute URL for a site route. */
export function absoluteUrl(route: string, origin: string = SITE_ORIGIN): string {
  return `${origin}${route.startsWith('/') ? route : `/${route}`}`;
}

/**
 * Public route for a `docs` collection entry id.
 *
 * Astro collapses a trailing `index` segment into its directory, so
 * `docs/start/index` and `docs/start` both serve at `/docs/start/`.
 */
export function routeFromDocId(id: string): string {
  const trimmed = id.replace(/(^|\/)index$/, '').replace(/^\/+|\/+$/g, '');
  return trimmed === '' ? '/' : `/${trimmed}/`;
}

/**
 * Which navigation group an authored `docs` entry belongs to, or `null` when the
 * entry is not a published content page (the 404 route, anything outside the
 * known groups). Corpus entries are classified by their loader metadata instead.
 */
export function classifyDocId(id: string): SectionKind | null {
  // Astro strips a trailing `index` segment, so a group's own landing page
  // arrives as `docs/ai` while its children arrive as `docs/ai/mcp-server` —
  // both belong to the same section.
  const normalized = id.replace(/^\/+|\/+$/g, '').replace(/(^|\/)index$/, '');
  if (normalized === 'docs' || normalized === '') return 'overview';

  const group = /^docs\/([^/]+)(?:\/|$)/.exec(normalized)?.[1];
  switch (group) {
    case 'corpus':
      return 'corpus';
    case 'start':
      return 'start';
    case 'framework':
      return 'framework';
    case 'ai':
      return 'ai';
    case 'project':
      return 'project';
    default:
      return null;
  }
}

/** Collapse a string to a single line of whitespace-normalised text. */
function singleLine(text: string): string {
  return text.replace(/\s+/g, ' ').trim();
}

/**
 * Truncate on a word boundary, appending an ellipsis. `llms.txt` stays small,
 * and some corpus summaries run to a full paragraph.
 */
export function truncateSummary(text: string, max: number = SUMMARY_MAX): string {
  const line = singleLine(text);
  if (line.length <= max) return line;
  const cut = line.slice(0, max);
  const boundary = cut.lastIndexOf(' ');
  return `${(boundary > max * 0.5 ? cut.slice(0, boundary) : cut).replace(/[\s,;:.\-–—]+$/, '')}…`;
}

/** Strip the inline markdown that would read as noise in a one-line summary. */
function stripInlineMarkdown(text: string): string {
  return text
    .replace(/!\[([^\]]*)\]\([^)]*\)/g, '$1')
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
    .replace(/`([^`]+)`/g, '$1')
    .replace(/\*\*([^*]+)\*\*/g, '$1')
    .replace(/(^|\s)\*([^*]+)\*/g, '$1$2')
    .replace(/(^|\s)_([^_]+)_/g, '$1$2');
}

/**
 * First prose paragraph of a markdown body — the fallback summary for a page
 * whose frontmatter declares none. Headings, fenced code, asides, JSX/HTML,
 * MDX imports, list items, tables and blockquotes are all skipped.
 */
export function firstParagraph(body: string): string | undefined {
  let inFence = false;

  for (const raw of body.split(/\r?\n/)) {
    const line = raw.trim();

    if (/^(```|~~~)/.test(line)) {
      inFence = !inFence;
      continue;
    }
    if (inFence || line === '') continue;
    if (/^(#|>|:::|\||[-*+]\s|\d+\.\s|<|import\s|export\s|---|===)/.test(line)) continue;

    const cleaned = singleLine(stripInlineMarkdown(line));
    if (cleaned !== '') return cleaned;
  }

  return undefined;
}

/**
 * The one-line summary for a page: its declared description, else the first
 * prose paragraph of its body, else nothing (the bullet then carries no note).
 */
export function summaryFor(page: LlmsPage, max: number = SUMMARY_MAX): string | undefined {
  const source = page.description?.trim() || (page.body ? firstParagraph(page.body) : undefined);
  if (!source) return undefined;
  const summary = truncateSummary(source, max);
  return summary === '' ? undefined : summary;
}

/** Sidebar-ish ordering: declared order first, then title, then route. */
function comparePages(a: LlmsPage, b: LlmsPage): number {
  const orderA = a.order ?? Number.MAX_SAFE_INTEGER;
  const orderB = b.order ?? Number.MAX_SAFE_INTEGER;
  if (orderA !== orderB) return orderA - orderB;
  const byTitle = a.title.localeCompare(b.title);
  return byTitle !== 0 ? byTitle : a.route.localeCompare(b.route);
}

/**
 * Group pages into ordered sections: Overview → Start → Framework → AI Layer →
 * Techniques Corpus (navigation pages) → one section per corpus category, by
 * label → Project. Empty sections are dropped.
 */
export function groupPages(pages: readonly LlmsPage[]): LlmsSection[] {
  const byKind = new Map<SectionKind, LlmsPage[]>();
  const byCategory = new Map<string, LlmsPage[]>();

  for (const page of pages) {
    if (page.kind === 'corpus' && page.category !== undefined) {
      const bucket = byCategory.get(page.category);
      if (bucket) bucket.push(page);
      else byCategory.set(page.category, [page]);
      continue;
    }
    const bucket = byKind.get(page.kind);
    if (bucket) bucket.push(page);
    else byKind.set(page.kind, [page]);
  }

  const sections: LlmsSection[] = [];

  for (const kind of SECTION_ORDER) {
    const bucket = byKind.get(kind);
    if (bucket && bucket.length > 0) {
      sections.push({ heading: SECTION_HEADING[kind], pages: [...bucket].sort(comparePages) });
    }

    // Corpus categories follow the corpus navigation section directly.
    if (kind !== 'corpus') continue;
    const categories = [...byCategory.entries()]
      .map(([category, docs]) => ({ label: categoryLabel(category), docs }))
      .sort((a, b) => a.label.localeCompare(b.label));
    for (const { label, docs } of categories) {
      sections.push({ heading: `Corpus: ${label}`, pages: [...docs].sort(comparePages) });
    }
  }

  return sections;
}

export interface BuildLlmsTxtOptions {
  pages: readonly LlmsPage[];
  origin?: string;
  title?: string;
  description?: string;
  summaryMax?: number;
}

/**
 * The `llms.txt` index. Every published page, grouped by navigation section,
 * one bullet each, with an absolute URL and a one-line summary.
 */
export function buildLlmsTxt(options: BuildLlmsTxtOptions): string {
  const {
    pages,
    origin = SITE_ORIGIN,
    title = SITE_TITLE,
    description = SITE_DESCRIPTION,
    summaryMax = SUMMARY_MAX,
  } = options;

  const lines: string[] = [`# ${title}`, '', `> ${singleLine(description)}`, ''];
  lines.push(
    `Every published page on ${origin.replace(/^https?:\/\//, '')}, grouped the way the site navigation groups it. ` +
      `The complete text of every guide and corpus document is at ${absoluteUrl('/llms-full.txt', origin)}.`
  );

  for (const section of groupPages(pages)) {
    lines.push('', `## ${section.heading}`, '');
    for (const page of section.pages) {
      const summary = summaryFor(page, summaryMax);
      const link = `- [${page.title}](${absoluteUrl(page.route, origin)})`;
      lines.push(summary ? `${link}: ${summary}` : link);
    }
  }

  return `${lines.join('\n')}\n`;
}

export interface DocSeparatorOptions {
  title: string;
  url: string;
  source?: string;
}

/**
 * The block that prefaces each document in `llms-full.txt`. Fenced by `---`
 * rules so a reader (human or model) can find document boundaries by scanning,
 * and carrying the canonical URL plus the repo path the text was published from.
 */
export function docSeparator({ title, url, source }: DocSeparatorOptions): string {
  const lines = ['---', `# ${title}`, `URL: ${url}`];
  if (source) lines.push(`Source: ${source}`);
  lines.push('---');
  return `\n\n${lines.join('\n')}\n\n`;
}

export interface BuildLlmsFullTxtOptions {
  pages: readonly LlmsPage[];
  origin?: string;
  title?: string;
  description?: string;
}

/**
 * The `llms-full.txt` payload: the markdown body of every published guide and
 * corpus document, in navigation order, each behind a separator block.
 *
 * Pages without a markdown body (the landing page and the generated corpus
 * navigation pages) are indexed in `llms.txt` but have no text to concatenate,
 * so they are skipped here.
 */
export function buildLlmsFullTxt(options: BuildLlmsFullTxtOptions): string {
  const {
    pages,
    origin = SITE_ORIGIN,
    title = SITE_TITLE,
    description = SITE_DESCRIPTION,
  } = options;

  const documents = groupPages(pages).flatMap((section) =>
    section.pages.filter((page) => (page.body ?? '').trim() !== '')
  );

  const header = [
    `# ${title} — full text`,
    '',
    `> ${singleLine(description)}`,
    '',
    `The complete text of ${documents.length} published document${documents.length === 1 ? '' : 's'} ` +
      `from ${origin.replace(/^https?:\/\//, '')}, in site navigation order. ` +
      `Each document is prefaced by a separator carrying its title, canonical URL and repository source path. ` +
      `The index is at ${absoluteUrl('/llms.txt', origin)}.`,
  ].join('\n');

  const body = documents
    .map(
      (page) =>
        docSeparator({
          title: page.title,
          url: absoluteUrl(page.route, origin),
          source: page.sourcePath,
        }) + (page.body ?? '').trim()
    )
    .join('');

  return `${header}${body}\n`;
}
