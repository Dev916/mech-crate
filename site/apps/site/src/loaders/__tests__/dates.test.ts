/**
 * Page → last-modified date resolution (`src/loaders/lib/dates.ts`).
 *
 * Two halves, same as the corpus pipeline: fixture-driven unit tests pin the
 * pure contract, then a real-repository block proves the thing the sitemap
 * actually depends on — that every published route resolves to a committed
 * date, that `researched:` beats git, and that no date is a build timestamp.
 */

import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

import {
  CORPUS_CATEGORY_PAGE,
  PageDateError,
  auditSitemapXml,
  buildPageDateSources,
  contentRoute,
  lastmodForUrl,
  normalizeRoute,
  pageDateIndex,
  pageRoute,
  parseGitLogDates,
  readGitDates,
  resolvePageDate,
  toIsoTimestamp,
} from '../lib/dates.ts';
import { defaultRepoRoot } from '../lib/sources.ts';

const APP_PATH = 'site/apps/site';
const CATEGORY_PAGE_REL = CORPUS_CATEGORY_PAGE.slice('src/pages/'.length);

/**
 * One commit header exactly as `--format=%x00%cI` writes it: a NUL byte, then
 * the committer date. (`\u0000` rather than `'\0'` — `\0` followed by a digit
 * is an octal escape, which is a syntax error in a module.)
 */
const commit = (iso: string): string => `\u0000${iso}`;

/** `git log --format=%x00%cI --name-only` output, as git emits it. */
const gitLogFixture = [
  commit('2026-09-05T18:35:04-04:00'),
  '',
  'docs/development/appendix-fsm.md',
  'site/apps/site/src/pages/index.astro',
  commit('2026-08-14T10:00:00+02:00'),
  '',
  'docs/development/appendix-fsm.md',
  'docs/development/older-only.md',
  '',
].join('\n');

describe('parseGitLogDates', () => {
  const dates = parseGitLogDates(gitLogFixture);

  it('keeps the newest commit that touched each path (git walks newest-first)', () => {
    expect(dates.get('docs/development/appendix-fsm.md')).toBe('2026-09-05T18:35:04-04:00');
    expect(dates.get('docs/development/older-only.md')).toBe('2026-08-14T10:00:00+02:00');
  });

  it('reads every path in a commit, not just the first', () => {
    expect(dates.get('site/apps/site/src/pages/index.astro')).toBe('2026-09-05T18:35:04-04:00');
    expect(dates.size).toBe(3);
  });

  it('ignores blank separator lines and tolerates CRLF', () => {
    const crlf = parseGitLogDates(`${commit('2026-01-02T03:04:05Z')}\r\n\r\na/b.md\r\n`);
    expect(crlf.get('a/b.md')).toBe('2026-01-02T03:04:05Z');
  });

  it('ignores path lines that appear before any commit header', () => {
    expect(parseGitLogDates('stray/path.md\n').size).toBe(0);
  });
});

describe('toIsoTimestamp', () => {
  it('pins a bare YYYY-MM-DD to UTC midnight so the authored day survives any timezone', () => {
    expect(toIsoTimestamp('2026-08-14')).toBe('2026-08-14T00:00:00.000Z');
  });

  it('normalises a git %cI offset timestamp to UTC', () => {
    expect(toIsoTimestamp('2026-08-17T00:57:46-04:00')).toBe('2026-08-17T04:57:46.000Z');
  });

  it('rejects anything that is not one of the two ISO shapes', () => {
    expect(() => toIsoTimestamp('last tuesday')).toThrow(PageDateError);
    expect(() => toIsoTimestamp('   ')).toThrow(PageDateError);
    expect(() => toIsoTimestamp('2026-13-45')).toThrow(PageDateError);
  });

  it("does not fall for V8's lenient parser, which reads `summer 2026` as 1 January", () => {
    expect(() => toIsoTimestamp('summer 2026')).toThrow(PageDateError);
  });
});

describe('route derivation', () => {
  it('normalises pathnames to a leading and trailing slash', () => {
    expect(normalizeRoute('/')).toBe('/');
    expect(normalizeRoute('docs/ai')).toBe('/docs/ai/');
    expect(normalizeRoute('/docs/ai/')).toBe('/docs/ai/');
  });

  it('maps collection files to their Starlight route, collapsing index files', () => {
    expect(contentRoute('docs/index.md')).toBe('/docs/');
    expect(contentRoute('docs/ai/index.mdx')).toBe('/docs/ai/');
    expect(contentRoute('docs/ai/agent-rules.md')).toBe('/docs/ai/agent-rules/');
    expect(contentRoute('404.md')).toBe('/404/');
    expect(contentRoute('docs/ai/notes.txt')).toBeNull();
  });

  it('maps file-routed pages, and refuses dynamic routes and endpoints', () => {
    expect(pageRoute('index.astro')).toBe('/');
    expect(pageRoute('docs/corpus/index.astro')).toBe('/docs/corpus/');
    expect(pageRoute(CATEGORY_PAGE_REL)).toBeNull();
    expect(pageRoute('llms.txt.ts')).toBeNull();
    expect(pageRoute('api/health.ts')).toBeNull();
  });
});

