/**
 * Unit tests for the JSON-LD generator (`src/lib/jsonld.ts`).
 *
 * Structural assertions rather than a JSON-Schema validator: schema.org has no
 * schema to validate against, so the only thing a validator could check is the
 * shape *we* decided on — which is what these assertions state directly, without
 * a dependency. What matters is that the properties Google's structured-data
 * documentation calls required are present and well-formed (BreadcrumbList:
 * every ListItem needs `position`, `name` and a resolvable `item`), that the
 * corpus provenance survives the trip into `citation`/`isBasedOn`, and that the
 * serialised block cannot break out of its own `<script>` element.
 *
 * The real-data half — real corpus documents, real git dates — is
 * `jsonld-real.test.ts`.
 */

import { describe, expect, it } from 'vitest';

import {
  APPLICATION_ID,
  ORGANIZATION_ID,
  SCHEMA_CONTEXT,
  WEBSITE_ID,
  breadcrumbList,
  breadcrumbTrail,
  docsPageGraph,
  homepageGraph,
  jsonLdScript,
  normalizePath,
  organization,
  pageDates,
  softwareApplication,
  techArticle,
  website,
  type JsonLdNode,
} from '../jsonld.ts';

const ORIGIN = 'https://mechcrate.dev';

const LOGO = { url: `${ORIGIN}/_astro/mechcrate-logo.hash.png`, width: 512, height: 512 };

/** Re-read a node through JSON, which is how a consumer actually sees it. */
function asJson(node: JsonLdNode): Record<string, unknown> {
  return JSON.parse(JSON.stringify(node)) as Record<string, unknown>;
}

/** The `@graph` of a serialised block, parsed. */
function graphOf(script: string): Record<string, unknown>[] {
  const parsed = JSON.parse(script) as { '@context': string; '@graph': Record<string, unknown>[] };
  expect(parsed['@context']).toBe(SCHEMA_CONTEXT);
  return parsed['@graph'];
}

describe('normalizePath', () => {
  it('gives every route a leading and a trailing slash', () => {
    expect(normalizePath('/docs/start')).toBe('/docs/start/');
    expect(normalizePath('docs/start/')).toBe('/docs/start/');
    expect(normalizePath('/')).toBe('/');
  });
});

describe('breadcrumbTrail', () => {
  it('walks Home → Documentation → group → page for an authored doc', () => {
    expect(breadcrumbTrail('/docs/start/install/', 'Install mx')).toEqual([
      { name: 'Home', url: `${ORIGIN}/` },
      { name: 'Documentation', url: `${ORIGIN}/docs/` },
      { name: 'Start', url: `${ORIGIN}/docs/start/` },
      { name: 'Install mx', url: `${ORIGIN}/docs/start/install/` },
    ]);
  });

  it('adds the category level for a corpus document', () => {
    expect(breadcrumbTrail('/docs/corpus/framework-guides/astro/', 'Astro')).toEqual([
      { name: 'Home', url: `${ORIGIN}/` },
      { name: 'Documentation', url: `${ORIGIN}/docs/` },
      { name: 'Techniques Corpus', url: `${ORIGIN}/docs/corpus/` },
      { name: 'Framework Guides', url: `${ORIGIN}/docs/corpus/framework-guides/` },
      { name: 'Astro', url: `${ORIGIN}/docs/corpus/framework-guides/astro/` },
    ]);
  });

  it('names groups the way the sidebar names them, acronyms included', () => {
    expect(breadcrumbTrail('/docs/ai/mcp-server/', 'MCP server')[2]).toEqual({
      name: 'AI Layer',
      url: `${ORIGIN}/docs/ai/`,
    });
    expect(breadcrumbTrail('/docs/corpus/ml/embeddings/', 'Embeddings')[3]?.name).toBe('ML');
  });

  it('stops at the page itself — a group index is its own last crumb', () => {
    expect(breadcrumbTrail('/docs/framework/', 'Framework')).toEqual([
      { name: 'Home', url: `${ORIGIN}/` },
      { name: 'Documentation', url: `${ORIGIN}/docs/` },
      { name: 'Framework', url: `${ORIGIN}/docs/framework/` },
    ]);
  });

  it('accepts a route with no trailing slash, as Astro.url.pathname may give', () => {
    expect(breadcrumbTrail('/docs/project', 'Project').at(-1)?.url).toBe(`${ORIGIN}/docs/project/`);
  });
});

