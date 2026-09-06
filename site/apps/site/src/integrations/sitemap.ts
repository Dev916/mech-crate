/**
 * The site's sitemap: `@astrojs/sitemap`, wired to real per-page dates.
 *
 * Two integrations, deliberately:
 *
 *   1. `@astrojs/sitemap` with a `serialize` hook that stamps every URL with the
 *      `lastmod` resolved by `src/loaders/lib/dates.ts` (frontmatter, else the
 *      source file's git commit date).
 *   2. a guard that re-reads the written XML and fails the build unless 100% of
 *      `<url>` entries carry a `<lastmod>`.
 *
 * The guard is not belt-and-braces. `@astrojs/sitemap` CATCHES anything thrown
 * from `serialize`, logs it, and returns — leaving no sitemap at all while the
 * build still exits 0. Undated pages are precisely the failure this task exists
 * to remove, so the assertion has to live somewhere a throw is fatal, and
 * `astro:build:done` is that place.
 *
 * Declaring `@astrojs/sitemap` here is also what stops Starlight injecting its
 * own copy: `@astrojs/starlight`'s config:setup only pushes `starlightSitemap()`
 * when no integration named `@astrojs/sitemap` is already present (see
 * node_modules/@astrojs/starlight/index.ts). Starlight's own config is `{}` for
 * a single-locale site, so the URL set is unchanged — this only adds lastmod.
 *
 * See docs/superpowers/specs/2026-09-05-seo-geo-design.md → "3. Freshness".
 */

import { readFileSync, readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';

import sitemap from '@astrojs/sitemap';
import type { AstroIntegration } from 'astro';

import {
  auditSitemapXml,
  lastmodForUrl,
  pageDateIndex,
  type PageDateIndex,
  type PageDateIndexOptions,
} from '../loaders/lib/dates.ts';

/** `sitemap-0.xml`, `sitemap-1.xml`, … — the chunks, not the index. */
const CHUNK_RE = /^sitemap-\d+\.xml$/;
const INDEX_FILE = 'sitemap-index.xml';

/**
 * `[sitemap, guard]`, in the order they must run — the guard's `astro:build:done`
 * has to fire after the one that writes the files, and Astro runs integration
 * hooks in `integrations` order.
 */
export function mechcrateSitemap(options: PageDateIndexOptions = {}): AstroIntegration[] {
  // Built once per build: the git walk and the corpus pipeline are not cheap
  // enough to repeat 110 times, once per serialized URL.
  let index: PageDateIndex | undefined;
  const dates = (): PageDateIndex => (index ??= pageDateIndex(options));

  const generator = sitemap({
    serialize(item) {
      item.lastmod = lastmodForUrl(item.url, dates());
      return item;
    },
  });

  const guard: AstroIntegration = {
    name: 'mechcrate:sitemap-lastmod',
    hooks: {
      'astro:build:done': ({ dir, logger }) => {
        const outDir = fileURLToPath(dir);
        const files = readdirSync(outDir).filter((name) => CHUNK_RE.test(name)).sort();

        if (files.length === 0) {
          throw new Error(
            'sitemap: no `sitemap-<n>.xml` was written. @astrojs/sitemap swallows errors thrown ' +
              'while serializing pages and writes nothing — look for an "Error serializing pages" ' +
              'line above this one for the page that could not be dated.'
          );
        }

        let total = 0;
        const missing: string[] = [];
        for (const name of files) {
          const audit = auditSitemapXml(readFileSync(join(outDir, name), 'utf8'));
          total += audit.total;
          missing.push(...audit.missing);
        }

        if (total === 0) throw new Error(`sitemap: ${files.join(', ')} contains no <url> entries.`);
        if (missing.length > 0) {
          throw new Error(
            `sitemap: ${missing.length} of ${total} URLs have no <lastmod>:\n` +
              missing.map((loc) => `  · ${loc}`).join('\n')
          );
        }

        for (const warning of dates().warnings) logger.warn(warning);

        const fromFrontmatter = [...dates().origins.values()].filter(
          (origin) => origin === 'frontmatter'
        ).length;
        logger.info(
          `${total} URLs, all with <lastmod> ` +
            `(${fromFrontmatter} from \`researched:\` frontmatter, ${total - fromFrontmatter} from git) ` +
            `across ${files.join(', ')} + ${INDEX_FILE}`
        );
      },
    },
  };

  return [generator, guard];
}
