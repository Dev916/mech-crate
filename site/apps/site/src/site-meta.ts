/**
 * Site-level copy that more than one surface has to agree on.
 *
 * The landing page renders these as its `<title>` and meta description; the
 * `llms.txt` index quotes the same strings for the `/` bullet. Keeping them in
 * one module is what stops the LLM index from describing the front page
 * differently from the front page.
 */

/** The landing page's title (`src/pages/index.astro`). */
export const LANDING_TITLE = 'MechCrate — an AI-native meta-framework for service ecosystems';

/** The landing page's meta description. */
export const LANDING_DESCRIPTION =
  'mx runs a subset of your service ecosystem locally, routes every project through one Traefik instance, scaffolds services that carry their wisdom, and feeds agents a techniques corpus you host yourself.';

/**
 * Canonical repository URL.
 *
 * The landing page links it from the masthead, the CTA and the footer; the
 * homepage JSON-LD carries it twice more, as the Organization's `sameAs` and the
 * SoftwareApplication's `codeRepository`. One constant so a fork or a rename
 * cannot leave the structured data pointing somewhere the links do not.
 */
export const GITHUB_REPO = 'https://github.com/Dev916/mech-crate';

/**
 * Both license texts, in the order the footer names them.
 *
 * mx is dual-licensed "Apache-2.0 or MIT, at your option", which schema.org
 * expresses as two `license` URLs rather than one SPDX expression.
 */
export const LICENSE_URLS = [
  `${GITHUB_REPO}/blob/main/LICENSE-APACHE`,
  `${GITHUB_REPO}/blob/main/LICENSE-MIT`,
] as const;

/**
 * Cloudflare Web Analytics site token (spec's analytics decision).
 *
 * A public identifier, not a secret — it ships in the HTML of every page by
 * design, and the beacon is cookie-less, which is what keeps the site out of
 * consent-banner territory. It lives here rather than in an env var because a
 * static build has no runtime config, and because referrers are the only way to
 * observe an AI answer engine citing the site.
 *
 * Emptying the string is the supported way to turn the beacon off: nothing is
 * emitted and the build stays green — see `cfBeaconTag()` in
 * `src/lib/site-head.ts`, which is unit-tested for exactly that.
 */
export const CF_BEACON_TOKEN = '4c7a272fdcaf472ba395a69fbf05507c';

/**
 * `theme-color`, the manifest's `theme_color`, and the ground the raster icons
 * are composited on: `--sl-color-black` from the dark palette, hsl(224 10% 10%).
 *
 * One value rather than a light/dark pair — dark is the site's base theme, and a
 * browser that paints its chrome to match should paint it the colour the site
 * opens in.
 */
export const THEME_COLOR = '#17181c';