describe('breadcrumbList', () => {
  const node = asJson(breadcrumbList('/docs/corpus/theory/fsm/', 'Finite state machines'));

  it('is a BreadcrumbList with a page-scoped @id', () => {
    expect(node['@type']).toBe('BreadcrumbList');
    expect(node['@id']).toBe(`${ORIGIN}/docs/corpus/theory/fsm/#breadcrumbs`);
  });

  it('gives every ListItem the position, name and item Google requires', () => {
    const items = node.itemListElement as Record<string, unknown>[];
    expect(items.length).toBe(5);
    items.forEach((item, i) => {
      expect(item['@type']).toBe('ListItem');
      expect(item.position).toBe(i + 1);
      expect(typeof item.name).toBe('string');
      expect(item.name).not.toBe('');
      expect(String(item.item).startsWith('https://')).toBe(true);
    });
  });
});

describe('techArticle', () => {
  const input = {
    route: '/docs/corpus/process/llm-token-cache-efficiency/',
    title: 'LLM Token & Cache Efficiency',
    description: 'Cache and token mechanics, from provider docs.',
    category: 'process',
    sources: ['https://example.com/a', 'https://example.com/b'],
    sourceUrl: 'https://github.com/Dev916/mech-crate/blob/main/docs/development/x.md',
    datePublished: '2026-08-14T00:00:00.000Z',
    dateModified: '2026-08-14T00:00:00.000Z',
  };
  const node = asJson(techArticle(input));

  it('carries the headline, description and canonical URL', () => {
    expect(node['@type']).toBe('TechArticle');
    expect(node.headline).toBe(input.title);
    expect(node.description).toBe(input.description);
    expect(node.url).toBe(`${ORIGIN}${input.route}`);
    expect(node.mainEntityOfPage).toEqual({ '@type': 'WebPage', '@id': `${ORIGIN}${input.route}` });
  });

  it('uses the human category label as articleSection', () => {
    expect(node.articleSection).toBe('Process');
    expect(asJson(techArticle({ ...input, category: 'framework-guides' })).articleSection).toBe(
      'Framework Guides'
    );
  });

  it('publishes the source list as citation[] and the repo file as isBasedOn', () => {
    expect(node.citation).toEqual(input.sources);
    expect(node.isBasedOn).toBe(input.sourceUrl);
  });

  it('keeps citation[] one-to-one with sources, dropping only blank entries', () => {
    const many = Array.from({ length: 24 }, (_, i) => `https://example.com/${i}`);
    expect(asJson(techArticle({ ...input, sources: many })).citation).toHaveLength(24);
    expect(asJson(techArticle({ ...input, sources: [' ', ''] })).citation).toBeUndefined();
    expect(asJson(techArticle({ ...input, sources: undefined })).citation).toBeUndefined();
  });

  it('references the site Organization as publisher rather than restating it', () => {
    expect((node.publisher as Record<string, unknown>)['@id']).toBe(ORGANIZATION_ID);
    expect((node.isPartOf as Record<string, unknown>)['@id']).toBe(WEBSITE_ID);
  });

  it('omits dates it was not given rather than inventing them', () => {
    const undated = asJson(
      techArticle({ ...input, datePublished: undefined, dateModified: undefined })
    );
    expect(undated.datePublished).toBeUndefined();
    expect(undated.dateModified).toBeUndefined();
    expect(node.datePublished).toBe(input.datePublished);
    expect(node.dateModified).toBe(input.dateModified);
  });
});

describe('pageDates', () => {
  const index = new Map([['/docs/corpus/ml/x/', '2026-08-14T00:00:00.000Z']]);

  it('takes both dates from Task 2’s map, so JSON-LD and <lastmod> agree', () => {
    expect(pageDates({ route: '/docs/corpus/ml/x/', index })).toEqual({
      datePublished: '2026-08-14T00:00:00.000Z',
      dateModified: '2026-08-14T00:00:00.000Z',
    });
  });

  it('normalises the route before the lookup', () => {
    expect(pageDates({ route: '/docs/corpus/ml/x', index }).dateModified).toBe(
      '2026-08-14T00:00:00.000Z'
    );
  });

  it('falls back to `researched:` for datePublished when the map is unavailable', () => {
    expect(pageDates({ route: '/docs/corpus/ml/y/', researched: '2026-08-14' })).toEqual({
      datePublished: '2026-08-14',
    });
  });

  it('emits nothing rather than a build-stamped date when neither source exists', () => {
    expect(pageDates({ route: '/docs/start/' })).toEqual({});
    expect(pageDates({ route: '/docs/start/', researched: 'summer 2026' })).toEqual({});
  });
});

