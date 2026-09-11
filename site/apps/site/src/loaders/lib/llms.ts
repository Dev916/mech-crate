/**
 * Generation of the LLM-facing surfaces: `/llms.txt`, `/llms-full.txt`, and the
 * per-section splits `/llms-guides.txt` + `/llms-corpus-<category>.txt`.
 *
 * `llms.txt` is the index — an H1, a one-line blockquote description, an
 * `## Instructions for LLM agents` contract, then one `##` section per
 * navigation group, each a bullet list of
 * `- [Title](https://mechcrate.dev/route): one-line summary`, per the
 * llmstxt.org convention. It stays small: summaries are collapsed to a single
 * line and truncated. The 68 corpus documents live under a single `## Optional`
 * heading at the end — llmstxt.org reserves that exact H2 for "secondary URLs a
 * reader may skip for a shorter context", which is precisely what the corpus is
 * relative to the authored guides.
 *
 * `llms-full.txt` is the payload — the markdown body of every published guide
 * and corpus document, each prefaced by a separator block carrying the title,
 * the canonical URL and the repo-relative source path. Its size is unbounded by
 * design; agents want everything (see the spec's "Error handling & policies").
 * At ~500k tokens it also truncates in most context windows, which is why the
 * same documents are additionally published as sixteen smaller files: one for
 * the authored guides, one per corpus category. Those splits partition
 * `llms-full.txt` exactly — every document appears in exactly one of them.
 *
 * Everything here is pure: the Astro endpoints in `src/pages/llms*.txt.ts` do
 * the `getCollection()` call and hand the entries in, so the whole contract —
 * grouping, URL construction, summary fallback, separator format — is unit
 * testable without a build.
 *
 * See docs/superpowers/specs/2026-08-20-mechcrate-site-design.md → "Content pipeline"
 * and docs/superpowers/specs/2026-09-05-seo-geo-design.md → "5. Agent surface".
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

/** The repository mx is installed from. There is no package registry entry. */
export const REPO_URL = 'https://github.com/Dev916/mech-crate';

/**
 * The only supported install path today, quoted verbatim from
 * `/docs/start/install/`. Stated as one line so a model can copy it.
 */
export const INSTALL_COMMAND = `git clone ${REPO_URL}.git && cd mech-crate && make install-local`;

/**
 * H2 that opens the agent contract. Not a link list, so it sits before every
 * llmstxt.org file-list section rather than among them.
 */
export const INSTRUCTIONS_HEADING = 'Instructions for LLM agents';

/**
 * H2 that carries the corpus. llmstxt.org gives this exact spelling a defined
 * meaning — "secondary URLs, which can be skipped for a shorter context" — so
 * the heading is a constant rather than prose, and it is always last.
 */
export const OPTIONAL_HEADING = 'Optional';

/** `/llms-full.txt` — every published document, one file. */
export const LLMS_FULL_PATH = '/llms-full.txt';

/** `/llms-guides.txt` — the authored guides only. */
export const LLMS_GUIDES_PATH = '/llms-guides.txt';

/** `/llms-corpus-<category>.txt` — one corpus category's documents. */
export function llmsCorpusPath(category: string): string {
  return `/llms-corpus-${category}.txt`;
}

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
  /**
   * Corpus category slug, set only on the per-category sections. What tells
   * `buildLlmsTxt` which sections belong under `## Optional` — a string test on
   * the heading would break the moment a category is renamed.
   */
  category?: string;
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
      .map(([category, docs]) => ({ category, label: categoryLabel(category), docs }))
      .sort((a, b) => a.label.localeCompare(b.label));
    for (const { category, label, docs } of categories) {
      sections.push({
        heading: `Corpus: ${label}`,
        pages: [...docs].sort(comparePages),
        category,
      });
    }
  }

  return sections;
}

export interface AgentInstructionsOptions {
  origin?: string;
  /** Corpus category slugs that have a split file, in the order to advertise them. */
  categories?: readonly string[];
  /**
   * Newest commit date behind the published content, ISO-8601. Comes from the
   * build's git-derived page-date map — never `Date.now()`, which would make the
   * file non-reproducible and would date a *build* rather than the content.
   * Omitted (and the line dropped) where git is unreadable, i.e. the dev
   * container.
   */
  generatedAt?: string;
}

