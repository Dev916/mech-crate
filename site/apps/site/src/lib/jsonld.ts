/**
 * Structured data (JSON-LD): the pure half.
 *
 * One `<script type="application/ld+json">` per page, holding a `@graph` of that
 * page's entities:
 *
 *   - **every docs page** → `BreadcrumbList`
 *   - **corpus documents** → `TechArticle` as well
 *   - **the landing page** → `Organization` + `WebSite` + `SoftwareApplication`
 *
 * Scoped exactly to those five types on purpose. `FAQPage` and `HowTo` lost
 * their rich results (May 2026 / September 2023) and `WebSite` `SearchAction`
 * was retired, so emitting them is markup nobody consumes — see the spec's
 * "Out of scope (folklore-killed by the audit)".
 *
 * Deliberately pure — no `node:*`, no `astro:content` value imports — for the
 * same reason `src/lib/og.ts` is: `src/components/Head.astro` imports this
 * module, and the containerised dev loop (`docker/compose/site.dev.yml`) mounts
 * only `apps/site` plus a read-only `docs/`, so anything a rendered component
 * pulls in that shells out to git breaks `make dev`. Dates arrive as an already-
 * resolved map (see `pageDates` and `src/lib/build-dates.ts`), never by reaching
 * for git from here.
 *
 * See docs/superpowers/specs/2026-09-05-seo-geo-design.md → "4. Structured data".
 */

import { categoryLabel } from '../components/corpus.ts';
import {
  SITE_DESCRIPTION,
  SITE_ORIGIN,
  SITE_TITLE,
  absoluteUrl,
} from '../loaders/lib/llms.ts';
import { GITHUB_REPO, LANDING_DESCRIPTION, LICENSE_URLS } from '../site-meta.ts';

/** Any value that may appear inside a JSON-LD node. */
export type JsonLdValue = string | number | boolean | JsonLdNode | readonly JsonLdValue[];

/** One JSON-LD entity. `undefined` members are dropped before serialisation. */
export interface JsonLdNode {
  readonly [key: string]: JsonLdValue | undefined;
}

/** The `@context` every block declares. */
export const SCHEMA_CONTEXT = 'https://schema.org';

/**
 * Stable node identities.
 *
 * The homepage defines these three; every other page references them by `@id`
 * (`publisher: { '@id': ORGANIZATION_ID }`) rather than restating the whole
 * entity, which is what lets a consumer merge 110 pages into one publisher.
 */
export const ORGANIZATION_ID = `${SITE_ORIGIN}/#organization`;
export const WEBSITE_ID = `${SITE_ORIGIN}/#website`;
export const APPLICATION_ID = `${SITE_ORIGIN}/#mx`;

/** The one-line description of `mx` the SoftwareApplication node carries. */
const APPLICATION_DESCRIPTION = SITE_DESCRIPTION;

/** Site language. Single-locale site; Starlight's `defaultLocale` is English. */
const LANGUAGE = 'en';

// ---------------------------------------------------------------------------
// Serialisation
// ---------------------------------------------------------------------------

/** Drop `undefined` members so an absent field is absent, not `"key": null`. */
function prune(node: Record<string, JsonLdValue | undefined>): JsonLdNode {
  const out: Record<string, JsonLdValue> = {};
  for (const [key, value] of Object.entries(node)) {
    if (value !== undefined) out[key] = value;
  }
  return out;
}

/**
 * Serialise a page's entities into the text of its `<script>` element.
 *
 * `<` is escaped because the payload is interpolated into a raw-text element:
 * a title or description containing `</script` would otherwise close the block
 * early and spill the rest of the graph into the document as markup. `>` and the
 * two JavaScript line terminators go with it — all four are legal `\uXXXX`
 * escapes inside a JSON string, so the result still parses as the same object.
 *
 * Throws on an empty graph rather than emitting `{"@graph":[]}`, which is a
 * structured-data validation error dressed up as valid JSON.
 */
