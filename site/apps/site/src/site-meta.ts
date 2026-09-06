/**
 * Site-level copy that more than one surface has to agree on.
 *
 * The landing page renders these as its `<title>` and meta description; the
 * `llms.txt` index quotes the same strings for the `/` bullet. Keeping them in
 * one module is what stops the LLM index from describing the front page
 * differently from the front page.
 */

/** The landing page's title (`src/pages/index.astro`). */
export const LANDING_TITLE = 'MechCrate: an AI-native meta-framework for service ecosystems';

/** The landing page's meta description. */
export const LANDING_DESCRIPTION =
  'mx runs a subset of your service ecosystem locally, routes every project through one Traefik instance, scaffolds services that carry their wisdom, and feeds agents a techniques corpus you host yourself.';
