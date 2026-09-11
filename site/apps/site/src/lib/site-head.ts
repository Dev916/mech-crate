/**
 * The `<head>` fragments every page carries, whoever renders it.
 *
 * The site has two head owners: Starlight's `<Head>` override
 * (`src/components/Head.astro`) for `/docs/**`, and the hand-written head of the
 * landing page (`src/pages/index.astro`), which is not a Starlight route. Tags
 * that must appear on *every* page — the raster icon set, the manifest, the
 * `theme-color`, the analytics beacon — would otherwise have to be typed twice
 * and kept in step by hope.
 *
 * So they are built here, as strings, and both owners emit the same string
 * through `<Fragment set:html>`. Strings rather than Astro markup for two
 * reasons: it makes the parity assertable in a unit test rather than in a
 * screenshot, and it puts the beacon's `data-cf-beacon` attribute on the page
 * byte-for-byte as Cloudflare documents it (Astro would re-quote the JSON with
 * entities — harmless, but there is no reason to make an operator squint at it).
 *
 * Deliberately pure: no `node:*`, no `astro:*`. `Head.astro` imports it, and the
 * containerised dev loop mounts no `.git` — see the note at the top of that file.
 *
 * See docs/superpowers/specs/2026-09-05-seo-geo-design.md → "6. Polish".
 */

import { THEME_COLOR } from '../site-meta.ts';

/** Where the sitemap index lives — `@astrojs/sitemap`'s output path. */
export const SITEMAP_PATH = '/sitemap-index.xml';

/**
 * Icons, manifest and `theme-color`.
 *
 * `favicon.svg` is NOT here: Starlight emits its own `<link rel="shortcut icon">`
 * for it, and the landing page writes it by hand, both *before* this block —
 * which is the order that matters, because a browser that understands SVG icons
 * takes the first one it can use.
 */
export const ICON_TAGS = [
  '<link rel="icon" href="/favicon.ico" sizes="16x16 32x32 48x48" />',
  '<link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png" />',
  '<link rel="manifest" href="/site.webmanifest" />',
  `<meta name="theme-color" content="${THEME_COLOR}" />`,
] as const;

/** `<link rel="sitemap">`. Starlight emits this itself; the landing page does not. */
export const SITEMAP_TAG = `<link rel="sitemap" href="${SITEMAP_PATH}" />`;

/**
 * The Cloudflare Web Analytics beacon, or the empty string when no token is
 * configured.
 *
 * Cookie-less and asynchronous; the token is a public site identifier. An empty
 * (or whitespace) token is a supported state — the site simply ships without
 * analytics — so this returns '' rather than throwing or emitting a tag with a
 * hole in it.
 *
 * No `integrity=`: Cloudflare rolls `beacon.min.js` behind a stable URL and
 * publishes no hash for it, so a pinned SRI digest would not harden the tag, it
 * would silently switch analytics off on Cloudflare's next release. This is the
 * snippet Cloudflare documents, verbatim.
 */
export function cfBeaconTag(token: string): string {
  const trimmed = token.trim();
  if (trimmed === '') return '';
  return (
    '<script type="module" src="https://static.cloudflareinsights.com/beacon.min.js" ' +
    `data-cf-beacon='{"token": "${trimmed}"}'></script>`
  );
}

/**
 * Everything both head owners emit, in order: icons, manifest, `theme-color`,
 * then the beacon.
 *
 * `sitemap: true` prepends `<link rel="sitemap">` for the one page whose head
 * Starlight does not write.
 */
export function commonHeadHtml(
  token: string,
  options: { sitemap?: boolean } = {}
): string {
  const tags = [...(options.sitemap === true ? [SITEMAP_TAG] : []), ...ICON_TAGS];
  const beacon = cfBeaconTag(token);
  if (beacon !== '') tags.push(beacon);
  return tags.join('\n');
}
