/**
 * Unit tests for the `llms.txt` / `llms-full.txt` generators.
 *
 * The generators are pure (`src/loaders/lib/llms.ts`, `lib/llms-pages.ts`), so
 * everything below runs without a build: grouping and section order, URL
 * construction, the summary fallback chain, the separator format, corpus
 * inclusion, and the guarantee that a held-back doc reaches neither file.
 *
 * The final block runs the real corpus pipeline — the same code `astro build`
 * runs — so the exclusion claim is made about the actual repository contents
 * rather than a fixture.
 */

import { describe, expect, it } from 'vitest';

import { buildCorpus } from '../lib/pipeline.ts';
import {
  collectCorpusSources,
  defaultRepoRoot,
  repoFileExistsIn,
} from '../lib/sources.ts';
import {
  SITE_ORIGIN,
  absoluteUrl,
  buildLlmsFullTxt,
  buildLlmsTxt,
  classifyDocId,
  docSeparator,
  firstParagraph,
  groupPages,
  routeFromDocId,
  summaryFor,
  truncateSummary,
  type LlmsPage,
} from '../lib/llms.ts';
import {
  APP_REPO_PREFIX,
  corpusCategoryCounts,
  generatedNavPages,
  pagesFromDocsEntries,
  type DocsEntryLike,
} from '../lib/llms-pages.ts';

function page(overrides: Partial<LlmsPage> & Pick<LlmsPage, 'title' | 'route' | 'kind'>): LlmsPage {
  return { ...overrides };
}

describe('routeFromDocId', () => {
  it('turns a collection id into a public route', () => {
    expect(routeFromDocId('docs/start/install')).toBe('/docs/start/install/');
  });

  it('collapses a trailing index segment into its directory', () => {
    expect(routeFromDocId('docs/start/index')).toBe('/docs/start/');
    expect(routeFromDocId('docs/index')).toBe('/docs/');
  });

  it('maps the collection root to the site root', () => {
    expect(routeFromDocId('')).toBe('/');
    expect(routeFromDocId('index')).toBe('/');
  });
});

describe('classifyDocId', () => {
  it('files each authored page under its navigation group', () => {
    expect(classifyDocId('docs/start/install')).toBe('start');
    expect(classifyDocId('docs/framework/router')).toBe('framework');
    expect(classifyDocId('docs/ai/mcp-server')).toBe('ai');
    expect(classifyDocId('docs/project/license')).toBe('project');
    expect(classifyDocId('docs/corpus/theory/appendix-fsm')).toBe('corpus');
  });

  it("files a group's own landing page in that group, not the overview", () => {
    // Astro strips the `index` segment, so `docs/ai/index.mdx` arrives as `docs/ai`.
    expect(classifyDocId('docs/ai')).toBe('ai');
    expect(classifyDocId('docs/start/index')).toBe('start');
    expect(classifyDocId('docs/project')).toBe('project');
  });

  it('files the docs root under Overview', () => {
    expect(classifyDocId('docs')).toBe('overview');
    expect(classifyDocId('docs/index')).toBe('overview');
  });

  it('rejects pages that are not published content', () => {
    expect(classifyDocId('404')).toBeNull();
    expect(classifyDocId('docs/scratch/notes')).toBeNull();
  });
});

describe('absoluteUrl', () => {
  it('prefixes the canonical origin', () => {
    expect(absoluteUrl('/docs/start/')).toBe('https://mechcrate.dev/docs/start/');
    expect(SITE_ORIGIN).toBe('https://mechcrate.dev');
  });

  it('accepts an origin override and a route without a leading slash', () => {
    expect(absoluteUrl('llms.txt', 'https://example.test')).toBe('https://example.test/llms.txt');
  });
});