/**
 * The `## Instructions for LLM agents` body: a short contract with a model that
 * has this file as its entry point.
 *
 * Every claim here is checkable against the site, and the awkward ones are
 * stated rather than smoothed over — a model that installs mx from a package
 * registry that does not exist, or that invents a plausible flag, wastes a user's
 * afternoon. Honesty about `mx upgrade` is deliberate for the same reason.
 */
export function agentInstructions(options: AgentInstructionsOptions = {}): string[] {
  const { origin = SITE_ORIGIN, categories = [], generatedAt } = options;
  const url = (route: string) => absoluteUrl(route, origin);

  const retrieval = [
    `${url(LLMS_FULL_PATH)} (everything, ~500k tokens — expect truncation)`,
    `${url(LLMS_GUIDES_PATH)} (the authored guides only)`,
    ...categories.map((category) => url(llmsCorpusPath(category))),
  ];

  const lines = [
    'Read this before acting on anything below. Each point is a fact about what mx does today, not a summary of intent.',
    '',
    `- **Install from source.** mx is not published to npm, cargo or Homebrew — there is no package to install, stale or otherwise. Build it from the repository: \`${INSTALL_COMMAND}\`, which puts a release binary in \`~/.local/bin\`. See ${url('/docs/start/install/')}.`,
    `- **Do not invent flags.** Every \`mx\` and \`make\` verb, with its real flags, is listed at ${url('/docs/start/cli-reference/')}, taken from the shipped \`--help\` output. A flag that is not on that page does not exist; do not carry one over from a similar tool.`,
    `- **\`mx upgrade\` is mid-repair.** On the current build it fails before doing anything, so do not plan work around it. What it is meant to do, and where it stops, is at ${url('/docs/framework/upgrade/')}; every open defect with a red test behind it is at ${url('/docs/project/known-broken/')}.`,
    `- **Every page has a markdown twin.** Append \`.md\` to a page URL for its source markdown without the HTML chrome — ${url('/docs/start/install/')} is also ${url('/docs/start/install.md')}. Each page advertises its own twin as \`<link rel="alternate" type="text/markdown">\`.`,
    `- **Retrieve in bulk instead of crawling.** ${retrieval.length} concatenated files carry the same text as the pages:`,
    ...retrieval.map((file) => `  - ${file}`),
    `- **Running inside mx, query the corpus instead of fetching it.** The MCP server exposes the same documents through its \`rag_context\` tool — ${url('/docs/ai/mcp-server/')}.`,
  ];

  if (generatedAt !== undefined) {
    lines.push(
      `- **Freshness.** Generated from the repository as of ${generatedAt}. Per-page dates are the \`<lastmod>\` values in ${url('/sitemap-index.xml')}.`
    );
  }

  return lines;
}

export interface BuildLlmsTxtOptions {
  pages: readonly LlmsPage[];
  origin?: string;
  title?: string;
  description?: string;
  summaryMax?: number;
  /** Newest content commit date, ISO-8601. See {@link AgentInstructionsOptions}. */
  generatedAt?: string;
}

/** One section's bullet list. */
function sectionLinks(
  section: LlmsSection,
  origin: string,
  summaryMax: number
): string[] {
  return section.pages.map((page) => {
    const summary = summaryFor(page, summaryMax);
    const link = `- [${page.title}](${absoluteUrl(page.route, origin)})`;
    return summary ? `${link}: ${summary}` : link;
  });
}

/**
 * The `llms.txt` index: H1, blockquote, the agent contract, then every published
 * page as one bullet with an absolute URL and a one-line summary.
 *
 * Primary sections keep their own `##` heading. The per-category corpus sections
 * are demoted under one `## Optional` at the end, their grouping preserved as
 * bold labels that double as the advertisement for each category's split file.
 */