describe('buildPageDateSources', () => {
  const input = {
    appPath: APP_PATH,
    contentFiles: ['404.md', 'docs/index.md', 'docs/ai/index.mdx', 'docs/ai/agent-rules.md'],
    pageFiles: ['index.astro', 'docs/corpus/index.astro', CATEGORY_PAGE_REL, 'llms.txt.ts'],
    corpusDocs: [
      {
        route: '/docs/corpus/theory/appendix-fsm/',
        repoPath: 'docs/development/appendix-fsm.md',
        category: 'theory',
        researched: '2026-08-14',
      },
      {
        route: '/docs/corpus/infra/infra-config/',
        repoPath: 'docs/development/infra-config.md',
        category: 'infra',
      },
    ],
  };
  const index = buildPageDateSources(input);

  it('dates a corpus page from its repo source, carrying `researched:` through', () => {
    expect(index.get('/docs/corpus/theory/appendix-fsm/')).toEqual({
      sourcePath: 'docs/development/appendix-fsm.md',
      explicit: '2026-08-14',
    });
    expect(index.get('/docs/corpus/infra/infra-config/')).toEqual({
      sourcePath: 'docs/development/infra-config.md',
    });
  });

  it('dates each generated category index from the generator that emits it', () => {
    for (const category of ['theory', 'infra']) {
      expect(index.get(`/docs/corpus/${category}/`)).toEqual({
        sourcePath: `${APP_PATH}/${CORPUS_CATEGORY_PAGE}`,
      });
    }
  });

  it('dates authored docs from their collection file and pages from their .astro file', () => {
    expect(index.get('/docs/ai/agent-rules/')?.sourcePath).toBe(
      `${APP_PATH}/src/content/docs/docs/ai/agent-rules.md`
    );
    expect(index.get('/docs/ai/')?.sourcePath).toBe(
      `${APP_PATH}/src/content/docs/docs/ai/index.mdx`
    );
    expect(index.get('/')?.sourcePath).toBe(`${APP_PATH}/src/pages/index.astro`);
    expect(index.get('/docs/corpus/')?.sourcePath).toBe(
      `${APP_PATH}/src/pages/docs/corpus/index.astro`
    );
  });

  it('leaves endpoints out — llms.txt is not a sitemap page', () => {
    expect([...index.keys()].some((route) => route.includes('llms'))).toBe(false);
  });

  it('lets a corpus doc that collapses onto a category route beat the generator', () => {
    const collapsed = buildPageDateSources({
      ...input,
      corpusDocs: [
        {
          route: '/docs/corpus/theory/',
          repoPath: 'docs/development/theory-index.md',
          category: 'theory',
        },
      ],
    });
    expect(collapsed.get('/docs/corpus/theory/')?.sourcePath).toBe(
      'docs/development/theory-index.md'
    );
  });

  it('fails loudly when the category generator is gone rather than leaving indexes undated', () => {
    expect(() =>
      buildPageDateSources({ ...input, pageFiles: ['index.astro'] })
    ).toThrow(/corpus category indexes have no generator/);
  });
});

describe('resolvePageDate', () => {
  const gitDates = new Map([['docs/development/appendix-fsm.md', '2026-09-05T18:35:04-04:00']]);

  it('prefers `researched:` frontmatter over the git commit date', () => {
    const resolved = resolvePageDate(
      '/docs/corpus/theory/appendix-fsm/',
      { sourcePath: 'docs/development/appendix-fsm.md', explicit: '2026-08-14' },
      gitDates
    );
    expect(resolved).toEqual({ iso: '2026-08-14T00:00:00.000Z', origin: 'frontmatter' });
  });

  it('falls back to the git commit date when there is no frontmatter date', () => {
    const resolved = resolvePageDate(
      '/docs/corpus/theory/appendix-fsm/',
      { sourcePath: 'docs/development/appendix-fsm.md' },
      gitDates
    );
    expect(resolved.iso).toBe('2026-09-05T22:35:04.000Z');
    expect(resolved.origin).toBe('git');
  });

  it('warns and uses git when `researched:` is present but unparseable', () => {
    const resolved = resolvePageDate(
      '/docs/corpus/theory/appendix-fsm/',
      { sourcePath: 'docs/development/appendix-fsm.md', explicit: 'summer 2026' },
      gitDates
    );
    expect(resolved.origin).toBe('git');
    expect(resolved.warning).toMatch(/unparseable `researched: summer 2026`/);
  });

  it('fails with fetch-depth guidance when git knows nothing about the file', () => {
    expect(() =>
      resolvePageDate('/docs/new/', { sourcePath: 'docs/development/brand-new.md' }, gitDates)
    ).toThrow(PageDateError);
    expect(() =>
      resolvePageDate('/docs/new/', { sourcePath: 'docs/development/brand-new.md' }, gitDates)
    ).toThrow(/fetch-depth: 0/);
  });
});