describe('the homepage entities', () => {
  const nodes = homepageGraph(LOGO).map(asJson);

  it('is exactly Organization, WebSite and SoftwareApplication', () => {
    expect(nodes.map((n) => n['@type'])).toEqual([
      'Organization',
      'WebSite',
      'SoftwareApplication',
    ]);
  });

  it('gives the Organization an absolute raster logo and the GitHub sameAs', () => {
    const org = asJson(organization(LOGO));
    expect(org['@id']).toBe(ORGANIZATION_ID);
    expect(org.logo).toEqual({
      '@type': 'ImageObject',
      url: LOGO.url,
      width: 512,
      height: 512,
      caption: 'MechCrate',
    });
    expect(org.sameAs).toEqual(['https://github.com/Dev916/mech-crate']);
  });

  it('publishes the WebSite under the Organization and declares no SearchAction', () => {
    const site = asJson(website());
    expect(site['@id']).toBe(WEBSITE_ID);
    expect(site.publisher).toEqual({ '@id': ORGANIZATION_ID });
    expect(site.potentialAction).toBeUndefined();
  });

  it('describes mx as a free, dual-licensed macOS/Linux developer tool', () => {
    const app = asJson(softwareApplication());
    expect(app['@id']).toBe(APPLICATION_ID);
    expect(app.name).toBe('mx');
    expect(app.applicationCategory).toBe('DeveloperApplication');
    expect(app.operatingSystem).toBe('macOS, Linux');
    expect(app.codeRepository).toBe('https://github.com/Dev916/mech-crate');
    expect(app.license).toEqual([
      'https://github.com/Dev916/mech-crate/blob/main/LICENSE-APACHE',
      'https://github.com/Dev916/mech-crate/blob/main/LICENSE-MIT',
    ]);
    expect(app.offers).toEqual({ '@type': 'Offer', price: '0', priceCurrency: 'USD' });
    expect(app.isAccessibleForFree).toBe(true);
  });

  it('lets every publisher reference resolve inside the graph it ships with', () => {
    const ids = new Set(nodes.map((n) => n['@id']));
    expect(ids.has(ORGANIZATION_ID)).toBe(true);
    for (const node of nodes) {
      const publisher = node.publisher as { '@id'?: string } | undefined;
      if (publisher?.['@id']) expect(ids.has(publisher['@id'])).toBe(true);
    }
  });
});

describe('docsPageGraph', () => {
  it('gives an authored page a BreadcrumbList and nothing else', () => {
    const nodes = docsPageGraph({ route: '/docs/start/install/', title: 'Install mx' });
    expect(nodes.map((n) => n['@type'])).toEqual(['BreadcrumbList']);
  });

  it('puts the TechArticle ahead of the breadcrumbs on a corpus page', () => {
    const nodes = docsPageGraph({
      route: '/docs/corpus/process/x/',
      title: 'X',
      description: 'about x',
      corpus: { category: 'process', sourceUrl: 'https://github.com/o/r/blob/main/x.md' },
      dates: { datePublished: '2026-01-01T00:00:00.000Z' },
    });
    expect(nodes.map((n) => n['@type'])).toEqual(['TechArticle', 'BreadcrumbList']);
    expect(asJson(nodes[0]!).datePublished).toBe('2026-01-01T00:00:00.000Z');
  });
});

describe('jsonLdScript', () => {
  it('wraps the page’s entities in one @context/@graph document', () => {
    const graph = graphOf(jsonLdScript(homepageGraph(LOGO)));
    expect(graph.map((n) => n['@type'])).toEqual([
      'Organization',
      'WebSite',
      'SoftwareApplication',
    ]);
  });

  it('escapes `<` so a title containing `</script>` cannot close the block', () => {
    const script = jsonLdScript([breadcrumbList('/docs/start/x/', 'Nasty </script><img> title')]);
    expect(script).not.toContain('<');
    expect(script).toContain('\\u003c');
    const items = graphOf(script)[0]!.itemListElement as Record<string, unknown>[];
    expect(items.at(-1)!.name).toBe('Nasty </script><img> title');
  });

  it('escapes the JavaScript line terminators too, and still round-trips', () => {
    const title = 'a\u2028b\u2029c';
    const script = jsonLdScript([breadcrumbList('/docs/start/x/', title)]);
    expect(script).not.toContain('\u2028');
    expect(script).not.toContain('\u2029');
    expect(script).toContain('\\u2028');
    const items = graphOf(script)[0]!.itemListElement as Record<string, unknown>[];
    expect(items.at(-1)!.name).toBe(title);
  });

  it('refuses an empty graph rather than emitting invalid structured data', () => {
    expect(() => jsonLdScript([])).toThrow(/empty @graph/);
  });
});
