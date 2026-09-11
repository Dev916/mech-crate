/**
 * `/og/<route>.png` — one 1200×630 social card per published page.
 *
 * The IO half of the social cards; the mapping and the copy live in
 * `src/lib/og.ts`. Prerendered, because the site is a static bundle on
 * Cloudflare Workers assets and runtime image generation there is impossible —
 * no `sharp`, no native canvas. Every card is therefore a file in `dist/og/`.
 *
 * The page list is `collectLlmsPages()`, the same function `/llms.txt` and
 * `/llms-full.txt` use, so the set of cards is by construction the set of pages
 * the site says it publishes. Endpoints are not pages, so these routes stay out
 * of the sitemap and out of the Pagefind index (which reads HTML only).
 *
 * See docs/superpowers/specs/2026-09-05-seo-geo-design.md → "2. Social cards".
 */

import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

import { OGImageRoute } from 'astro-og-canvas';

import { OG_PALETTE, ogCards, type OgCard } from '../../lib/og.ts';
import { collectLlmsPages } from '../../loaders/llms.ts';

export const prerender = true;

/**
 * Noto Sans, the family astro-og-canvas defaults to, pinned to explicit weights
 * so the title can be bold.
 *
 * Downloaded rather than vendored, but downloaded by *us*: astro-og-canvas
 * fetches its default font itself and, when the fetch fails, hands CanvasKit an
 * empty font manager — which draws no text at all and still returns a valid
 * PNG. A network blip in CI would ship 111 blank cards and a green build. So the
 * fetch happens here, where a failure can throw, and the bytes are cached on
 * disk so only the first build on a machine needs the network.
 */
const FONT_URLS = [
  'https://api.fontsource.org/v1/fonts/noto-sans/latin-400-normal.ttf',
  'https://api.fontsource.org/v1/fonts/noto-sans/latin-700-normal.ttf',
];

/** Sits beside astro-og-canvas's own `./node_modules/.astro-og-canvas` cache. */
const FONT_CACHE_DIR = './node_modules/.mechcrate-og-fonts';

/** Local paths for {@link FONT_URLS}, downloading any that are not cached yet. */
async function cachedFonts(): Promise<string[]> {
  await mkdir(FONT_CACHE_DIR, { recursive: true });

  return Promise.all(
    FONT_URLS.map(async (url) => {
      const file = join(FONT_CACHE_DIR, url.slice(url.lastIndexOf('/') + 1));

      const cached = await readFile(file).catch(() => undefined);
      if (cached && cached.byteLength > 0) return file;

      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(
          `og cards: could not download ${url} — ${response.status} ${response.statusText}. ` +
            'The card renderer silently draws nothing without a font, so the build stops here ' +
            `instead. Retry with network access, or drop a copy of the file at ${file}.`
        );
      }

      await writeFile(file, Buffer.from(await response.arrayBuffer()));
      return file;
    })
  );
}

const fonts = await cachedFonts();
const cards = ogCards(await collectLlmsPages());

export const { getStaticPaths, GET } = await OGImageRoute({
  pages: Object.fromEntries(cards),

  // `pages` is already keyed by card slug, so the route parameter is the slug
  // plus the extension. (The library's default would additionally collapse a
  // trailing `/index`, which would collide `/docs/` with `/docs/index/`.)
  getSlug: (slug) => `${slug}.png`,

  getImageOptions: (_slug, card: OgCard) => ({
    title: card.title,
    // astro-og-canvas draws one paragraph: title, a small gap, then this. The
    // kicker leads so the section reads as a byline under the title.
    description: card.description ? `${card.label}\n${card.description}` : card.label,

    logo: { path: './src/assets/mechcrate-logo.png', size: [96] },

    bgGradient: [[...OG_PALETTE.bgTop], [...OG_PALETTE.bgBottom]],
    border: { color: [...OG_PALETTE.accent], width: 12, side: 'inline-start' },
    padding: 60,

    font: {
      title: {
        color: [...OG_PALETTE.title],
        size: 54,
        lineHeight: 1.15,
        weight: 'Bold',
        families: ['Noto Sans'],
      },
      description: {
        color: [...OG_PALETTE.body],
        size: 26,
        lineHeight: 1.4,
        weight: 'Normal',
        families: ['Noto Sans'],
      },
    },
    fonts,

    format: 'PNG',
  }),
});