export function jsonLdScript(nodes: readonly JsonLdNode[]): string {
  if (nodes.length === 0) {
    throw new Error('jsonLdScript: refusing to emit an empty @graph — every page has at least one entity');
  }

  return JSON.stringify({ '@context': SCHEMA_CONTEXT, '@graph': nodes })
    .replace(/</g, '\\u003c')
    .replace(/>/g, '\\u003e')
    .replace(/\u2028/g, '\\u2028')
    .replace(/\u2029/g, '\\u2029');
}

// ---------------------------------------------------------------------------
// Routes
// ---------------------------------------------------------------------------

/** `/`-prefixed, `/`-terminated form of a pathname. `/` stays `/`. */
export function normalizePath(pathname: string): string {
  const path = pathname.startsWith('/') ? pathname : `/${pathname}`;
  return path.endsWith('/') ? path : `${path}/`;
}

// ---------------------------------------------------------------------------
// BreadcrumbList
// ---------------------------------------------------------------------------

/**
 * Labels for the navigation groups, matching the sidebar labels in
 * `astro.config.mjs` so a breadcrumb names a section the way the site does.
 */
const GROUP_LABELS: Record<string, string> = {
  start: 'Start',
  framework: 'Framework',
  ai: 'AI Layer',
  corpus: 'Techniques Corpus',
  project: 'Project',
};

/** Label for `/docs/`, the documentation overview. */
const DOCS_LABEL = 'Documentation';

/** Label for the site root. */
const HOME_LABEL = 'Home';

/** One step of a breadcrumb trail. */
export interface Crumb {
  name: string;
  /** Absolute URL — Google requires `item` to be resolvable, not a path. */
  url: string;
}

/** Label for an ancestor segment (the page's own segment uses its title). */
function ancestorLabel(segments: readonly string[], depth: number): string {
  const segment = segments[depth] ?? '';
  if (depth === 0 && segment === 'docs') return DOCS_LABEL;
  if (depth === 1) return GROUP_LABELS[segment] ?? categoryLabel(segment);
  return categoryLabel(segment);
}

/**
 * Home → … → page, one crumb per URL segment.
 *
 * Every segment gets a crumb because every segment of this site's URL space is
 * a real page: `/docs/corpus/concurrency/actor-model/` passes through `/docs/`
 * (the documentation overview), `/docs/corpus/` (the corpus overview) and
 * `/docs/corpus/concurrency/` (the category index) on its way down. Skipping a
 * level would describe a hierarchy the site does not have.
 *
 * The final crumb carries the page's own title; ancestors carry their sidebar
 * label (`ai` → "AI Layer") or, for corpus categories, their category label
 * (`framework-guides` → "Framework Guides").
 */
export function breadcrumbTrail(route: string, title: string): Crumb[] {
  const path = normalizePath(route);
  const crumbs: Crumb[] = [{ name: HOME_LABEL, url: absoluteUrl('/') }];
  const segments = path.split('/').filter(Boolean);

  segments.forEach((_segment, depth) => {
    const url = absoluteUrl(`/${segments.slice(0, depth + 1).join('/')}/`);
    const last = depth === segments.length - 1;
    crumbs.push({ name: last ? title : ancestorLabel(segments, depth), url });
  });

  return crumbs;
}

/** The page's `BreadcrumbList` entity. */
export function breadcrumbList(route: string, title: string): JsonLdNode {
  const url = absoluteUrl(normalizePath(route));
  return {
    '@type': 'BreadcrumbList',
    '@id': `${url}#breadcrumbs`,
    itemListElement: breadcrumbTrail(route, title).map((crumb, index) => ({
      '@type': 'ListItem',
      position: index + 1,
      name: crumb.name,
      item: crumb.url,
    })),
  };
}

// ---------------------------------------------------------------------------
// Dates
// ---------------------------------------------------------------------------

/** `2026-08-14` — the shape `researched:` frontmatter uses. */
const DATE_ONLY_RE = /^\d{4}-\d{2}-\d{2}$/;

