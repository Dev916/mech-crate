/**
 * Social cards: the pure half.
 *
 * One 1200×630 PNG per published page, generated at build time by
 * `src/pages/og/[...route].ts` and pointed at from every page's `<head>`. This
 * module owns the two things both ends have to agree on:
 *
 *   1. **The route mapping.** `/docs/start/install/` → `/og/docs/start/install.png`,
 *      `/` → `/og/index.png`. The endpoint derives its `getStaticPaths` from it;
 *      `src/components/Head.astro` and `src/pages/index.astro` derive the
 *      `og:image` URL from it. A card whose route the endpoint does not publish
 *      is a 404 in a share preview, so the mapping is one function, not two.
 *   2. **The card's copy.** Title, the `MechCrate · <section>` kicker, and a
 *      truncated summary — cut here rather than in the renderer, because
 *      titles run to 95 characters and descriptions to 478, and CanvasKit will
 *      happily lay both off the bottom of the canvas.
 *
 * Deliberately pure — no `node:*`, no `astro:content` value imports. That is
 * what lets `Head.astro` import it: the containerised dev loop
 * (`docker/compose/site.dev.yml`) mounts only `apps/site` plus a read-only
 * `docs/`, so anything a rendered component pulls in that shells out to git —
 * `src/loaders/lib/dates.ts`, for one — breaks `make dev`. Route math only.
 *
 * See docs/superpowers/specs/2026-09-05-seo-geo-design.md → "2. Social cards".
 */

import { categoryLabel } from '../components/corpus.ts';
import {
  SITE_ORIGIN,
  classifyDocId,
  truncateSummary,
  type LlmsPage,
} from '../loaders/lib/llms.ts';

/** Card width in pixels — the `og:image:width` every page advertises. */
export const OG_CARD_WIDTH = 1200;

/** Card height in pixels — the `og:image:height` every page advertises. */
export const OG_CARD_HEIGHT = 630;

/** Card MIME type — the `og:image:type` every page advertises. */
export const OG_CARD_TYPE = 'image/png';

/** Slug of the landing page's card, so `/og/index.png` is written for `/`. */
export const OG_LANDING_SLUG = 'index';

/**
 * The card palette, in the `[R, G, B]` triples astro-og-canvas takes.
 *
 * Converted from the design tokens at the top of `src/styles/landing.css`, which
 * are themselves the values Starlight ships. Dark is the site's base theme, so
 * the card is dark too: a `--sl-color-gray-6` → `--sl-color-black` vertical
 * wash, `--mx-amber` down the leading edge for the crate/raccoon identity,
 * white title, `--sl-color-gray-2` for the smaller text.
 */
export const OG_PALETTE = {
  /** `--sl-color-gray-6`, hsl(224 14% 16%). Top of the background gradient. */
  bgTop: [35, 38, 47],
  /** `--sl-color-black`, hsl(224 10% 10%). Bottom of the background gradient. */
  bgBottom: [23, 24, 28],
  /** `--mx-amber`, hsl(44 88% 66%). The leading-edge rule. */
  accent: [245, 204, 92],
  /** `--sl-color-white`. Title ink. */
  title: [255, 255, 255],
  /** `--sl-color-gray-2`, hsl(224 6% 77%). Kicker and summary ink. */
  body: [193, 195, 200],
} as const satisfies Record<string, readonly [number, number, number]>;

/**
 * Longest title drawn on a card. At 54px over a 1008px text column this is
 * three lines, which clears the description with room to spare.
 */
const TITLE_MAX = 84;

/** Longest summary drawn on a card — three lines at 26px. */
const DESCRIPTION_MAX = 150;

/** The site name, drawn as the first half of every kicker. */
const SITE_NAME = 'MechCrate';

/**
 * The landing card's kicker. Its title is already `MechCrate — an AI-native…`,
 * so repeating the site name under it just says the word twice; the domain
 * identifies the site without the echo, and a share preview is exactly where a
 * reader is deciding whether the domain is worth a click.
 */
const LANDING_LABEL = 'mechcrate.dev';

/**
 * Kicker labels for the navigation groups, matching the sidebar labels in
 * `astro.config.mjs` so a card names a section the way the site does. Corpus
 * documents override this with their category label.
 */
