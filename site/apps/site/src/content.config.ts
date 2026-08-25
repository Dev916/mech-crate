import { defineCollection, z } from 'astro:content';
import { docsSchema } from '@astrojs/starlight/schema';
import { corpusDocsLoader } from './loaders/corpus.ts';

/**
 * Starlight's content collections.
 *
 * `corpusDocsLoader()` wraps Starlight's `docsLoader()`: the authored pages are
 * globbed from `src/content/docs/docs/**` (one level deeper than the collection
 * base so every Starlight route is prefixed with `/docs/`, leaving `/` for the
 * custom landing page), and the techniques corpus is then injected from the repo
 * root as extra entries under `docs/corpus/<category>/<slug>`.
 *
 * See docs/superpowers/specs/2026-08-20-mechcrate-site-design.md — Content pipeline.
 */

/**
 * Corpus metadata carried through from `docs/development/*.md` frontmatter.
 * Starlight's schema is a plain Zod object, which strips unknown keys, so these
 * fields have to be declared for the page template to be able to read them.
 */
const corpusMetadata = z
  .object({
    category: z.string(),
    slug: z.string(),
    /** Repo-relative path of the source doc, e.g. `docs/development/appendix-fsm.md`. */
    repoPath: z.string(),
    /** Canonical GitHub URL for the source doc. */
    sourceUrl: z.string().url(),
    complexity: z.string().optional(),
    languages: z.array(z.string()).optional(),
    useCases: z.array(z.string()).optional(),
    provenance: z.string().optional(),
    researched: z.string().optional(),
    sources: z.array(z.string()).optional(),
    /** Set when the pipeline had to synthesise the value (a build warning was emitted). */
    inferredTitle: z.boolean().default(false),
    inferredCategory: z.boolean().default(false),
  })
  .optional();

export const collections = {
  docs: defineCollection({
    loader: corpusDocsLoader(),
    schema: docsSchema({ extend: z.object({ corpus: corpusMetadata }) }),
  }),
};