/** A page's publication dates, either of which may be unresolvable. */
export interface JsonLdDates {
  datePublished?: string;
  dateModified?: string;
}

export interface PageDatesInput {
  /** The page's route pathname. */
  route: string;
  /** The corpus doc's `researched:` frontmatter, when it has one. */
  researched?: string;
  /**
   * Task 2's route → ISO-timestamp map (`pageDateIndex().dates`), when the build
   * could read it. Absent in the containerised dev loop, which has no `.git`.
   */
  index?: ReadonlyMap<string, string>;
}

/**
 * The dates a page advertises, resolved from Task 2's map with a dev-only
 * fallback.
 *
 * **When the map is available** (CI and any local `astro build` inside the
 * checkout) both dates come from it, so JSON-LD, the `article:*` metas and the
 * sitemap's `<lastmod>` can only agree. The map already prefers a corpus doc's
 * `researched:` frontmatter over the source file's git commit date, so
 * `datePublished` is the researched day on the nine docs that declare one and
 * the commit date everywhere else — exactly the rule the spec states, without a
 * second date source to keep in sync.
 *
 * **When it is not** — `make dev`, whose container mounts no `.git`, so
 * `pageDateIndex()` throws — a corpus doc still knows its own `researched:`
 * frontmatter, which is enough for `datePublished`. `dateModified` has no
 * git-free source and is omitted rather than guessed; a build-stamped date is
 * the exact signal Google learned to ignore. Nothing in that fallback ships:
 * production builds run in the checkout, and `jsonld.real.test.ts` asserts the
 * real pipeline resolves both dates for corpus and authored pages alike.
 */
export function pageDates(input: PageDatesInput): JsonLdDates {
  const indexed = input.index?.get(normalizePath(input.route));
  if (indexed !== undefined) return { datePublished: indexed, dateModified: indexed };

  const researched = input.researched?.trim();
  if (researched !== undefined && DATE_ONLY_RE.test(researched)) {
    return { datePublished: researched };
  }

  return {};
}

// ---------------------------------------------------------------------------
// TechArticle (corpus documents)
// ---------------------------------------------------------------------------

export interface TechArticleInput {
  /** The page's route pathname, e.g. `/docs/corpus/ml/llm-token-cache-efficiency/`. */
  route: string;
  title: string;
  description?: string;
  /** Corpus category slug — becomes `articleSection` as its human label. */
  category: string;
  /** The doc's `sources:` frontmatter — becomes `citation[]`. */
  sources?: readonly string[];
  /** Canonical GitHub URL of the markdown this page is published from. */
  sourceUrl: string;
  datePublished?: string;
  dateModified?: string;
}

/**
 * A corpus document's `TechArticle`.
 *
 * `citation` carries the doc's source list verbatim (schema.org accepts a URL or
 * a Text for it), and `isBasedOn` points at the markdown in the repository — the
 * two properties that say, in machine-readable form, that these pages are
 * research with an audit trail rather than generated filler. That is the whole
 * reason this type is here: an answer engine deciding whether to cite a page
 * gets to see what the page itself cites.
 */
export function techArticle(input: TechArticleInput): JsonLdNode {
  const url = absoluteUrl(normalizePath(input.route));
  const citation = (input.sources ?? []).map((source) => source.trim()).filter((s) => s !== '');

  return prune({
    '@type': 'TechArticle',
    '@id': `${url}#article`,
    headline: input.title,
    name: input.title,
    description: input.description,
    url,
    mainEntityOfPage: { '@type': 'WebPage', '@id': url },
    inLanguage: LANGUAGE,
    articleSection: categoryLabel(input.category),
    datePublished: input.datePublished,
    dateModified: input.dateModified,
    isBasedOn: input.sourceUrl,
    citation: citation.length > 0 ? citation : undefined,
    isPartOf: { '@type': 'WebSite', '@id': WEBSITE_ID },
    publisher: {
      '@type': 'Organization',
      '@id': ORGANIZATION_ID,
      name: SITE_TITLE,
      url: absoluteUrl('/'),
    },
  });
}

