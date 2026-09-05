/**
 * Unit tests for the social-card mapping (`src/lib/og.ts`).
 *
 * The card endpoint (`src/pages/og/[...route].ts`) is a thin wrapper: it hands
 * `ogCards()` the page list from `collectLlmsPages()` and turns the returned map
 * straight into `getStaticPaths`. The `<head>` tags come from `ogCardUrl()` and
 * `routeHasOgCard()`. So asserting on those functions IS asserting that every
 * published page has a card route and that no page advertises one it does not
 * have — the two halves cannot disagree because they are the same function.
 *
 * The last block runs the real corpus pipeline, the same code `astro build`
 * runs, so the coverage claim is made about the repository's actual documents
 * rather than a fixture.
 */

import { describe, expect, it } from 'vitest';

import { buildCorpus } from '../../loaders/lib/pipeline.ts';
import {
  collectCorpusSources,
  defaultRepoRoot,
  repoFileExistsIn,
} from '../../loaders/lib/sources.ts';
import { SITE_ORIGIN, type LlmsPage } from '../../loaders/lib/llms.ts';
import {
  corpusCategoryCounts,
  generatedNavPages,
  pagesFromDocsEntries,
  type DocsEntryLike,
} from '../../loaders/lib/llms-pages.ts';
import {
  OG_CARD_HEIGHT,
  OG_CARD_TYPE,
  OG_CARD_WIDTH,
  OG_PALETTE,
  ogCard,
  ogCardLabel,
  ogCardPath,
  ogCardSlug,
  ogCardUrl,
  ogCards,
  routeHasOgCard,
} from '../og.ts';

function page(overrides: Partial<LlmsPage> & Pick<LlmsPage, 'title' | 'route' | 'kind'>): LlmsPage {
  return { ...overrides };
}

describe('card dimensions', () => {
  it('are the 1200×630 PNG every consumer expects', () => {
    expect([OG_CARD_WIDTH, OG_CARD_HEIGHT]).toEqual([1200, 630]);
    expect(OG_CARD_TYPE).toBe('image/png');
  });
});

describe('ogCardSlug', () => {
  it('trims the slashes off a route', () => {
    expect(ogCardSlug('/docs/start/install/')).toBe('docs/start/install');
  });

  it('names the landing page card `index`', () => {
    expect(ogCardSlug('/')).toBe('index');
    expect(ogCardSlug('')).toBe('index');
  });

  it('reads the same route with or without a trailing slash', () => {
    expect(ogCardSlug('/docs/corpus/theory/appendix-fsm')).toBe(
      ogCardSlug('/docs/corpus/theory/appendix-fsm/')
    );
  });

  it('gives a group index and its children distinct, coexisting files', () => {
    // `og/docs.png` and `og/docs/start.png` — a file and a directory of the
    // same name, which is why the library's default `/index` collapsing is
    // overridden in the endpoint.
    expect(ogCardPath('/docs/')).toBe('/og/docs.png');
    expect(ogCardPath('/docs/start/')).toBe('/og/docs/start.png');
  });
});

describe('ogCardUrl', () => {
  it('is absolute — a relative og:image is not resolvable by a scraper', () => {
    expect(ogCardUrl('/docs/start/')).toBe('https://mechcrate.dev/og/docs/start.png');
    expect(ogCardUrl('/docs/start/').startsWith(SITE_ORIGIN)).toBe(true);
  });

  it('points the landing page at its own card', () => {
    expect(ogCardUrl('/')).toBe('https://mechcrate.dev/og/index.png');
  });

  it('takes an origin override for a preview deploy', () => {
    expect(ogCardUrl('/docs/', 'https://preview.example')).toBe(
      'https://preview.example/og/docs.png'
    );
  });
});

describe('routeHasOgCard', () => {
  it('is true for every published navigation group', () => {
    for (const route of [
      '/',
      '/docs/',
      '/docs/start/install/',
      '/docs/framework/router/',
      '/docs/ai/mcp-server/',
      '/docs/project/license/',
      '/docs/corpus/',
      '/docs/corpus/concurrency/',
      '/docs/corpus/concurrency/appendix-actor-model/',
    ]) {
      expect(routeHasOgCard(route), route).toBe(true);
    }
  });

  it('is false for a sidebar-hidden page — the 404 route', () => {
    expect(routeHasOgCard('/404/', true)).toBe(false);
  });

  it('is false for a route outside the known navigation groups', () => {
    expect(routeHasOgCard('/docs/scratch/notes/')).toBe(false);
  });
});

describe('ogCardLabel', () => {
  it('names the domain on the landing card, whose title already says MechCrate', () => {
    expect(ogCardLabel(page({ title: 'MechCrate', route: '/', kind: 'overview' }))).toBe(
      'mechcrate.dev'
    );
  });

  it('names the sidebar group for an authored page', () => {
    expect(ogCardLabel(page({ title: 'Install', route: '/docs/start/install/', kind: 'start' }))).toBe(
      'MechCrate · Start'
    );
    expect(ogCardLabel(page({ title: 'MCP', route: '/docs/ai/mcp-server/', kind: 'ai' }))).toBe(
      'MechCrate · AI Layer'
    );
  });

  it('names the category for a corpus document, humanised', () => {
    expect(
      ogCardLabel(
        page({
          title: 'Actor model',
          route: '/docs/corpus/framework-guides/x/',
          kind: 'corpus',
          category: 'framework-guides',
        })
      )
    ).toBe('MechCrate · Framework Guides');
  });

  it('falls back to the corpus label for a corpus page with no category', () => {
    expect(
      ogCardLabel(page({ title: 'The agent corpus', route: '/docs/corpus/', kind: 'corpus' }))
    ).toBe('MechCrate · Techniques Corpus');
  });

  it('drops the section when the title already says it', () => {
    // `/docs/start/` is titled "Start" and sits in the Start group — the kicker
    // would otherwise read `MechCrate · Start` under a heading reading `Start`.
    expect(ogCardLabel(page({ title: 'Start', route: '/docs/start/', kind: 'start' }))).toBe(
      'MechCrate'
    );
    expect(
      ogCardLabel(
        page({ title: 'Concurrency — Techniques Corpus', route: '/docs/corpus/x/', kind: 'corpus' })
      )
    ).toBe('MechCrate');
  });
});

