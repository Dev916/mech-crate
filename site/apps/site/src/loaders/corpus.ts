/**
 * Astro content-collection loader for the techniques corpus.
 *
 * Starlight owns the `docs` collection and a collection may only have one
 * loader, so this composes: Starlight's own `docsLoader()` runs first to pick up
 * the authored pages under `src/content/docs/docs/**`, then the corpus docs are
 * injected as additional entries under `docs/corpus/<category>/<slug>`. That id
 * shape is what puts them in the `/docs/corpus/**` route tree and inside the
 * "Techniques Corpus" sidebar group (`autogenerate: { directory: 'docs/corpus' }`).
 *
 * Nothing is generated onto disk — the corpus never leaves `docs/` in the repo,
 * so `npm run build` is self-contained with no prestep and no gitignored output.
 *
 * Ordering matters: Astro's glob loader deletes any collection entry it did not
 * touch during a full load, so corpus entries must be written *after* it runs.
 * Its incremental (watch) updates only delete the specific changed id, so live
 * reload in `astro dev` leaves the injected entries alone.
 *
 * See docs/superpowers/specs/2026-08-20-mechcrate-site-design.md → "Content pipeline".
 */

import type { Loader, LoaderContext } from 'astro/loaders';
import { docsLoader } from '@astrojs/starlight/loaders';

import { buildCorpus } from './lib/pipeline.ts';
import {
  collectCorpusSources,
  defaultRepoRoot,
  repoFileExistsIn,
} from './lib/sources.ts';
import { CorpusError } from './lib/types.ts';

export interface CorpusDocsLoaderOptions {
  /** Override the repository root. Defaults to the detected mech-crate root. */
  repoRoot?: string;
}

export function corpusDocsLoader(options: CorpusDocsLoaderOptions = {}): Loader {
  const base = docsLoader();

  return {
    name: 'mechcrate-corpus-loader',
    async load(context: LoaderContext): Promise<void> {
      await base.load(context);

      const repoRoot = options.repoRoot ?? defaultRepoRoot();
      const { logger, store, parseData, renderMarkdown, generateDigest } = context;

      let result;
      try {
        result = buildCorpus(collectCorpusSources(repoRoot), {
          repoFileExists: repoFileExistsIn(repoRoot),
        });
      } catch (error) {
        if (error instanceof CorpusError) {
          // Fail loudly and name the file/line — never publish a partial site.
          logger.error(`corpus: ${error.message}`);
          throw new Error(`Corpus loader failed — ${error.message}`, { cause: error });
        }
        throw error;
      }

      for (const warning of result.warnings) logger.warn(`corpus: ${warning}`);

      for (const doc of result.published) {
        const data = await parseData({ id: doc.id, data: doc.data });
        const rendered = await renderMarkdown(doc.body);
        store.set({
          id: doc.id,
          data,
          body: doc.body,
          digest: generateDigest(doc.body),
          rendered,
        });
      }

      logger.info(
        `corpus: published ${result.published.length} doc${result.published.length === 1 ? '' : 's'}` +
          (result.skipped.length > 0 ? `, held back ${result.skipped.length}` : '')
      );
    },
  };
}
