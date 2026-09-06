/**
 * Build-time "when was this page last modified" resolution.
 *
 * Google only trusts `<lastmod>` when it is verifiably accurate, so the site
 * derives every date from something that actually records a change:
 *
 *   1. a corpus doc's `researched:` frontmatter, when it carries one;
 *   2. otherwise the **last commit date of the page's source file** (`%cI`).
 *
 * Never `new Date()`, never file mtime, never build time — a build-stamped
 * lastmod is exactly the signal Google learned to ignore, and it would also make
 * `dist/` non-reproducible. Both inputs above are committed facts, so two builds
 * of the same tree produce byte-identical sitemaps.
 *
 * Every published URL must resolve to SOME source file. A URL that cannot be
 * dated is a build failure (see `resolvePageDate`), because the failure mode
 * this module exists to eliminate is a silently missing `<lastmod>`.
 *
 * Shape of the index: route pathname (leading + trailing slash, `/` for the
 * landing page) → ISO-8601 UTC timestamp. Task 4/6 surfaces reuse it for
 * `article:modified_time` and JSON-LD `dateModified`, which is why the map is
 * exported rather than inlined into the sitemap integration.
 *
 * See docs/superpowers/specs/2026-09-05-seo-geo-design.md → "3. Freshness".
 */

import { execFileSync } from 'node:child_process';
import { existsSync, readdirSync } from 'node:fs';
import { dirname, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

import { buildCorpus } from './pipeline.ts';
import { collectCorpusSources, defaultRepoRoot, repoFileExistsIn } from './sources.ts';

/** App-relative directory holding the Starlight `docs` collection. */
export const CONTENT_DOCS_DIR = 'src/content/docs';

/** App-relative directory holding the Astro file-routed pages. */
export const PAGES_DIR = 'src/pages';

/**
 * App-relative path of the generator behind `/docs/corpus/<category>/`.
 * Named explicitly so that moving the file fails the build loudly instead of
 * quietly leaving every category index undated.
 */
export const CORPUS_CATEGORY_PAGE = 'src/pages/docs/corpus/[category].astro';

/** File extensions Astro turns into pages (endpoints like `llms.txt.ts` are not pages). */
const PAGE_EXTENSIONS = ['.astro', '.md', '.mdx', '.mdoc', '.html'] as const;

/** Extensions the `docs` content collection accepts. */
const CONTENT_EXTENSIONS = ['.md', '.mdx', '.mdoc'] as const;

/** A page could not be dated. Always names the route and the source path. */
export class PageDateError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'PageDateError';
  }
}

/** Where one route's date comes from. */
export interface PageDateSource {
  /** Repo-relative POSIX path whose git history dates this route. */
  sourcePath: string;
  /** Authored date (`researched:` frontmatter), `YYYY-MM-DD`. Wins over git. */
  explicit?: string;
}

/** One resolved date, with its provenance (used for build diagnostics). */
export interface ResolvedPageDate {
  /** ISO-8601 UTC timestamp, e.g. `2026-08-14T00:00:00.000Z`. */
  iso: string;
  origin: 'frontmatter' | 'git';
  /** Non-fatal problem worth logging (e.g. an unparseable `researched:` value). */
  warning?: string;
}

/** Corpus input for the index — the subset of `CorpusDoc` this module needs. */
export interface CorpusDateInput {
  route: string;
  repoPath: string;
  category: string;
  researched?: string;
}

export interface PageDateSourcesInput {
  /** POSIX path of the Astro app relative to the repo root, e.g. `site/apps/site`. */
  appPath: string;
  /** Paths relative to `<app>/src/content/docs`, POSIX (`docs/ai/index.mdx`). */
  contentFiles: readonly string[];
  /** Paths relative to `<app>/src/pages`, POSIX (`docs/corpus/index.astro`). */
  pageFiles: readonly string[];
  /** Published corpus docs, in pipeline order. */
  corpusDocs: readonly CorpusDateInput[];
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/** `/`-terminated, `/`-prefixed form of a route path. `/` stays `/`. */
export function normalizeRoute(pathname: string): string {
  let route = pathname.startsWith('/') ? pathname : `/${pathname}`;
  if (!route.endsWith('/')) route += '/';
  return route;
}

/** `2026-08-14` — a bare calendar day, the shape `researched:` frontmatter uses. */
const DATE_ONLY_RE = /^\d{4}-\d{2}-\d{2}$/;

/** `2026-08-17T00:57:46-04:00` / `…Z` — the shape `git log --format=%cI` emits. */
const DATE_TIME_RE = /^\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}(:\d{2}(\.\d+)?)?(Z|[+-]\d{2}:?\d{2})?$/;