const SECTION_LABELS: Record<string, string> = {
  start: 'Start',
  framework: 'Framework',
  ai: 'AI Layer',
  project: 'Project',
  corpus: 'Techniques Corpus',
  overview: 'Documentation',
};

/** A single card, ready for the renderer. */
export interface OgCard {
  /** Public route the card belongs to, e.g. `/docs/start/install/`. */
  route: string;
  /** The big line. Truncated to {@link TITLE_MAX}. */
  title: string;
  /** The kicker under the title, e.g. `MechCrate · Concurrency`. */
  label: string;
  /** The summary under the kicker. Truncated to {@link DESCRIPTION_MAX}; may be empty. */
  description: string;
  /** `og:image:alt` text — the untruncated title. */
  alt: string;
}

/**
 * Card slug for a route: the route with its slashes trimmed, and `index` for
 * the site root. `/docs/` → `docs` and `/docs/start/` → `docs/start` coexist
 * fine on disk as `og/docs.png` alongside `og/docs/start.png`.
 */
export function ogCardSlug(route: string): string {
  const trimmed = route.replace(/^\/+|\/+$/g, '');
  return trimmed === '' ? OG_LANDING_SLUG : trimmed;
}

/** Site-absolute path of a route's card, e.g. `/og/docs/start.png`. */
export function ogCardPath(route: string): string {
  return `/og/${ogCardSlug(route)}.png`;
}

/** Fully-qualified card URL — what `og:image` has to carry to be usable. */
export function ogCardUrl(route: string, origin: string = SITE_ORIGIN): string {
  return `${origin}${ogCardPath(route)}`;
}

/**
 * Whether a Starlight route gets a card, decided from the route alone.
 *
 * Mirrors the filter in `pagesFromDocsEntries()` — which is what feeds the
 * endpoint — so the tags a page emits and the cards the build writes can only
 * agree. `hidden` is the entry's `sidebar.hidden`, the one part of that filter
 * a pathname cannot express; the 404 route is the only page that sets it.
 */
export function routeHasOgCard(pathname: string, hidden: boolean = false): boolean {
  if (hidden) return false;
  // `classifyDocId` already collapses a trailing `index` segment, so the
  // landing page's `index` slug classifies as the overview, same as `/docs`.
  return classifyDocId(ogCardSlug(pathname)) !== null;
}

/**
 * The kicker line: site name plus the page's section or corpus category.
 *
 * The section is dropped when the title already contains it, which is what
 * keeps the group and category index cards from reading `Start` over
 * `MechCrate · Start`. Nothing is lost — the words are still on the card.
 */
export function ogCardLabel(page: LlmsPage): string {
  if (page.route === '/') return LANDING_LABEL;

  const section = page.category
    ? categoryLabel(page.category)
    : (SECTION_LABELS[page.kind] ?? SECTION_LABELS.overview!);

  if (page.title.toLowerCase().includes(section.toLowerCase())) return SITE_NAME;
  return `${SITE_NAME} · ${section}`;
}

/** One published page → one card. */
export function ogCard(page: LlmsPage): OgCard {
  return {
    route: page.route,
    title: truncateSummary(page.title, TITLE_MAX),
    label: ogCardLabel(page),
    description: page.description ? truncateSummary(page.description, DESCRIPTION_MAX) : '',
    alt: page.title,
  };
}

/**
 * Every published page → the card map the endpoint turns into static paths,
 * keyed by card slug.
 *
 * Throws on a slug collision rather than silently dropping a page's card: two
 * routes mapping to one file would leave one of them advertising an image that
 * shows the other page's title.
 */
export function ogCards(pages: readonly LlmsPage[]): Map<string, OgCard> {
  const cards = new Map<string, OgCard>();

  for (const page of pages) {
    const slug = ogCardSlug(page.route);
    const clash = cards.get(slug);
    if (clash) {
      throw new Error(
        `og cards: ${page.route} and ${clash.route} both map to /og/${slug}.png. ` +
          'Card slugs come from routes, so two routes cannot differ only by trailing slash.'
      );
    }
    cards.set(slug, ogCard(page));
  }

  return cards;
}
