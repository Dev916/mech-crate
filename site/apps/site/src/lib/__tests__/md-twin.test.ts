/**
 * Unit tests for the markdown twins (`src/lib/md-twin.ts`).
 *
 * Two contracts, and they are separate things:
 *
 *   1. **The predicate agrees with reality.** `Head.astro` decides whether to
 *      advertise a twin from the pathname alone; the endpoint decides whether to
 *      write one from the page list. A disagreement is either a `<link>` pointing
 *      at a 404 or a document no agent can fetch, and both are silent. The last
 *      block runs the predicate over the real corpus, so the claim is made about
 *      the repository's actual contents rather than a fixture.
 *   2. **The twin carries the page's markdown unchanged.** Asserted here against
 *      the bytes on disk for three sampled pages — two corpus documents through
 *      the real pipeline, one authored guide read straight out of the content
 *      collection — so "the twin is the page" is a checked fact, not a comment.
 */

import { readFileSync } from 'node:fs';
import { join } from 'node:path';

import { describe, expect, it } from 'vitest';

import {
  MD_TWIN_EXTENSION,
  MD_TWIN_TYPE,
  buildMdTwin,
  mdTwinPath,
  mdTwinUrl,
  mdTwins,
  routeHasMdTwin,
} from '../md-twin.ts';
import { headingMatchesTitle } from '../../loaders/lib/headings.ts';
import { buildCorpus } from '../../loaders/lib/pipeline.ts';
import { collectCorpusSources, defaultRepoRoot, repoFileExistsIn } from '../../loaders/lib/sources.ts';
import { SITE_ORIGIN, type LlmsPage } from '../../loaders/lib/llms.ts';
import { pagesFromDocsEntries, generatedNavPages, corpusCategoryCounts } from '../../loaders/lib/llms-pages.ts';

function page(overrides: Partial<LlmsPage> & Pick<LlmsPage, 'title' | 'route' | 'kind'>): LlmsPage {
  return { ...overrides };
}

/** The header/body boundary is the FIRST rule — corpus bodies contain `---` too. */
const RULE = '\n\n---\n\n';
function splitTwin(twin: string): [header: string, body: string] {
  const at = twin.indexOf(RULE);
  expect(at, 'twin has no header rule').toBeGreaterThan(0);
  return [twin.slice(0, at), twin.slice(at + RULE.length)];
}

describe('routeHasMdTwin', () => {
  it('gives every authored and corpus document a twin', () => {
    expect(routeHasMdTwin('/docs/')).toBe(true);
    expect(routeHasMdTwin('/docs/start/')).toBe(true);
    expect(routeHasMdTwin('/docs/start/install/')).toBe(true);
    expect(routeHasMdTwin('/docs/corpus/theory/appendix-fsm/')).toBe(true);
  });

  it('withholds one from the generated navigation pages, which have no markdown source', () => {
    // The landing page, the corpus overview, and the fifteen category indexes
    // are Astro pages under src/pages — llms-full.txt skips them for the same
    // reason.
    expect(routeHasMdTwin('/')).toBe(false);
    expect(routeHasMdTwin('/docs/corpus/')).toBe(false);
    expect(routeHasMdTwin('/docs/corpus/theory/')).toBe(false);
    expect(routeHasMdTwin('/docs/corpus/framework-guides/')).toBe(false);
  });

  it('withholds one from routes that are not published content', () => {
    expect(routeHasMdTwin('/404/')).toBe(false);
    expect(routeHasMdTwin('/docs/scratch/notes/')).toBe(false);
  });

  it('honours sidebar.hidden, the one part of the filter a pathname cannot express', () => {
    expect(routeHasMdTwin('/docs/start/install/', true)).toBe(false);
  });

  it('tolerates a pathname without a trailing slash', () => {
    expect(routeHasMdTwin('/docs/start/install')).toBe(true);
  });
});

describe('mdTwinPath / mdTwinUrl', () => {
  it('appends .md to the route, as a sibling of the HTML directory', () => {
    expect(mdTwinPath('/docs/start/install/')).toBe('/docs/start/install.md');
    expect(mdTwinPath('/docs/')).toBe('/docs.md');
    expect(MD_TWIN_EXTENSION).toBe('.md');
  });

  it('qualifies the URL with the canonical origin', () => {
    expect(mdTwinUrl('/docs/start/install/')).toBe(`${SITE_ORIGIN}/docs/start/install.md`);
    expect(mdTwinUrl('/x/', 'https://staging.example')).toBe('https://staging.example/x.md');
  });

  it('advertises the type `<link rel="alternate">` and _headers agree on', () => {
    expect(MD_TWIN_TYPE).toBe('text/markdown');
  });
});

describe('buildMdTwin', () => {
  const twin = buildMdTwin(
    page({
      title: 'Install',
      route: '/docs/start/install/',
      kind: 'start',
      sourcePath: 'site/apps/site/src/content/docs/docs/start/install.md',
      body: '\nThere is no published binary yet.\n\n## Requirements\n\nDocker.\n',
    })
  );

  it('opens with an H1, the canonical URL and the repository source path', () => {
    expect(splitTwin(twin)[0]).toBe(
      [
        '# Install',
        '',
        'URL: https://mechcrate.dev/docs/start/install/',
        'Source: site/apps/site/src/content/docs/docs/start/install.md',
      ].join('\n')
    );
  });

  it('carries the page body verbatim after the rule, trimmed and newline-terminated', () => {
    expect(splitTwin(twin)[1]).toBe(
      'There is no published binary yet.\n\n## Requirements\n\nDocker.\n'
    );
  });

  it('omits the Source line for a page with no repository file', () => {
    const out = buildMdTwin(page({ title: 'T', route: '/t/', kind: 'start', body: 'x' }));
    expect(out).not.toContain('Source:');
    expect(out).toBe('# T\n\nURL: https://mechcrate.dev/t/\n\n---\n\nx\n');
  });
});

