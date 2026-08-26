/**
 * The pure corpus pipeline: raw source files in, publishable page data out.
 *
 * Deliberately IO-free so the whole contract — filtering, slugs, link rewriting,
 * the secret lint, frontmatter fallbacks — is unit testable without a build.
 * `src/loaders/corpus.ts` supplies the filesystem and hands results to Astro.
 *
 * Order matters: frontmatter and slugs are resolved for every source first, so
 * link rewriting can see the complete published-route index (a doc may link
 * forward to a doc that appears later in the list).
 */

import { isHeldBack, normalizeMetadata, parseFrontmatter } from './frontmatter.ts';
import { GITHUB_BLOB_BASE, rewriteLinks } from './links.ts';
import { findSecrets, formatFindings } from './secrets.ts';
import { corpusEntryId, corpusRoute, fileStem, sanitizeSlug } from './slug.ts';
import {
  CorpusError,
  type CorpusBuildResult,
  type CorpusDoc,
  type CorpusSkip,
  type CorpusSource,
} from './types.ts';

export interface BuildCorpusOptions {
  /** Does this repo-relative path exist on disk? Defaults to "no". */
  repoFileExists?: (repoPath: string) => boolean;
  /** Base for GitHub blob links. Overridable for tests. */
  githubBlobBase?: string;
}

interface Staged {
  source: CorpusSource;
  body: string;
  bodyStartLine: number;
  slug: string;
  meta: ReturnType<typeof normalizeMetadata>;
}

export function buildCorpus(
  sources: readonly CorpusSource[],
  options: BuildCorpusOptions = {}
): CorpusBuildResult {
  const repoFileExists = options.repoFileExists ?? (() => false);
  const githubBlobBase = options.githubBlobBase ?? GITHUB_BLOB_BASE;

  const warnings: string[] = [];
  const skipped: CorpusSkip[] = [];
  const staged: Staged[] = [];

  // Pass 1 — parse, filter, resolve slugs and categories.
  for (const source of sources) {
    const { data, body, bodyStartLine } = parseFrontmatter(source.repoPath, source.raw);

    if (isHeldBack(data)) {
      skipped.push({ repoPath: source.repoPath, reason: 'publish: false' });
      continue;
    }

    const meta = normalizeMetadata(source.repoPath, data, {
      defaultCategory: source.defaultCategory,
    });
    warnings.push(...meta.warnings);

    const slug = sanitizeSlug(fileStem(source.repoPath));
    if (slug === '') {
      throw new CorpusError(source.repoPath, 'filename produces an empty slug');
    }

    staged.push({ source, body, bodyStartLine, slug, meta });
  }

  // Pass 2 — build the route index and reject collisions before anything renders.
  const routeByRepoPath = new Map<string, string>();
  const ownerById = new Map<string, string>();
  for (const item of staged) {
    const id = corpusEntryId(item.meta.category, item.slug);
    const existing = ownerById.get(id);
    if (existing !== undefined) {
      throw new CorpusError(
        item.source.repoPath,
        `route collision on \`/${id}/\` — already claimed by \`${existing}\``
      );
    }
    ownerById.set(id, item.source.repoPath);
    routeByRepoPath.set(item.source.repoPath, corpusRoute(item.meta.category, item.slug));
  }

  // Pass 3 — rewrite links, lint, emit.
  const published: CorpusDoc[] = [];
  for (const item of staged) {
    const { source, bodyStartLine, slug, meta } = item;

    const rewritten = rewriteLinks({
      repoPath: source.repoPath,
      body: item.body,
      bodyStartLine,
      routeByRepoPath,
      repoFileExists,
      githubBlobBase,
    });
    warnings.push(...rewritten.warnings);

    const sourceUrl = `${githubBlobBase}${source.repoPath}`;
    const data = {
      title: meta.title,
      ...(meta.description === undefined ? {} : { description: meta.description }),
      editUrl: sourceUrl,
      corpus: {
        category: meta.category,
        slug,
        repoPath: source.repoPath,
        sourceUrl,
        ...(meta.complexity === undefined ? {} : { complexity: meta.complexity }),
        ...(meta.languages === undefined ? {} : { languages: meta.languages }),
        ...(meta.useCases === undefined ? {} : { useCases: meta.useCases }),
        ...(meta.provenance === undefined ? {} : { provenance: meta.provenance }),
        ...(meta.researched === undefined ? {} : { researched: meta.researched }),
        ...(meta.sources === undefined ? {} : { sources: meta.sources }),
        inferredTitle: meta.inferredTitle,
        inferredCategory: meta.inferredCategory,
      },
    };

    // Lint what actually ships: the rewritten body plus every frontmatter value
    // that reaches the page. Body offsets are mapped back to real file lines.
    const bodyFindings = findSecrets(rewritten.body, bodyStartLine);
    const metaFindings = findSecrets(serializeMetadata(data), 1).map((finding) => ({
      ...finding,
      line: 1,
      description: `${finding.description} (in frontmatter)`,
    }));
    const findings = [...bodyFindings, ...metaFindings];
    if (findings.length > 0) {
      throw new CorpusError(
        source.repoPath,
        `secret lint failed (${findings.length} match${findings.length === 1 ? '' : 'es'}):\n${formatFindings(
          source.repoPath,
          findings
        )}`
      );
    }

    published.push({
      id: corpusEntryId(meta.category, slug),
      route: corpusRoute(meta.category, slug),
      repoPath: source.repoPath,
      category: meta.category,
      slug,
      body: rewritten.body,
      bodyStartLine,
      data,
    });
  }

  return { published, skipped, warnings };
}

/** Flatten page data to a scannable string for the frontmatter half of the lint. */
function serializeMetadata(data: { title: string; description?: string; corpus: object }): string {
  return [data.title, data.description ?? '', JSON.stringify(data.corpus)].join('\n');
}