describe('truncateSummary', () => {
  it('passes short text through, collapsed to one line', () => {
    expect(truncateSummary('one\n  two   three')).toBe('one two three');
  });

  it('elides on a word boundary and marks the elision', () => {
    const long = `${'alpha '.repeat(60)}omega`;
    const out = truncateSummary(long, 40);
    expect(out.length).toBeLessThanOrEqual(41);
    expect(out.endsWith('…')).toBe(true);
    expect(out).not.toContain('omega');
    expect(out.slice(0, -1).trim().endsWith('alpha')).toBe(true);
  });

  it('does not elide text exactly at the limit', () => {
    const exact = 'x'.repeat(30);
    expect(truncateSummary(exact, 30)).toBe(exact);
  });
});

describe('firstParagraph', () => {
  it('returns the first prose line, skipping the title heading', () => {
    expect(firstParagraph('# Title\n\nThe first real sentence.\n')).toBe(
      'The first real sentence.'
    );
  });

  it('skips fenced code, asides, tables, lists, blockquotes, HTML and MDX imports', () => {
    const body = [
      '---',
      '# Heading',
      "import Thing from '@/components/Thing.astro';",
      '<Thing name="x" />',
      ':::note',
      '> quoted',
      '- a bullet',
      '1. numbered',
      '| a | b |',
      '```ts',
      'const notProse = 1;',
      '```',
      '',
      'Actual prose at last.',
    ].join('\n');
    expect(firstParagraph(body)).toBe('Actual prose at last.');
  });

  it('strips inline markdown so the summary reads as a sentence', () => {
    expect(firstParagraph('**Bold** text with `code` and a [link](https://x.test).')).toBe(
      'Bold text with code and a link.'
    );
  });

  it('returns undefined when there is no prose at all', () => {
    expect(firstParagraph('# Only\n\n## Headings\n')).toBeUndefined();
    expect(firstParagraph('')).toBeUndefined();
  });
});

describe('summaryFor', () => {
  const base = { title: 'T', route: '/t/', kind: 'start' } as const;

  it('prefers the declared description', () => {
    expect(
      summaryFor(page({ ...base, description: 'Declared.', body: '# H\n\nDerived.' }))
    ).toBe('Declared.');
  });

  it('falls back to the first paragraph of the body', () => {
    expect(summaryFor(page({ ...base, body: '# H\n\nDerived.' }))).toBe('Derived.');
  });

  it('falls back past an empty description', () => {
    expect(summaryFor(page({ ...base, description: '   ', body: '# H\n\nDerived.' }))).toBe(
      'Derived.'
    );
  });

  it('returns undefined when neither source yields anything', () => {
    expect(summaryFor(page({ ...base }))).toBeUndefined();
    expect(summaryFor(page({ ...base, body: '# Heading only' }))).toBeUndefined();
  });

  it('truncates a paragraph-length corpus summary', () => {
    const summary = summaryFor(page({ ...base, description: 'word '.repeat(200) }), 80);
    expect(summary!.length).toBeLessThanOrEqual(81);
    expect(summary!.endsWith('…')).toBe(true);
  });
});

describe('groupPages', () => {
  const pages: LlmsPage[] = [
    page({ title: 'License', route: '/docs/project/license/', kind: 'project', order: 4 }),
    page({ title: 'Theory doc', route: '/docs/corpus/theory/a/', kind: 'corpus', category: 'theory' }),
    page({ title: 'Landing', route: '/', kind: 'overview', order: -1 }),
    page({ title: 'Corpus overview', route: '/docs/corpus/', kind: 'corpus', order: -1 }),
    page({ title: 'API doc', route: '/docs/corpus/api-design/a/', kind: 'corpus', category: 'api-design' }),
    page({ title: 'Install', route: '/docs/start/install/', kind: 'start', order: 2 }),
    page({ title: 'Start', route: '/docs/start/', kind: 'start', order: 1 }),
    page({ title: 'Router', route: '/docs/framework/router/', kind: 'framework', order: 2 }),
    page({ title: 'MCP server', route: '/docs/ai/mcp-server/', kind: 'ai', order: 2 }),
  ];

  it('orders sections Overview → Start → Framework → AI Layer → Corpus → Project', () => {
    expect(groupPages(pages).map((s) => s.heading)).toEqual([
      'Overview',
      'Start',
      'Framework',
      'AI Layer',
      'Techniques Corpus',
      'Corpus: API Design',
      'Corpus: Theory',
      'Project',
    ]);
  });

  it('sorts within a section by sidebar order, then title', () => {
    const start = groupPages(pages).find((s) => s.heading === 'Start')!;
    expect(start.pages.map((p) => p.title)).toEqual(['Start', 'Install']);
  });

  it('keeps corpus navigation pages out of the per-category sections', () => {
    const sections = groupPages(pages);
    const nav = sections.find((s) => s.heading === 'Techniques Corpus')!;
    expect(nav.pages.map((p) => p.route)).toEqual(['/docs/corpus/']);
    expect(sections.find((s) => s.heading === 'Corpus: Theory')!.pages).toHaveLength(1);
  });

  it('drops empty sections', () => {
    const headings = groupPages([page({ title: 'Only', route: '/docs/start/', kind: 'start' })]).map(
      (s) => s.heading
    );
    expect(headings).toEqual(['Start']);
  });
});

