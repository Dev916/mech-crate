import { defineCollection } from 'astro:content';
import { docsLoader } from '@astrojs/starlight/loaders';
import { docsSchema } from '@astrojs/starlight/schema';

/**
 * Starlight's content collections.
 *
 * `docsLoader()` globs `src/content/docs/**`; the authored pages live one level
 * deeper (`src/content/docs/docs/**`) so every Starlight route is prefixed with
 * `/docs/`, leaving `/` for the custom landing page in `src/pages/index.astro`.
 *
 * A later task adds the repo-root corpus loader alongside this collection (see
 * docs/superpowers/specs/2026-08-20-mechcrate-site-design.md — Content pipeline).
 */
export const collections = {
  docs: defineCollection({ loader: docsLoader(), schema: docsSchema() }),
};