describe('ogCard', () => {
  const long = page({
    title:
      'Tries, Radix Trees, and Trie-Path Dispatch: Replacing Conditional Chains with Structural Lookup',
    route: '/docs/corpus/patterns/tries/',
    kind: 'corpus',
    category: 'patterns',
    description: 'x'.repeat(400),
  });

  it('truncates a title that would lay off the bottom of the canvas', () => {
    const card = ogCard(long);
    expect(card.title.length).toBeLessThanOrEqual(85);
    expect(card.title.endsWith('…')).toBe(true);
  });

  it('truncates the summary too', () => {
    expect(ogCard(long).description.length).toBeLessThanOrEqual(151);
  });

  it('keeps the untruncated title as the alt text', () => {
    expect(ogCard(long).alt).toBe(long.title);
  });

  it('leaves the summary empty when the page declares no description', () => {
    const card = ogCard(page({ title: 'Installing mx', route: '/docs/start/x/', kind: 'start' }));
    expect(card.description).toBe('');
    expect(card.label).toBe('MechCrate · Start');
  });
});

describe('ogCards', () => {
  it('emits exactly one card per page, addressable by the page route', () => {
    const pages = [
      page({ title: 'Home', route: '/', kind: 'overview' }),
      page({ title: 'Docs', route: '/docs/', kind: 'overview' }),
      page({ title: 'Install', route: '/docs/start/install/', kind: 'start' }),
    ];

    const cards = ogCards(pages);

    expect(cards.size).toBe(pages.length);
    for (const p of pages) {
      expect(cards.has(ogCardSlug(p.route)), p.route).toBe(true);
      expect(`/og/${[...cards.keys()].find((k) => k === ogCardSlug(p.route))}.png`).toBe(
        ogCardPath(p.route)
      );
    }
  });

  it('refuses a slug collision rather than dropping one page silently', () => {
    expect(() =>
      ogCards([
        page({ title: 'A', route: '/docs/start/', kind: 'start' }),
        page({ title: 'B', route: '/docs/start', kind: 'start' }),
      ])
    ).toThrow(/both map to \/og\/docs\/start\.png/);
  });
});

describe('the palette', () => {
  it('is RGB triples in range, as astro-og-canvas requires', () => {
    for (const [name, rgb] of Object.entries(OG_PALETTE)) {
      expect(rgb.length, name).toBe(3);
      for (const channel of rgb) {
        expect(Number.isInteger(channel), name).toBe(true);
        expect(channel, name).toBeGreaterThanOrEqual(0);
        expect(channel, name).toBeLessThanOrEqual(255);
      }
    }
  });
});

describe('coverage of the real published page set', () => {
  // Same construction as `collectLlmsPages()`, with the corpus half produced by
  // the real pipeline and the authored half by the files on disk, so this is the
  // set the endpoint actually receives at build time.
  const repoRoot = defaultRepoRoot();
  const corpus = buildCorpus(collectCorpusSources(repoRoot), {
    repoFileExists: repoFileExistsIn(repoRoot),
  });

  const entries: DocsEntryLike[] = corpus.published.map((doc) => ({
    id: doc.id,
    data: {
      title: doc.data.title,
      ...(doc.data.description === undefined ? {} : { description: doc.data.description }),
      corpus: doc.data.corpus,
    },
  }));
  entries.push({
    id: '404',
    data: { title: 'Not found', sidebar: { hidden: true } },
  });

  const docPages = pagesFromDocsEntries(entries);
  const pages = [
    ...docPages,
    ...generatedNavPages(
      { title: 'MechCrate', description: 'mx' },
      corpusCategoryCounts(docPages)
    ),
  ];
  const cards = ogCards(pages);

  it('covers the whole corpus and then some', () => {
    expect(pages.length).toBeGreaterThan(60);
  });

  it('gives every published route its own card, with no collisions', () => {
    expect(cards.size).toBe(pages.length);
    for (const p of pages) {
      expect(cards.get(ogCardSlug(p.route))?.route, p.route).toBe(p.route);
    }
  });

  it('advertises a card on exactly the routes that have one', () => {
    for (const p of pages) {
      expect(routeHasOgCard(p.route), p.route).toBe(true);
      expect(ogCardUrl(p.route)).toBe(`${SITE_ORIGIN}${ogCardPath(p.route)}`);
    }
  });

  it('leaves the held-back 404 route without a card', () => {
    expect(cards.has('404')).toBe(false);
    expect(routeHasOgCard('/404/', true)).toBe(false);
  });

  it('gives every card a title and a kicker naming the site', () => {
    for (const [slug, card] of cards) {
      expect(card.title.length, slug).toBeGreaterThan(0);
      expect(/^(MechCrate|mechcrate\.dev)\b/.test(card.label), `${slug}: ${card.label}`).toBe(true);
    }
  });
});
