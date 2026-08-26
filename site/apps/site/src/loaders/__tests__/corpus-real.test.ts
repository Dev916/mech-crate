/**
 * Smoke test against the real repository corpus.
 *
 * The unit tests above pin behaviour with fixtures; this one guards the actual
 * contract the build depends on — every doc in `docs/development/` plus the three
 * allowlisted root guides parses, routes and passes the secret lint. It is the
 * same code path `astro build` runs, minus Astro.
 */

import { describe, expect, it } from 'vitest';
import { buildCorpus } from '../lib/pipeline.ts';
import {
  ROOT_GUIDE_ALLOWLIST,
  ROOT_GUIDE_CATEGORY,
  collectCorpusSources,
  defaultRepoRoot,
  repoFileExistsIn,
} from '../lib/sources.ts';

const repoRoot = defaultRepoRoot();
const sources = collectCorpusSources(repoRoot);
const result = buildCorpus(sources, { repoFileExists: repoFileExistsIn(repoRoot) });

describe('the real corpus', () => {
  it('finds the repo root from the nested app without depending on cwd', () => {
    expect(repoRoot.endsWith('mech-crate')).toBe(true);
  });

  it('collects docs/development/*.md plus the three allowlisted root guides', () => {
    const rootGuides = sources.filter((s) => !s.repoPath.startsWith('docs/development/'));
    expect(rootGuides.map((s) => s.repoPath)).toEqual([...ROOT_GUIDE_ALLOWLIST]);
    expect(rootGuides.every((s) => s.defaultCategory === ROOT_GUIDE_CATEGORY)).toBe(true);
    expect(sources.length).toBeGreaterThanOrEqual(60);
  });

  it('publishes every doc that is not held back, with no secret-lint hits', () => {
    expect(result.published.length).toBe(sources.length - result.skipped.length);
    expect(result.published.length).toBeGreaterThan(0);
  });

  it('gives every published doc a unique /docs/corpus/ route', () => {
    const routes = result.published.map((d) => d.route);
    expect(new Set(routes).size).toBe(routes.length);
    expect(routes.every((r) => r.startsWith('/docs/corpus/') && r.endsWith('/'))).toBe(true);
  });

  it('only ever emits corpus links that point at a route it actually publishes', () => {
    const routes = new Set(result.published.map((d) => d.route));
    const dangling: string[] = [];
    for (const doc of result.published) {
      for (const match of doc.body.matchAll(/\]\((\/docs\/corpus\/[^)#\s]*)/g)) {
        if (!routes.has(match[1]!)) dangling.push(`${doc.repoPath} → ${match[1]}`);
      }
    }
    expect(dangling).toEqual([]);
  });

  it('leaves no unrewritten relative markdown links in published bodies', () => {
    const offenders = result.published.filter((doc) =>
      /\]\((?!https?:|\/|#|sediment:|mailto:)[^)\s]*\.md[^)\s]*\)/.test(doc.body)
    );
    expect(offenders.map((d) => d.repoPath)).toEqual([]);
  });
});