/**
 * Normalise an authored or git date to an ISO-8601 UTC timestamp.
 *
 * A bare `YYYY-MM-DD` becomes UTC midnight (`2026-08-14T00:00:00.000Z`) so the
 * emitted `<lastmod>` carries exactly the authored day regardless of the build
 * machine's timezone.
 *
 * The shape is checked before `Date` sees it because V8's fallback date parser
 * is far too generous to trust with hand-written frontmatter — `new Date('summer
 * 2026')` happily yields 1 January. Only the two shapes this pipeline actually
 * produces are accepted; everything else is a typo, and typos must surface.
 */
export function toIsoTimestamp(value: string): string {
  const trimmed = value.trim();
  if (trimmed === '') throw new PageDateError('empty date value');
  if (!DATE_ONLY_RE.test(trimmed) && !DATE_TIME_RE.test(trimmed)) {
    throw new PageDateError(`\`${value}\` is not an ISO-8601 date (expected YYYY-MM-DD)`);
  }
  const parsed = new Date(DATE_ONLY_RE.test(trimmed) ? `${trimmed}T00:00:00.000Z` : trimmed);
  if (Number.isNaN(parsed.getTime())) throw new PageDateError(`unparseable date \`${value}\``);
  return parsed.toISOString();
}

/**
 * Parse `git log --format=%x00%cI --name-only` output into path → newest date.
 *
 * `git log` walks newest-first, so the first commit that names a path is that
 * path's last modification: first write wins, later (older) ones are ignored.
 */
export function parseGitLogDates(stdout: string): Map<string, string> {
  const dates = new Map<string, string>();
  let current: string | undefined;

  for (const rawLine of stdout.split('\n')) {
    const line = rawLine.endsWith('\r') ? rawLine.slice(0, -1) : rawLine;
    if (line.startsWith('\0')) {
      current = line.slice(1);
      continue;
    }
    if (line === '' || current === undefined) continue;
    if (!dates.has(line)) dates.set(line, current);
  }

  return dates;
}

function stripExtension(path: string, extensions: readonly string[]): string | null {
  for (const ext of extensions) {
    if (path.endsWith(ext)) return path.slice(0, -ext.length);
  }
  return null;
}

/**
 * Route for a `docs` collection file, path relative to `src/content/docs`.
 *
 *   `docs/index.md`            → `/docs/`
 *   `docs/ai/index.mdx`        → `/docs/ai/`
 *   `docs/ai/agent-rules.md`   → `/docs/ai/agent-rules/`
 *
 * Returns `null` for anything that is not a collection entry.
 */
export function contentRoute(relPath: string): string | null {
  const stem = stripExtension(relPath, CONTENT_EXTENSIONS);
  if (stem === null) return null;

  const segments = stem.split('/').filter(Boolean);
  if (segments.length === 0) return null;
  if (segments[segments.length - 1] === 'index') segments.pop();
  return segments.length === 0 ? '/' : `/${segments.join('/')}/`;
}

/**
 * Route for a file-routed page, path relative to `src/pages`.
 *
 *   `index.astro`                  → `/`
 *   `docs/corpus/index.astro`      → `/docs/corpus/`
 *   `docs/corpus/[category].astro` → `null`  (dynamic — handled per-param)
 *   `llms.txt.ts`, `api/health.ts` → `null`  (endpoints, not sitemap pages)
 */
export function pageRoute(relPath: string): string | null {
  if (relPath.includes('[')) return null;
  const stem = stripExtension(relPath, PAGE_EXTENSIONS);
  if (stem === null) return null;

  const segments = stem.split('/').filter(Boolean);
  if (segments.length === 0) return null;
  if (segments[segments.length - 1] === 'index') segments.pop();
  return segments.length === 0 ? '/' : `/${segments.join('/')}/`;
}

/**
 * Build the route → source-file index for every page the site publishes.
 *
 * Deliberately built from four independent enumerations rather than one clever
 * rule, because each family of routes is generated differently:
 *   - corpus docs        → the repo doc under `docs/development/` (or a root guide)
 *   - category indexes   → `[category].astro`, the generator
 *   - authored docs      → their collection file
 *   - landing + overview → their `src/pages` file
 *
 * Corpus docs are written first and never overwritten: a doc whose filename
 * stem is `index` collapses onto `/docs/corpus/<category>/`, and in that case
 * the doc — not the generator — is the page.
 */
