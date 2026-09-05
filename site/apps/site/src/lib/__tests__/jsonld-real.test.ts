/**
 * JSON-LD against the real repository — the production guarantee.
 *
 * `src/lib/jsonld.ts` is pure and takes its dates as an argument, and
 * `src/lib/build-dates.ts` will fall back to `researched:` frontmatter (or to
 * nothing) when a process cannot read git history — which is correct for the
 * `make dev` container and would be a silent regression anywhere else. This file
 * is what stops that fallback becoming the shipped behaviour: it runs the real
 * pipeline the way `astro build` does, with git available, and asserts that
 * every corpus and authored route ends up with BOTH dates, resolved to exactly
 * the value the sitemap's `<lastmod>` carries.
 *
 * It also pins the provenance claim end to end — that a researched document's
 * `citation[]` really is its own `sources:` list, one entry per source.
 */

import { describe, expect, it } from 'vitest';

import { buildCorpus } from '../../loaders/lib/pipeline.ts';
import { collectCorpusSources, defaultRepoRoot, repoFileExistsIn } from '../../loaders/lib/sources.ts';
import { pageDateIndex } from '../../loaders/lib/dates.ts';
import { breadcrumbList, docsPageGraph, jsonLdScript, pageDates, type JsonLdNode } from '../jsonld.ts';

const repoRoot = defaultRepoRoot();
const corpus = buildCorpus(collectCorpusSources(repoRoot), {
  repoFileExists: repoFileExistsIn(repoRoot),
});
const index = pageDateIndex();

/** The doc whose 24-source list the acceptance criteria pin. */
const CITED_DOC = 'llm-token-cache-efficiency';

function asJson(node: JsonLdNode): Record<string, unknown> {
  return JSON.parse(JSON.stringify(node)) as Record<string, unknown>;
}

/** The graph a corpus document's page would emit, dates and all. */
function graphFor(doc: (typeof corpus.published)[number]): Record<string, unknown>[] {
  const meta = doc.data.corpus;
  return docsPageGraph({
    route: doc.route,
    title: doc.data.title,
    ...(doc.data.description === undefined ? {} : { description: doc.data.description }),
    corpus: {
      category: meta.category,
      sourceUrl: meta.sourceUrl,
      ...(meta.sources === undefined ? {} : { sources: meta.sources }),
    },
    dates: pageDates({
      route: doc.route,
      ...(meta.researched === undefined ? {} : { researched: meta.researched }),
      index: index.dates,
    }),
  }).map(asJson);
}

describe('every published corpus document', () => {
  it('has documents to check at all', () => {
    expect(corpus.published.length).toBeGreaterThan(50);
  });

  it('resolves both dates from the build’s date map — never the git-less fallback', () => {
    const undated = corpus.published.filter((doc) => {
      const article = graphFor(doc)[0]!;
      return (
        typeof article.datePublished !== 'string' || typeof article.dateModified !== 'string'
      );
    });
    expect(undated.map((doc) => doc.route)).toEqual([]);
  });

  it('carries exactly the dates the sitemap gives the same route', () => {
    const mismatched = corpus.published.filter((doc) => {
      const article = graphFor(doc)[0]!;
      return article.dateModified !== index.dates.get(doc.route);
    });
    expect(mismatched.map((doc) => doc.route)).toEqual([]);
  });

  it('emits a TechArticle followed by a BreadcrumbList, and nothing else', () => {
    for (const doc of corpus.published) {
      expect(graphFor(doc).map((n) => n['@type'])).toEqual(['TechArticle', 'BreadcrumbList']);
    }
  });

  it('cites its own sources one for one, isBasedOn its own repo file', () => {
    for (const doc of corpus.published) {
      const article = graphFor(doc)[0]!;
      const declared = doc.data.corpus.sources ?? [];
      expect(article.citation ?? []).toHaveLength(declared.length);
      expect(article.isBasedOn).toBe(doc.data.corpus.sourceUrl);
      expect(String(article.isBasedOn)).toContain(doc.repoPath);
    }
  });

  it(`gives ${CITED_DOC} a citation for each of its 24 sources`, () => {
    const doc = corpus.published.find((d) => d.data.corpus.slug === CITED_DOC);
    expect(doc, `${CITED_DOC} is not in the published corpus`).toBeDefined();

    const sources = doc!.data.corpus.sources ?? [];
    expect(sources).toHaveLength(24);

    const article = graphFor(doc!)[0]!;
    expect(article.citation).toEqual(sources);
    expect(article.articleSection).toBe('Process');
    expect(article.datePublished).toBe(index.dates.get(doc!.route));
  });
});

describe('every authored (non-corpus) route', () => {
  // Corpus documents are dated from their repo file; everything else — the
  // guides, the generated indexes, the landing page — is dated from its own
  // source file. Both halves have to resolve for the head to be complete.
  const corpusRoutes = new Set(corpus.published.map((doc) => doc.route));
  const authored = [...index.dates.keys()].filter((route) => !corpusRoutes.has(route));

  it('is a real set of routes, not an empty filter result', () => {
    expect(authored.length).toBeGreaterThan(10);
  });

  it('resolves a dateModified for its breadcrumb page too', () => {
    for (const route of authored) {
      expect(pageDates({ route, index: index.dates }).dateModified).toBe(index.dates.get(route));
    }
  });
});

describe('the serialised block', () => {
  it('parses as one JSON document per page, for every published route', () => {
    for (const route of index.dates.keys()) {
      if (route === '/') continue; // the landing page emits the homepage graph instead
      const script = jsonLdScript([breadcrumbList(route, 'Title')]);
      expect(() => JSON.parse(script)).not.toThrow();
    }
  });

  it('never lets a real corpus title break out of the script element', () => {
    for (const doc of corpus.published) {
      const script = jsonLdScript(
        docsPageGraph({
          route: doc.route,
          title: doc.data.title,
          corpus: { category: doc.data.corpus.category, sourceUrl: doc.data.corpus.sourceUrl },
        })
      );
      expect(script).not.toContain('<');
      expect(() => JSON.parse(script)).not.toThrow();
    }
  });
});