// ---------------------------------------------------------------------------
// Homepage entities
// ---------------------------------------------------------------------------

/** The logo bitmap, as the landing page's build emitted it. */
export interface LogoImage {
  /** Absolute URL of an emitted raster file (Google's Organization logo wants one). */
  url: string;
  width: number;
  height: number;
}

/** `Organization` — the publisher every other page's `@id` reference resolves to. */
export function organization(logo: LogoImage): JsonLdNode {
  return {
    '@type': 'Organization',
    '@id': ORGANIZATION_ID,
    name: SITE_TITLE,
    url: absoluteUrl('/'),
    logo: {
      '@type': 'ImageObject',
      url: logo.url,
      width: logo.width,
      height: logo.height,
      caption: SITE_TITLE,
    },
    sameAs: [GITHUB_REPO],
  };
}

/**
 * `WebSite`.
 *
 * No `potentialAction` / `SearchAction`: Google retired the sitelinks search box
 * it fed, so the property is markup with no consumer.
 */
export function website(): JsonLdNode {
  return {
    '@type': 'WebSite',
    '@id': WEBSITE_ID,
    name: SITE_TITLE,
    url: absoluteUrl('/'),
    description: LANDING_DESCRIPTION,
    inLanguage: LANGUAGE,
    publisher: { '@id': ORGANIZATION_ID },
  };
}

/** `SoftwareApplication` — the `mx` CLI itself. */
export function softwareApplication(): JsonLdNode {
  return {
    '@type': 'SoftwareApplication',
    '@id': APPLICATION_ID,
    name: 'mx',
    alternateName: 'MechCrate CLI',
    applicationCategory: 'DeveloperApplication',
    // Comma-joined rather than an array: mx builds and runs on macOS and Linux,
    // and there is no Windows target.
    operatingSystem: 'macOS, Linux',
    description: APPLICATION_DESCRIPTION,
    url: absoluteUrl('/'),
    softwareHelp: { '@type': 'WebPage', url: absoluteUrl('/docs/') },
    codeRepository: GITHUB_REPO,
    // Dual-licensed, so two URLs rather than one SPDX expression.
    license: [...LICENSE_URLS],
    isAccessibleForFree: true,
    // Google's software rich results want an explicit free offer; `price: "0"`
    // is how "free" is stated, and it is the truth — mx is source-installed OSS.
    offers: { '@type': 'Offer', price: '0', priceCurrency: 'USD' },
    author: { '@id': ORGANIZATION_ID },
    publisher: { '@id': ORGANIZATION_ID },
  };
}

/** The landing page's three entities, in the order they are emitted. */
export function homepageGraph(logo: LogoImage): JsonLdNode[] {
  return [organization(logo), website(), softwareApplication()];
}

// ---------------------------------------------------------------------------
// Per-page graphs
// ---------------------------------------------------------------------------

export interface DocsPageGraphInput {
  route: string;
  title: string;
  description?: string;
  /** Corpus metadata, when the page is a corpus document rather than an authored one. */
  corpus?: {
    category: string;
    sourceUrl: string;
    sources?: readonly string[];
  };
  dates?: JsonLdDates;
}

/**
 * A docs page's entities: `TechArticle` first when the page is a corpus
 * document, then the `BreadcrumbList` every docs page carries.
 */
export function docsPageGraph(input: DocsPageGraphInput): JsonLdNode[] {
  const nodes: JsonLdNode[] = [];

  if (input.corpus) {
    nodes.push(
      techArticle({
        route: input.route,
        title: input.title,
        ...(input.description === undefined ? {} : { description: input.description }),
        category: input.corpus.category,
        sourceUrl: input.corpus.sourceUrl,
        ...(input.corpus.sources === undefined ? {} : { sources: input.corpus.sources }),
        ...input.dates,
      })
    );
  }

  nodes.push(breadcrumbList(input.route, input.title));
  return nodes;
}
