/**
 * Unit tests for the pure helpers behind the corpus page chrome.
 *
 * The `.astro` components are thin: they decide *whether* to render from
 * `corpusMetaOf()` and *what* to say from `provenanceSummary()`/`categoryLabel()`.
 * Pinning those three here means the build only has to prove the wiring.
 */

import { describe, expect, it } from 'vitest';
import { categoryLabel, corpusMetaOf, provenanceSummary, type CorpusMeta } from '../corpus.ts';

function meta(overrides: Partial<CorpusMeta> = {}): CorpusMeta {
  return {
    category: 'process',
    slug: 'llm-token-cache-efficiency',
    repoPath: 'docs/development/llm-token-cache-efficiency.md',
    sourceUrl:
      'https://github.com/Dev916/mech-crate/blob/main/docs/development/llm-token-cache-efficiency.md',
    inferredTitle: false,
    inferredCategory: false,
    ...overrides,
  };
}

describe('corpusMetaOf', () => {
  it('returns the metadata for a corpus entry', () => {
    const corpus = meta();
    expect(corpusMetaOf({ data: { corpus } })).toBe(corpus);
  });

  it('returns undefined for authored pages, which must not get corpus chrome', () => {
    expect(corpusMetaOf({ data: { title: 'Install' } })).toBeUndefined();
  });

  it('tolerates a missing entry or missing data rather than throwing mid-render', () => {
    expect(corpusMetaOf(undefined)).toBeUndefined();
    expect(corpusMetaOf({})).toBeUndefined();
  });

  it('rejects a corpus object without a category — chrome needs the category link', () => {
    expect(corpusMetaOf({ data: { corpus: { slug: 'x' } } })).toBeUndefined();
  });
});

describe('provenanceSummary', () => {
  it('renders provenance, date and source count as the spec line', () => {
    expect(
      provenanceSummary(
        meta({ provenance: 'researched', researched: '2026-08-14', sources: Array(24).fill('u') })
      )
    ).toBe('Researched 2026-08-14 · 24 sources');
  });

  it('singularises a lone source', () => {
    expect(provenanceSummary(meta({ provenance: 'researched', sources: ['u'] }))).toBe(
      'Researched · 1 source'
    );
  });

  it('drops the count when a researched doc lists no sources', () => {
    expect(provenanceSummary(meta({ provenance: 'researched', researched: '2026-07-18' }))).toBe(
      'Researched 2026-07-18'
    );
  });

  it('renders nothing for docs that predate the research pipeline', () => {
    expect(provenanceSummary(meta())).toBeUndefined();
    expect(provenanceSummary(meta({ sources: ['u'], researched: '2026-08-14' }))).toBeUndefined();
  });
});

describe('categoryLabel', () => {
  it('title-cases hyphenated category slugs', () => {
    expect(categoryLabel('framework-guides')).toBe('Framework Guides');
    expect(categoryLabel('process')).toBe('Process');
  });

  it('upper-cases acronyms so `ml` does not read as "Ml"', () => {
    expect(categoryLabel('ml')).toBe('ML');
    expect(categoryLabel('api-design')).toBe('API Design');
  });
});