describe('buildLlmsTxt', () => {
  const text = buildLlmsTxt({
    pages: [
      page({ title: 'Landing', route: '/', description: 'The front page.', kind: 'overview' }),
      page({ title: 'Install', route: '/docs/start/install/', description: 'How to install.', kind: 'start' }),
      page({ title: 'No summary', route: '/docs/start/bare/', kind: 'start' }),
      page({
        title: 'Appendix: FSM',
        route: '/docs/corpus/patterns/appendix-fsm/',
        description: 'State machines.',
        kind: 'corpus',
        category: 'patterns',
      }),
    ],
  });

  it('opens with the site title as an H1 and a one-line blockquote description', () => {
    const lines = text.split('\n');
    expect(lines[0]).toBe('# MechCrate');
    expect(lines[1]).toBe('');
    expect(lines[2]!.startsWith('> ')).toBe(true);
    expect(lines[2]).not.toContain('\n');
  });

  it('points at the full-text companion', () => {
    expect(text).toContain('https://mechcrate.dev/llms-full.txt');
  });

  it('uses ## for every group heading', () => {
    expect(text.match(/^## .*$/gm)).toEqual(['## Overview', '## Start', '## Corpus: Patterns']);
  });

  it('writes each page as a bullet with an absolute URL and a one-line summary', () => {
    expect(text).toContain('- [Install](https://mechcrate.dev/docs/start/install/): How to install.');
    expect(text).toContain(
      '- [Appendix: FSM](https://mechcrate.dev/docs/corpus/patterns/appendix-fsm/): State machines.'
    );
  });

  it('omits the trailing note for a page with no summary', () => {
    expect(text).toContain('- [No summary](https://mechcrate.dev/docs/start/bare/)\n');
  });

  it('honours an origin override', () => {
    const staged = buildLlmsTxt({
      pages: [page({ title: 'X', route: '/x/', kind: 'start' })],
      origin: 'https://staging.example',
    });
    expect(staged).toContain('- [X](https://staging.example/x/)');
  });

  it('ends with a single newline', () => {
    expect(text.endsWith('\n')).toBe(true);
    expect(text.endsWith('\n\n')).toBe(false);
  });
});

describe('docSeparator', () => {
  it('emits the title, canonical URL and repo source path between rules', () => {
    expect(
      docSeparator({
        title: 'Appendix: FSM',
        url: 'https://mechcrate.dev/docs/corpus/patterns/appendix-fsm/',
        source: 'docs/development/appendix-fsm.md',
      })
    ).toBe(
      '\n\n---\n# Appendix: FSM\nURL: https://mechcrate.dev/docs/corpus/patterns/appendix-fsm/\nSource: docs/development/appendix-fsm.md\n---\n\n'
    );
  });

  it('omits the source line when the page has no repository file', () => {
    const out = docSeparator({ title: 'T', url: 'https://mechcrate.dev/t/' });
    expect(out).not.toContain('Source:');
    expect(out).toBe('\n\n---\n# T\nURL: https://mechcrate.dev/t/\n---\n\n');
  });
});

describe('buildLlmsFullTxt', () => {
  const pages: LlmsPage[] = [
    page({ title: 'Landing', route: '/', description: 'No markdown body.', kind: 'overview' }),
    page({
      title: 'Install',
      route: '/docs/start/install/',
      body: '# Install\n\nBuild mx from source.\n',
      sourcePath: `${APP_REPO_PREFIX}/src/content/docs/docs/start/install.md`,
      kind: 'start',
    }),
    page({ title: 'Corpus overview', route: '/docs/corpus/', kind: 'corpus', order: -1 }),
    page({
      title: 'Appendix: FSM',
      route: '/docs/corpus/patterns/appendix-fsm/',
      body: '# Appendix: FSM\n\nA distinctive paragraph about state machines.\n',
      sourcePath: 'docs/development/appendix-fsm.md',
      kind: 'corpus',
      category: 'patterns',
    }),
  ];

  const text = buildLlmsFullTxt({ pages });

  it('concatenates the full body of every document', () => {
    expect(text).toContain('Build mx from source.');
    expect(text).toContain('A distinctive paragraph about state machines.');
  });

  it('prefaces each document with its separator block', () => {
    expect(text).toContain(
      '---\n# Install\nURL: https://mechcrate.dev/docs/start/install/\nSource: site/apps/site/src/content/docs/docs/start/install.md\n---'
    );
    expect(text).toContain('Source: docs/development/appendix-fsm.md');
  });

  it('skips pages that have no markdown body, and counts only what it emitted', () => {
    expect(text.match(/^URL: /gm)).toHaveLength(2);
    expect(text).toContain('The complete text of 2 published documents');
    expect(text).not.toContain('URL: https://mechcrate.dev/\n');
    expect(text).not.toContain('# Corpus overview\nURL:');
  });

  it('emits documents in navigation order — guides before corpus', () => {
    expect(text.indexOf('# Install\nURL:')).toBeLessThan(text.indexOf('# Appendix: FSM\nURL:'));
  });

  it('carries the same blockquote description and links back to the index', () => {
    expect(text.split('\n')[0]).toBe('# MechCrate — full text');
    expect(text).toContain('https://mechcrate.dev/llms.txt');
  });
});

describe('pagesFromDocsEntries', () => {
  const entries: DocsEntryLike[] = [
    {
      id: '404',
      filePath: 'src/content/docs/404.md',
      body: 'not found',
      data: { title: 'Page not found', sidebar: { hidden: true } },
    },
    {
      id: 'docs/start/install',
      filePath: 'src/content/docs/docs/start/install.md',
      body: '# Install\n\nBuild mx.\n',
      data: { title: 'Install', description: 'How to install.', sidebar: { order: 2 } },
    },
    {
      id: 'docs/corpus/patterns/appendix-fsm',
      body: '# FSM\n\nStates.\n',
      data: {
        title: 'Appendix: FSM',
        description: 'State machines.',
        corpus: { category: 'patterns', repoPath: 'docs/development/appendix-fsm.md' },
      },
    },
    {
      id: 'docs/internal/scratch',
      filePath: 'src/content/docs/docs/internal/scratch.md',
      body: 'x',
      data: { title: 'Scratch' },
    },
  ];

  const pages = pagesFromDocsEntries(entries);

  it('drops the 404 route and anything outside the navigation groups', () => {
    expect(pages.map((p) => p.route)).toEqual([
      '/docs/start/install/',
      '/docs/corpus/patterns/appendix-fsm/',
    ]);
  });

  it('carries the sidebar order, description and body through', () => {
    const install = pages.find((p) => p.route === '/docs/start/install/')!;
    expect(install).toMatchObject({
      title: 'Install',
      description: 'How to install.',
      kind: 'start',
      order: 2,
    });
    expect(install.body).toContain('Build mx.');
  });

  it('makes an authored page source path repo-relative', () => {
    const install = pages.find((p) => p.route === '/docs/start/install/')!;
    expect(install.sourcePath).toBe(`${APP_REPO_PREFIX}/src/content/docs/docs/start/install.md`);
  });

  it('uses the corpus repo path verbatim and tags the category', () => {
    const fsm = pages.find((p) => p.route === '/docs/corpus/patterns/appendix-fsm/')!;
    expect(fsm.sourcePath).toBe('docs/development/appendix-fsm.md');
    expect(fsm.kind).toBe('corpus');
    expect(fsm.category).toBe('patterns');
  });
});

describe('generatedNavPages', () => {
  const nav = generatedNavPages(
    { title: 'MechCrate', description: 'The front page.' },
    new Map([
      ['theory', 12],
      ['api-design', 1],
    ])
  );

  it('emits the landing page, the corpus overview and one index per category', () => {
    expect(nav.map((p) => p.route)).toEqual([
      '/',
      '/docs/corpus/',
      '/docs/corpus/api-design/',
      '/docs/corpus/theory/',
    ]);
  });

  it('gives navigation pages no markdown body, so llms-full.txt skips them', () => {
    expect(nav.every((p) => p.body === undefined)).toBe(true);
  });

  it('states the document count on each category index, pluralised', () => {
    expect(nav.find((p) => p.route === '/docs/corpus/api-design/')!.description).toContain(
      'The 1 api design document in'
    );
    expect(nav.find((p) => p.route === '/docs/corpus/theory/')!.description).toContain(
      'The 12 theory documents in'
    );
  });

  it('files every corpus navigation page without a category', () => {
    const corpusNav = nav.filter((p) => p.kind === 'corpus');
    expect(corpusNav).toHaveLength(3);
    expect(corpusNav.every((p) => p.category === undefined)).toBe(true);
  });
});

describe('corpusCategoryCounts', () => {
  it('counts only corpus documents, not corpus navigation pages', () => {
    const counts = corpusCategoryCounts([
      page({ title: 'a', route: '/docs/corpus/theory/a/', kind: 'corpus', category: 'theory' }),
      page({ title: 'b', route: '/docs/corpus/theory/b/', kind: 'corpus', category: 'theory' }),
      page({ title: 'nav', route: '/docs/corpus/', kind: 'corpus' }),
      page({ title: 'start', route: '/docs/start/', kind: 'start' }),
    ]);
    expect([...counts.entries()]).toEqual([['theory', 2]]);
  });
});

describe('the real corpus reaching both files', () => {
  const repoRoot = defaultRepoRoot();
  const corpus = buildCorpus(collectCorpusSources(repoRoot), {
    repoFileExists: repoFileExistsIn(repoRoot),
  });

  const pages = pagesFromDocsEntries(
    corpus.published.map((doc) => ({
      id: doc.id,
      body: doc.body,
      data: { title: doc.data.title, description: doc.data.description, corpus: doc.data.corpus },
    }))
  );
  const index = buildLlmsTxt({ pages });
  const full = buildLlmsFullTxt({ pages });

  it('indexes every published corpus document exactly once', () => {
    expect(pages).toHaveLength(corpus.published.length);
    for (const doc of corpus.published) {
      expect(index.split(`(https://mechcrate.dev${doc.route})`)).toHaveLength(2);
    }
  });

  it('carries the full text of every published corpus document', () => {
    expect(full.match(/^URL: /gm)).toHaveLength(corpus.published.length);
    for (const doc of corpus.published) {
      expect(full).toContain(`Source: ${doc.repoPath}`);
    }
  });

  it('holds back every doc the pipeline held back — from both files', () => {
    expect(corpus.skipped.length).toBeGreaterThan(0);
    for (const skip of corpus.skipped) {
      expect(index).not.toContain(skip.repoPath);
      expect(full).not.toContain(`Source: ${skip.repoPath}`);
    }
  });

  it('specifically excludes the Apple design guidelines hold-backs', () => {
    const heldBack = corpus.skipped.map((s) => s.repoPath);
    expect(heldBack).toContain('docs/development/APPLE_DESIGN_GUIDELINES.md');
    expect(index.toLowerCase()).not.toContain('apple-design-guidelines');
    expect(full.toLowerCase()).not.toContain('source: docs/development/apple_design');
  });
});