export function buildLlmsTxt(options: BuildLlmsTxtOptions): string {
  const {
    pages,
    origin = SITE_ORIGIN,
    title = SITE_TITLE,
    description = SITE_DESCRIPTION,
    summaryMax = SUMMARY_MAX,
    generatedAt,
  } = options;

  const sections = groupPages(pages);
  const primary = sections.filter((section) => section.category === undefined);
  const optional = sections.filter((section) => section.category !== undefined);

  const lines: string[] = [`# ${title}`, '', `> ${singleLine(description)}`, ''];
  lines.push(
    `Every published page on ${origin.replace(/^https?:\/\//, '')}, grouped the way the site navigation groups it. ` +
      `The complete text of every guide and corpus document is at ${absoluteUrl(LLMS_FULL_PATH, origin)}, ` +
      `or in the smaller per-section files listed below.`
  );

  lines.push('', `## ${INSTRUCTIONS_HEADING}`, '');
  lines.push(
    ...agentInstructions({
      origin,
      categories: optional.map((section) => section.category!),
      ...(generatedAt === undefined ? {} : { generatedAt }),
    })
  );

  for (const section of primary) {
    lines.push('', `## ${section.heading}`, '');
    lines.push(...sectionLinks(section, origin, summaryMax));
  }

  if (optional.length > 0) {
    const count = optional.reduce((total, section) => total + section.pages.length, 0);
    lines.push('', `## ${OPTIONAL_HEADING}`, '');
    lines.push(
      `The ${count} techniques-corpus document${count === 1 ? '' : 's'}, grouped by category. ` +
        'Secondary reading: skip this section for a shorter context — the sections above are the primary surface. ' +
        "Each category's full text is one file."
    );

    for (const section of optional) {
      const label = categoryLabel(section.category!);
      lines.push('', `**${label}** — ${absoluteUrl(llmsCorpusPath(section.category!), origin)}`, '');
      lines.push(...sectionLinks(section, origin, summaryMax));
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

/**
 * Whether a page carries text to concatenate. The landing page and the generated
 * corpus navigation pages are indexed in `llms.txt` and have no markdown source,
 * so every concatenated file skips them.
 */
export function hasBody(page: LlmsPage): boolean {
  return (page.body ?? '').trim() !== '';
}

/** Every page with a body, in site navigation order. */
export function documentsOf(pages: readonly LlmsPage[]): LlmsPage[] {
  return groupPages(pages).flatMap((section) => section.pages.filter(hasBody));
}

/**
 * The documents each split file carries: the authored guides, and one bucket per
 * corpus category.
 *
 * A partition, not a filter — `guides` is "no category" rather than "not corpus",
 * so the buckets are provably disjoint and their union is exactly
 * {@link documentsOf}. That is the invariant `llms-guides.txt` plus the fifteen
 * `llms-corpus-*.txt` files summing to `llms-full.txt` rests on.
 */
export function splitDocuments(pages: readonly LlmsPage[]): {
  guides: LlmsPage[];
  corpus: Map<string, LlmsPage[]>;
} {
  const guides: LlmsPage[] = [];
  const corpus = new Map<string, LlmsPage[]>();

  for (const page of documentsOf(pages)) {
    if (page.category === undefined) {
      guides.push(page);
      continue;
    }
    const bucket = corpus.get(page.category);
    if (bucket) bucket.push(page);
    else corpus.set(page.category, [page]);
  }

  return { guides, corpus };
}

/** Every document's separator block plus its trimmed body, concatenated. */
function concatenateDocuments(documents: readonly LlmsPage[], origin: string): string {
  return documents
    .map(
      (page) =>
        docSeparator({
          title: page.title,
          url: absoluteUrl(page.route, origin),
          source: page.sourcePath,
        }) + (page.body ?? '').trim()
    )
    .join('');
}

export interface BuildDocumentFileOptions {
  pages: readonly LlmsPage[];
  origin?: string;
  title?: string;
  description?: string;
}

/**
 * The shape every concatenated file shares: an H1, the site blockquote, one
 * paragraph saying what is in the file and how to reach the rest, then the
 * documents behind their separator blocks.
 */
function buildDocumentFile(
  documents: readonly LlmsPage[],
  options: {
    origin: string;
    title: string;
    description: string;
    heading: string;
    /** Sentence naming this file's contents; the navigation note is appended. */
    scope: string;
    /** Where to go for the rest of the corpus. */
    siblings: string;
  }
): string {
  const { origin, title, description, heading, scope, siblings } = options;

  const header = [
    `# ${title} — ${heading}`,
    '',
    `> ${singleLine(description)}`,
    '',
    `${scope} ` +
      `Each document is prefaced by a separator carrying its title, canonical URL and repository source path. ` +
      `${siblings} The index is at ${absoluteUrl('/llms.txt', origin)}.`,
  ].join('\n');

  return `${header}${concatenateDocuments(documents, origin)}\n`;
}

/** `N published document(s) from mechcrate.dev, in site navigation order.` */
function scopeSentence(count: number, what: string, origin: string): string {
  return (
    `The complete text of ${count} ${what}${count === 1 ? '' : 's'} ` +
    `from ${origin.replace(/^https?:\/\//, '')}, in site navigation order.`
  );
}

export interface BuildLlmsFullTxtOptions extends BuildDocumentFileOptions {}

/**
 * The `llms-full.txt` payload: the markdown body of every published guide and
 * corpus document, in navigation order, each behind a separator block.
 */
export function buildLlmsFullTxt(options: BuildLlmsFullTxtOptions): string {
  const {
    pages,
    origin = SITE_ORIGIN,
    title = SITE_TITLE,
    description = SITE_DESCRIPTION,
  } = options;

  const documents = documentsOf(pages);

  return buildDocumentFile(documents, {
    origin,
    title,
    description,
    heading: 'full text',
    scope: scopeSentence(documents.length, 'published document', origin),
    siblings:
      `Smaller per-section files — ${absoluteUrl(LLMS_GUIDES_PATH, origin)} and ` +
      `${absoluteUrl(llmsCorpusPath('<category>'), origin)} — carry the same text split up.`,
  });
}

export interface BuildLlmsGuidesTxtOptions extends BuildDocumentFileOptions {}

/**
 * `llms-guides.txt` — the authored guides (Start, Framework, AI Layer, Project
 * and the docs overview), without the corpus. The half of `llms-full.txt` that
 * describes mx itself, small enough to read whole.
 */
export function buildLlmsGuidesTxt(options: BuildLlmsGuidesTxtOptions): string {
  const {
    pages,
    origin = SITE_ORIGIN,
    title = SITE_TITLE,
    description = SITE_DESCRIPTION,
  } = options;

  const documents = splitDocuments(pages).guides;

  return buildDocumentFile(documents, {
    origin,
    title,
    description,
    heading: 'guides',
    scope: scopeSentence(documents.length, 'authored guide', origin),
    siblings:
      `The techniques corpus is published separately, one file per category ` +
      `(${absoluteUrl(llmsCorpusPath('<category>'), origin)}); everything together is ` +
      `${absoluteUrl(LLMS_FULL_PATH, origin)}.`,
  });
}

export interface BuildLlmsCorpusTxtOptions extends BuildDocumentFileOptions {
  /** Corpus category slug, e.g. `concurrency`. */
  category: string;
}

/**
 * `llms-corpus-<category>.txt` — one category's corpus documents. Fifteen of
 * these plus `llms-guides.txt` reconstruct `llms-full.txt` exactly.
 */
export function buildLlmsCorpusTxt(options: BuildLlmsCorpusTxtOptions): string {
  const {
    pages,
    category,
    origin = SITE_ORIGIN,
    title = SITE_TITLE,
    description = SITE_DESCRIPTION,
  } = options;

  const label = categoryLabel(category);
  const documents = splitDocuments(pages).corpus.get(category) ?? [];

  return buildDocumentFile(documents, {
    origin,
    title,
    description,
    heading: `${label} corpus`,
    scope: scopeSentence(documents.length, `${label.toLowerCase()} corpus document`, origin),
    siblings:
      `The category index is ${absoluteUrl(`/docs/corpus/${category}/`, origin)}; ` +
      `the whole corpus plus the guides is ${absoluteUrl(LLMS_FULL_PATH, origin)}.`,
  });
}
