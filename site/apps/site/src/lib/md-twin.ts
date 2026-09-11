/**
 * Markdown twins: the pure half.
 *
 * Every page that has a markdown source is published twice — once as HTML at
 * `/docs/start/install/`, and once as its raw markdown at `/docs/start/install.md`.
 * An agent that wants the text does not have to parse the chrome off the page,
 * and does not have to download the 500k-token `llms-full.txt` to get one
 * document.
 *
 * The two paths coexist on disk because they are different names: `install.md`
 * is a file, `install/` is a directory holding `index.html`. Astro's static build
 * writes both from separate routes (`src/pages/[...slug].md.ts` and Starlight's
 * own catch-all), and Cloudflare's asset server resolves each by exact path. No
 * collision, so the fallback prefix the spec allowed for (`/md/<slug>`) is not
 * needed — see the verification note in the task's proof snapshot.
 *
 * This module owns the two things both ends have to agree on:
 *
 *   1. **Which routes have a twin.** `src/components/Head.astro` decides from a
 *      pathname alone whether to emit `<link rel="alternate">`; the endpoint
 *      decides from the page list whether to write the file. Those answers must
 *      match, so {@link mdTwins} asserts they do and fails the build otherwise.
 *   2. **The file's shape.** An H1, the canonical URL, the repository source
 *      path, then the same markdown body `llms-full.txt` concatenates.
 *
 * Deliberately pure — no `node:*`, no `astro:content` value imports — for the
 * same reason `src/lib/og.ts` is: `Head.astro` imports it, and the containerised
 * dev loop mounts no `.git`, so anything a rendered component pulls in that
 * shells out to git breaks `make dev`.
 *
 * See docs/superpowers/specs/2026-09-05-seo-geo-design.md → "5. Agent surface".
 */

import { SITE_ORIGIN, classifyDocId, type LlmsPage } from '../loaders/lib/llms.ts';

/** MIME type of a twin — what `<link rel="alternate">` and `_headers` advertise. */
export const MD_TWIN_TYPE = 'text/markdown';

/** Extension appended to a page route to reach its twin. */
export const MD_TWIN_EXTENSION = '.md';

/**
 * Route slug: the pathname with its surrounding slashes trimmed.
 * `/docs/start/install/` → `docs/start/install`, `/` → ``.
 */
function routeSlug(pathname: string): string {
  return pathname.replace(/^\/+|\/+$/g, '');
}

/**
 * Whether a route has a markdown twin, decided from the route alone.
 *
 * Three families of published route have no markdown source and therefore no
 * twin: the landing page, the corpus overview, and the fifteen per-category
 * corpus indexes. All three are generated Astro pages (`src/pages/index.astro`,
 * `src/pages/docs/corpus/**`) rather than collection entries — which is exactly
 * why `llms-full.txt` skips them too.
 *
 * `hidden` is the entry's `sidebar.hidden`, the one part of the published-page
 * filter a pathname cannot express; the 404 route is the only page that sets it.
 *
 * {@link mdTwins} cross-checks this predicate against the real page list on every
 * build, so a new family of generated pages cannot quietly start advertising a
 * twin that was never written.
 */
export function routeHasMdTwin(pathname: string, hidden: boolean = false): boolean {
  if (hidden) return false;

  const slug = routeSlug(pathname);
  if (classifyDocId(slug) === null) return false;

  // The landing page: `classifyDocId` reads an empty slug as the overview.
  if (slug === '') return false;
  // The generated corpus overview and its per-category indexes.
  if (slug === 'docs/corpus') return false;
  if (/^docs\/corpus\/[^/]+$/.test(slug)) return false;

  return true;
}

/** Site-absolute path of a route's twin, e.g. `/docs/start/install.md`. */
export function mdTwinPath(route: string): string {
  return `/${routeSlug(route)}${MD_TWIN_EXTENSION}`;
}

/** Fully-qualified twin URL — what `<link rel="alternate">` carries. */
export function mdTwinUrl(route: string, origin: string = SITE_ORIGIN): string {
  return `${origin}${mdTwinPath(route)}`;
}

/**
 * The twin's text: a title heading, the canonical URL, the repository source
 * path, a rule, and then the page's markdown body verbatim.
 *
 * The header is the same three facts `llms-full.txt`'s separator block carries,
 * so a document retrieved either way identifies itself identically. The body is
 * `entry.body` — the markdown as authored, minus frontmatter — which is what
 * `llms-full.txt` concatenates, so the two can be compared byte for byte.
 */
export function buildMdTwin(page: LlmsPage, origin: string = SITE_ORIGIN): string {
  const header = [`# ${page.title}`, '', `URL: ${origin}${page.route}`];
  if (page.sourcePath !== undefined) header.push(`Source: ${page.sourcePath}`);

  return `${header.join('\n')}\n\n---\n\n${(page.body ?? '').trim()}\n`;
}

/**
 * Every page that gets a twin, keyed by the route parameter the endpoint turns
 * into a static path (`docs/start/install` → `dist/docs/start/install.md`).
 *
 * Throws when {@link routeHasMdTwin} and the page list disagree in either
 * direction. A page with a body but no twin is a document an agent cannot fetch;
 * a twin advertised in `<head>` with no file behind it is a 404 in an agent's
 * retrieval path. Both are silent failures at runtime, so they are loud ones at
 * build time.
 */
export function mdTwins(pages: readonly LlmsPage[]): Map<string, LlmsPage> {
  const twins = new Map<string, LlmsPage>();

  for (const page of pages) {
    const hasSource = (page.body ?? '').trim() !== '';
    const predicted = routeHasMdTwin(page.route);

    if (hasSource !== predicted) {
      throw new Error(
        `md twins: ${page.route} ${hasSource ? 'has a markdown body but' : 'has no markdown body yet'} ` +
          `routeHasMdTwin() says ${predicted}. Head.astro decides from the route alone, so the ` +
          'predicate in src/lib/md-twin.ts has to be taught about this page family.'
      );
    }

    if (!hasSource) continue;

    const slug = routeSlug(page.route);
    const clash = twins.get(slug);
    if (clash) {
      throw new Error(
        `md twins: ${page.route} and ${clash.route} both map to /${slug}${MD_TWIN_EXTENSION}.`
      );
    }
    twins.set(slug, page);
  }

  return twins;
}