describe('auditSitemapXml', () => {
  it('counts URLs and names the ones missing a lastmod', () => {
    const xml =
      '<urlset><url><loc>https://mechcrate.dev/a/</loc><lastmod>2026-08-14T00:00:00.000Z</lastmod></url>' +
      '<url><loc>https://mechcrate.dev/b/</loc></url></urlset>';
    expect(auditSitemapXml(xml)).toEqual({ total: 2, missing: ['https://mechcrate.dev/b/'] });
  });

  it('treats an empty lastmod element as missing', () => {
    const audit = auditSitemapXml('<url><loc>https://mechcrate.dev/c/</loc><lastmod></lastmod></url>');
    expect(audit.missing).toEqual(['https://mechcrate.dev/c/']);
  });

  it('reports nothing for an empty sitemap', () => {
    expect(auditSitemapXml('<urlset></urlset>')).toEqual({ total: 0, missing: [] });
  });
});

describe('the real repository', () => {
  const repoRoot = defaultRepoRoot();
  const gitDates = readGitDates(repoRoot);
  const index = pageDateIndex();

  const gitLogOne = (repoPath: string): string =>
    execFileSync('git', ['log', '-1', '--format=%cI', '--', repoPath], {
      cwd: repoRoot,
      encoding: 'utf8',
    }).trim();

  it.each([
    'docs/development/appendix-fsm.md',
    'site/apps/site/src/content/docs/docs/start/install.md',
    'site/apps/site/src/pages/index.astro',
  ])('reads the same commit date as `git log -1` for %s', (repoPath) => {
    expect(gitDates.get(repoPath)).toBe(gitLogOne(repoPath));
  });

  it('dates every published route, landing page and generated index included', () => {
    const undated = [...index.dates.entries()].filter(([, iso]) => !iso);
    expect(undated).toEqual([]);
    expect(index.dates.get('/')).toBe(toIsoTimestamp(gitLogOne('site/apps/site/src/pages/index.astro')));
    expect(index.dates.has('/docs/')).toBe(true);
    expect(index.dates.has('/docs/corpus/')).toBe(true);
    expect(index.dates.has('/docs/corpus/theory/')).toBe(true);
    expect(index.dates.size).toBeGreaterThan(100);
  });

  it('shows a `researched:` doc exactly the date its frontmatter claims', () => {
    const route = '/docs/corpus/architecture/multi-agent-systems-in-practice/';
    expect(index.dates.get(route)).toBe('2026-08-14T00:00:00.000Z');
    expect(index.origins.get(route)).toBe('frontmatter');
  });

  it('takes nine dates from frontmatter and the rest from git', () => {
    const origins = [...index.origins.values()];
    expect(origins.filter((origin) => origin === 'frontmatter')).toHaveLength(9);
    expect(origins.every((origin) => origin === 'frontmatter' || origin === 'git')).toBe(true);
  });

  it('never invents a date — every value is a committed fact in the past', () => {
    const now = Date.now();
    for (const [route, iso] of index.dates) {
      expect(iso, route).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/);
      expect(Date.parse(iso), route).toBeLessThanOrEqual(now);
    }
  });

  it('gives every sitemap URL a lastmod', () => {
    // The routes the sitemap publishes: everything but the status-code page,
    // which @astrojs/sitemap drops.
    const routes = [...index.dates.keys()].filter((route) => route !== '/404/');
    const xml = routes
      .map((route) => {
        const url = `https://mechcrate.dev${route}`;
        return `<url><loc>${url}</loc><lastmod>${lastmodForUrl(url, index)}</lastmod></url>`;
      })
      .join('');

    expect(auditSitemapXml(`<urlset>${xml}</urlset>`)).toEqual({
      total: routes.length,
      missing: [],
    });
  });

  it('refuses to guess for a URL no source file backs', () => {
    expect(() => lastmodForUrl('https://mechcrate.dev/not-a-page/', index)).toThrow(PageDateError);
  });

  it.runIf(existsSync(join(repoRoot, 'site/apps/site/dist/sitemap-0.xml')))(
    'has a built sitemap whose every URL carries a lastmod',
    () => {
      const xml = readFileSync(join(repoRoot, 'site/apps/site/dist/sitemap-0.xml'), 'utf8');
      const audit = auditSitemapXml(xml);
      expect(audit.missing).toEqual([]);
      expect(audit.total).toBeGreaterThan(100);
    }
  );
});