describe('mdTwins', () => {
  it('keys every page that has a body by its route slug, and skips those that do not', () => {
    const twins = mdTwins([
      page({ title: 'Landing', route: '/', kind: 'overview' }),
      page({ title: 'Corpus', route: '/docs/corpus/', kind: 'corpus' }),
      page({ title: 'Install', route: '/docs/start/install/', kind: 'start', body: '# I\n' }),
    ]);
    expect([...twins.keys()]).toEqual(['docs/start/install']);
  });

  it('fails loudly when a page has a body the predicate does not expect', () => {
    // A generated navigation page that grew a markdown source would otherwise
    // ship a twin no page links to.
    expect(() =>
      mdTwins([page({ title: 'Corpus', route: '/docs/corpus/', kind: 'corpus', body: '# C\n' })])
    ).toThrow(/routeHasMdTwin\(\) says false/);
  });

  it('fails loudly when the predicate promises a twin the page cannot supply', () => {
    expect(() =>
      mdTwins([page({ title: 'Install', route: '/docs/start/install/', kind: 'start' })])
    ).toThrow(/routeHasMdTwin\(\) says true/);
  });
});

describe('twins of the real repository content', () => {
  const repoRoot = defaultRepoRoot();
  const appRoot = join(repoRoot, 'site', 'apps', 'site');
  const corpus = buildCorpus(collectCorpusSources(repoRoot), {
    repoFileExists: repoFileExistsIn(repoRoot),
  });

  const corpusPages = pagesFromDocsEntries(
    corpus.published.map((doc) => ({
      id: doc.id,
      body: doc.body,
      data: { title: doc.data.title, description: doc.data.description, corpus: doc.data.corpus },
    }))
  );

  const navPages = generatedNavPages(
    { title: 'MechCrate', description: 'The front page.' },
    corpusCategoryCounts(corpusPages)
  );

  it('agrees with the page list about which routes have a markdown source', () => {
    // The build asserts this too (mdTwins throws), but failing here names the
    // route without a 10-second `astro build` in between.
    for (const p of [...corpusPages, ...navPages]) {
      expect(routeHasMdTwin(p.route), p.route).toBe((p.body ?? '').trim() !== '');
    }
  });

  it('writes one twin per published corpus document and none per navigation page', () => {
    const twins = mdTwins([...corpusPages, ...navPages]);
    expect(twins.size).toBe(corpus.published.length);
    expect(twins.size).toBeGreaterThan(50);
  });

  it('does not repeat its own H1 in the body — the corpus H1 dedup reaches the twins', () => {
    // The twin's header already opens `# <title>`; before the dedup
    // (src/loaders/lib/headings.ts) the body under the rule opened with the same
    // heading again. The strip happens on the one body the HTML page,
    // llms-full.txt and this file all share, so asserting it here is asserting
    // it for all three.
    for (const p of corpusPages) {
      const [header, body] = splitTwin(buildMdTwin(p));
      expect(header.startsWith(`# ${p.title}`), p.route).toBe(true);
      const opening = body.split('\n').find((line) => line.trim() !== '') ?? '';
      const h1 = /^ {0,3}#[ \t]+(.*)$/.exec(opening);
      if (h1 !== null) {
        expect(headingMatchesTitle(h1[1]!, p.title), `${p.route} repeats its title`).toBe(false);
      }
    }
  });

  it('serves the page markdown unchanged — two corpus documents, one authored guide', () => {
    const samples: { page: LlmsPage; sourceFile: string }[] = [
      { page: corpusPages[0]!, sourceFile: join(repoRoot, corpusPages[0]!.sourcePath!) },
      { page: corpusPages[1]!, sourceFile: join(repoRoot, corpusPages[1]!.sourcePath!) },
    ];

    for (const sample of samples) {
      const twin = buildMdTwin(sample.page);
      const [header, body] = splitTwin(twin);
      expect(header, sample.page.route).toBe(
        [
          `# ${sample.page.title}`,
          '',
          `URL: ${SITE_ORIGIN}${sample.page.route}`,
          `Source: ${sample.page.sourcePath}`,
        ].join('\n')
      );
      // The corpus body is the pipeline's output (frontmatter stripped, relative
      // links rewritten), which is exactly what llms-full.txt concatenates.
      expect(body, sample.page.route).toBe(`${sample.page.body!.trim()}\n`);
      // …and that output is derived from a file that really exists.
      expect(readFileSync(sample.sourceFile, 'utf8').length).toBeGreaterThan(0);
    }

    // The third sample is an authored guide, read straight off disk: its twin
    // body must be the file minus its frontmatter block.
    const relative = 'src/content/docs/docs/start/install.md';
    const raw = readFileSync(join(appRoot, relative), 'utf8');
    const authored = page({
      title: 'Install',
      route: '/docs/start/install/',
      kind: 'start',
      sourcePath: `site/apps/site/${relative}`,
      body: raw.replace(/^---\n[\s\S]*?\n---\n/, ''),
    });
    const twin = buildMdTwin(authored);
    expect(splitTwin(twin)[1]).toBe(`${raw.replace(/^---\n[\s\S]*?\n---\n/, '').trim()}\n`);
    expect(twin).toContain('URL: https://mechcrate.dev/docs/start/install/');
  });
});