export function buildPageDateSources(input: PageDateSourcesInput): Map<string, PageDateSource> {
  const { appPath, contentFiles, pageFiles, corpusDocs } = input;
  const prefix = appPath === '' ? '' : `${appPath}/`;
  const index = new Map<string, PageDateSource>();

  for (const doc of corpusDocs) {
    index.set(normalizeRoute(doc.route), {
      sourcePath: doc.repoPath,
      ...(doc.researched === undefined ? {} : { explicit: doc.researched }),
    });
  }

  if (corpusDocs.length > 0) {
    if (!pageFiles.includes(CORPUS_CATEGORY_PAGE.slice(`${PAGES_DIR}/`.length))) {
      throw new PageDateError(
        `corpus category indexes have no generator: expected \`${prefix}${CORPUS_CATEGORY_PAGE}\`. ` +
          'If that page moved, update CORPUS_CATEGORY_PAGE in src/loaders/lib/dates.ts.'
      );
    }
    const generator = `${prefix}${CORPUS_CATEGORY_PAGE}`;
    for (const category of new Set(corpusDocs.map((doc) => doc.category))) {
      const route = normalizeRoute(`/docs/corpus/${category}`);
      if (!index.has(route)) index.set(route, { sourcePath: generator });
    }
  }

  for (const file of contentFiles) {
    const route = contentRoute(file);
    if (route === null) continue;
    if (!index.has(route)) index.set(route, { sourcePath: `${prefix}${CONTENT_DOCS_DIR}/${file}` });
  }

  for (const file of pageFiles) {
    const route = pageRoute(file);
    if (route === null) continue;
    if (!index.has(route)) index.set(route, { sourcePath: `${prefix}${PAGES_DIR}/${file}` });
  }

  return index;
}

/**
 * Resolve one route's date. Frontmatter wins; git is the fallback; a route with
 * neither is a hard error.
 *
 * The "no git date" branch is the shallow-clone case: `actions/checkout` fetches
 * depth 1 by default, so `git log` knows about exactly one commit and every
 * other file looks untouched. The message names the fix rather than the symptom.
 */
export function resolvePageDate(
  route: string,
  source: PageDateSource,
  gitDates: ReadonlyMap<string, string>
): ResolvedPageDate {
  let warning: string | undefined;

  if (source.explicit !== undefined) {
    try {
      return { iso: toIsoTimestamp(source.explicit), origin: 'frontmatter' };
    } catch {
      // A typo in `researched:` must not sink the build when git can still date
      // the file — but it must be visible.
      warning = `${source.sourcePath}: ignoring unparseable \`researched: ${source.explicit}\`, using the git commit date`;
    }
  }

  const gitDate = gitDates.get(source.sourcePath);
  if (gitDate === undefined) {
    throw new PageDateError(
      `${route} cannot be dated: git knows no commit touching \`${source.sourcePath}\`.\n` +
        '  · In CI this means a shallow checkout — set `fetch-depth: 0` on actions/checkout.\n' +
        '  · Locally it means the file is new and uncommitted — commit it, or the page ships without a lastmod.\n' +
        '  · It also fires when the build runs outside a git work tree (no repository to read).'
    );
  }

  return { iso: toIsoTimestamp(gitDate), origin: 'git', ...(warning === undefined ? {} : { warning }) };
}

/**
 * Count `<url>` entries and report the ones missing a `<lastmod>`.
 * Used by the build-time guard so "100% of URLs carry a lastmod" is asserted on
 * the bytes that actually ship, not on the intent that produced them.
 */
export function auditSitemapXml(xml: string): { total: number; missing: string[] } {
  const missing: string[] = [];
  let total = 0;

  for (const match of xml.matchAll(/<url>([\s\S]*?)<\/url>/g)) {
    total += 1;
    const entry = match[1] ?? '';
    if (!/<lastmod>[^<]+<\/lastmod>/.test(entry)) {
      missing.push(/<loc>([^<]*)<\/loc>/.exec(entry)?.[1] ?? `<url> #${total}`);
    }
  }

  return { total, missing };
}

// ---------------------------------------------------------------------------
// Filesystem / git side
// ---------------------------------------------------------------------------

/** Every file under `dir`, as POSIX paths relative to `dir`, sorted. */
export function listFilesRelative(dir: string): string[] {
  if (!existsSync(dir)) return [];
  const out: string[] = [];

  const walk = (current: string, prefix: string): void => {
    for (const entry of readdirSync(current, { withFileTypes: true }).sort((a, b) =>
      a.name.localeCompare(b.name)
    )) {
      const rel = prefix === '' ? entry.name : `${prefix}/${entry.name}`;
      if (entry.isDirectory()) walk(join(current, entry.name), rel);
      else if (entry.isFile()) out.push(rel);
    }
  };

  walk(dir, '');
  return out;
}

/**
 * One `git log` walk over the whole history → path → last commit date.
 *
 * One process, not 110: the walk costs ~0.1s on this repo, where 110 individual
 * `git log -1 -- <path>` calls cost seconds of process spawn. `--no-renames`
 * keeps a renamed file's current name in the commit that introduced it, so a
 * path is always dated under the name it ships as.
 */
export function readGitDates(repoRoot: string): Map<string, string> {
  let stdout: string;
  try {
    stdout = execFileSync(
      'git',
      ['-c', 'core.quotePath=false', 'log', '--no-renames', '--format=%x00%cI', '--name-only'],
      { cwd: repoRoot, encoding: 'utf8', maxBuffer: 512 * 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe'] }
    );
  } catch (error) {
    const detail = error instanceof Error ? error.message.split('\n')[0] : String(error);
    throw new PageDateError(
      `git log failed in \`${repoRoot}\` — page dates come from commit history, so the build cannot continue.\n` +
        `  · ${detail}\n` +
        '  · In CI, `actions/checkout` needs `fetch-depth: 0` (a depth-1 clone dates nothing).'
    );
  }

  return parseGitLogDates(stdout);
}

export interface PageDateIndexOptions {
  /** Repository root. Defaults to the detected mech-crate root. */
  repoRoot?: string;
  /** Astro app root. Defaults to this module's own app (`site/apps/site`). */
  appRoot?: string;
}

/** App root resolved relative to this module: …/src/loaders/lib → …/apps/site. */
function defaultAppRoot(): string {
  return resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
}

/** POSIX path of `appRoot` relative to `repoRoot`. */
function appPathWithin(repoRoot: string, appRoot: string): string {
  const rel = relative(repoRoot, appRoot).split(sep).join('/');
  if (rel.startsWith('..')) {
    throw new PageDateError(
      `the Astro app (\`${appRoot}\`) is not inside the repository root (\`${repoRoot}\`), ` +
        'so its source files have no commit history to read. ' +
        'Set MECHCRATE_REPO_ROOT to the checkout that contains the app.'
    );
  }
  return rel;
}

/** Route → source file for every published page, read from disk. */
export function pageDateSources(options: PageDateIndexOptions = {}): Map<string, PageDateSource> {
  const repoRoot = options.repoRoot ?? defaultRepoRoot();
  const appRoot = options.appRoot ?? defaultAppRoot();

  const corpus = buildCorpus(collectCorpusSources(repoRoot), {
    repoFileExists: repoFileExistsIn(repoRoot),
  });

  return buildPageDateSources({
    appPath: appPathWithin(repoRoot, appRoot),
    contentFiles: listFilesRelative(join(appRoot, CONTENT_DOCS_DIR)),
    pageFiles: listFilesRelative(join(appRoot, PAGES_DIR)),
    corpusDocs: corpus.published.map((doc) => ({
      route: doc.route,
      repoPath: doc.repoPath,
      category: doc.category,
      ...(doc.data.corpus.researched === undefined
        ? {}
        : { researched: doc.data.corpus.researched }),
    })),
  });
}

export interface PageDateIndex {
  /** Route pathname → ISO-8601 UTC timestamp. */
  dates: Map<string, string>;
  /** Route pathname → where the date came from. */
  origins: Map<string, ResolvedPageDate['origin']>;
  /** Non-fatal problems, for the caller's logger. */
  warnings: string[];
}

let cache: { key: string; index: PageDateIndex } | undefined;

/**
 * The build's page → date map. Memoised per (repoRoot, appRoot) because the
 * sitemap serializer runs once per URL and later tasks read it per page render;
 * the git walk and the corpus pipeline must not run 110 times.
 */
export function pageDateIndex(options: PageDateIndexOptions = {}): PageDateIndex {
  const repoRoot = options.repoRoot ?? defaultRepoRoot();
  const appRoot = options.appRoot ?? defaultAppRoot();
  const key = `${repoRoot} ${appRoot}`;
  if (cache?.key === key) return cache.index;

  const sources = pageDateSources({ repoRoot, appRoot });
  const gitDates = readGitDates(repoRoot);

  const dates = new Map<string, string>();
  const origins = new Map<string, ResolvedPageDate['origin']>();
  const warnings: string[] = [];

  for (const [route, source] of sources) {
    const resolved = resolvePageDate(route, source, gitDates);
    dates.set(route, resolved.iso);
    origins.set(route, resolved.origin);
    if (resolved.warning !== undefined) warnings.push(resolved.warning);
  }

  const index: PageDateIndex = { dates, origins, warnings };
  cache = { key, index };
  return index;
}

/**
 * Look up a page's `lastmod` by absolute URL (what the sitemap hands us).
 * Throws when the URL is not one of the routes we can date — a missing
 * `<lastmod>` is the bug this whole module exists to prevent, so it is never
 * papered over with a default.
 */
export function lastmodForUrl(url: string, index: PageDateIndex): string {
  const route = normalizeRoute(new URL(url).pathname);
  const iso = index.dates.get(route);
  if (iso === undefined) {
    throw new PageDateError(
      `no source file is mapped to \`${route}\` (${url}), so it cannot be given a <lastmod>.\n` +
        '  · A new page family needs an entry in buildPageDateSources() in src/loaders/lib/dates.ts.'
    );
  }
  return iso;
}
