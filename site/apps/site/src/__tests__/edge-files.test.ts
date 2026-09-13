/**
 * Unit tests for the two edge-policy files in `public/`: `robots.txt` and
 * `_headers`.
 *
 * Neither file is code, so nothing here can be caught by `astro build` — it
 * copies `public/` verbatim and never looks inside. A typo in a header name, a
 * relative `Sitemap:` line, or a second rule quietly comma-joining onto a
 * `Cache-Control` would all ship silently. Hence assertions on the bytes.
 *
 * `parseHeaderRules` below deliberately reimplements the rules that Cloudflare's
 * asset worker uses (read out of wrangler 4.125.0,
 * `workers-shared/utils/configuration/parseHeaders.ts`): blank and `#` lines are
 * dropped, a rule starts on a line beginning with `/` or a scheme, header lines
 * are `Name: value` with the name lowercased, `! Name` detaches a header. If
 * `_headers` ever stops parsing the way the edge parses it, the shape
 * assertions fail here rather than in production.
 *
 * The live half of this — that the parsed rules actually reach the wire — is
 * the "Edge headers" step in `.github/workflows/site.yml`, which curls a real
 * `wrangler dev` (workerd) serving `dist/`.
 */

import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import { SITE_ORIGIN } from '../loaders/lib/llms.ts';

const publicDir = join(dirname(fileURLToPath(import.meta.url)), '..', '..', 'public');

const robotsTxt = readFileSync(join(publicDir, 'robots.txt'), 'utf8');
const headersFile = readFileSync(join(publicDir, '_headers'), 'utf8');

/** Limits enforced by the Workers-assets `_headers` parser. */
const MAX_HEADER_RULES = 100;
const MAX_LINE_LENGTH = 2000;

interface HeaderRule {
  path: string;
  /** Header name (lowercased) → value, in file order. */
  set: Record<string, string>;
  /** Names detached via the `! Name` operator. */
  unset: string[];
}

/** Mirror of the asset worker's `_headers` parser (see the module docblock). */
function parseHeaderRules(input: string): HeaderRule[] {
  const rules: HeaderRule[] = [];
  let current: HeaderRule | undefined;

  for (const raw of input.split('\n')) {
    const line = raw.trim();
    if (line.length === 0 || line.startsWith('#')) continue;

    if (/^([^\s]+:\/\/|\/)/.test(line)) {
      current = { path: line, set: {}, unset: [] };
      rules.push(current);
      continue;
    }

    if (!current) throw new Error(`header line before any path rule: ${line}`);

    if (line.startsWith('! ')) {
      current.unset.push(line.slice(2).trim());
      continue;
    }

    const separator = line.indexOf(':');
    if (separator === -1) throw new Error(`not a header pair: ${line}`);
    const name = line.slice(0, separator).trim().toLowerCase();
    const value = line.slice(separator + 1).trim();
    expect(name, `header name in "${line}"`).not.toBe('');
    expect(value, `header value in "${line}"`).not.toBe('');
    current.set[name] = value;
  }

  return rules;
}

/**
 * Whether two rule paths can both match a single request. Mirrors the asset
 * worker's rule compilation (`*` becomes a greedy wildcard, everything else is
 * literal) closely enough for the single-splat paths this file uses.
 */
function pathsOverlap(a: string, b: string): boolean {
  const probe = '__splat__';
  const toRegExp = (path: string) =>
    new RegExp(`^${path.split('*').map((part) => part.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('.*')}$`);
  const toSample = (path: string) => path.split('*').join(probe);
  return toRegExp(a).test(toSample(b)) || toRegExp(b).test(toSample(a));
}

const rules = parseHeaderRules(headersFile);
const ruleFor = (path: string): HeaderRule => {
  const match = rules.find((rule) => rule.path === path);
  if (!match) throw new Error(`no _headers rule for ${path}; have ${rules.map((r) => r.path).join(', ')}`);
  return match;
};

describe('public/robots.txt', () => {
  it('is an allow-all policy with no Disallow directives', () => {
    expect(robotsTxt).toContain('User-agent: *');
    expect(robotsTxt).toContain('Allow: /');
    expect(robotsTxt).not.toMatch(/^\s*Disallow:/m);
  });

  it('carries the Content-Signal opt-in for search, AI input and AI training', () => {
    expect(robotsTxt).toMatch(/^Content-Signal: search=yes, ai-input=yes, ai-train=yes$/m);
  });

  it('points at the sitemap index by absolute URL on the canonical origin', () => {
    expect(robotsTxt).toMatch(/^Sitemap: (\S+)$/m);
    const sitemap = /^Sitemap: (\S+)$/m.exec(robotsTxt)?.[1];
    expect(sitemap).toBe(`${SITE_ORIGIN}/sitemap-index.xml`);
    // Google ignores relative Sitemap lines outright.
    expect(sitemap?.startsWith('https://')).toBe(true);
  });

  it('is exactly the policy the design spec fixes, byte for byte', () => {
    expect(robotsTxt).toBe(
      [
        'User-agent: *',
        'Allow: /',
        '',
        'Content-Signal: search=yes, ai-input=yes, ai-train=yes',
        '',
        `Sitemap: ${SITE_ORIGIN}/sitemap-index.xml`,
        '',
      ].join('\n'),
    );
  });
});

describe('public/_headers', () => {
  it('parses within the Workers-assets limits', () => {
    expect(rules.length).toBeLessThanOrEqual(MAX_HEADER_RULES);
    for (const line of headersFile.split('\n')) {
      expect(line.length).toBeLessThanOrEqual(MAX_LINE_LENGTH);
    }
  });

  it('uses at most one splat per rule path', () => {
    for (const rule of rules) {
      expect((rule.path.match(/\*/g) ?? []).length, rule.path).toBeLessThanOrEqual(1);
    }
  });

  it('never sets the same header from two rules that can match one request', () => {
    // The asset worker comma-JOINS a header set by a second MATCHING rule
    // rather than overwriting it, so two overlapping rules both setting
    // `Cache-Control` would emit "public, max-age=3600, public, max-age=…".
    // Rules whose paths cannot both match the same request (/llms.txt and
    // /llms-full.txt) are free to repeat a header name.
    for (let i = 0; i < rules.length; i++) {
      for (let j = i + 1; j < rules.length; j++) {
        const a = rules[i]!;
        const b = rules[j]!;
        if (!pathsOverlap(a.path, b.path)) continue;
        const shared = Object.keys(a.set).filter((name) => name in b.set);
        expect(shared, `${a.path} and ${b.path} both set ${shared.join(', ')}`).toEqual([]);
      }
    }
  });

  it('caches fingerprinted /_astro/ output immutably for a year', () => {
    expect(ruleFor('/_astro/*').set['cache-control']).toBe(
      'public, max-age=31536000, immutable',
    );
  });

  it('sets the site-wide transport and privacy headers', () => {
    const site = ruleFor('/*');
    expect(site.set['strict-transport-security']).toBe(
      'max-age=31536000; includeSubDomains',
    );
    expect(site.set['x-content-type-options']).toBe('nosniff');
    expect(site.set['referrer-policy']).toBe('strict-origin-when-cross-origin');
  });

  it('ships HSTS without preload', () => {
    // `preload` is a one-way door: submission is easy, removal takes months.
    // It goes in only once the http -> https 301 has been live and healthy.
    expect(ruleFor('/*').set['strict-transport-security']).not.toContain('preload');
  });

  it('serves every llms artefact as inline UTF-8 text with an hour of cache', () => {
    for (const path of ['/llms.txt', '/llms-*.txt']) {
      const rule = ruleFor(path);
      expect(rule.set['content-type'], path).toBe('text/plain; charset=utf-8');
      expect(rule.set['cache-control'], path).toBe('public, max-age=3600');
    }
  });

  it('covers the section splits with one splat rule that cannot reach /llms.txt', () => {
    // `/llms-*.txt` has to catch llms-full.txt, llms-guides.txt and the fifteen
    // llms-corpus-<category>.txt files without also matching /llms.txt — which
    // would comma-join Content-Type onto itself for that one path.
    const matches = (pattern: string, path: string) =>
      new RegExp(
        `^${pattern.split('*').map((part) => part.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('.*')}$`,
      ).test(path);

    for (const path of [
      '/llms-full.txt',
      '/llms-guides.txt',
      '/llms-corpus-theory.txt',
      '/llms-corpus-framework-guides.txt',
    ]) {
      expect(matches('/llms-*.txt', path), path).toBe(true);
      expect(matches('/llms.txt', path), path).toBe(false);
    }
    expect(matches('/llms-*.txt', '/llms.txt')).toBe(false);
    // A page route must never be swept up by it.
    expect(matches('/llms-*.txt', '/docs/start/install/')).toBe(false);
  });

  it('pins the markdown twins to text/markdown with the same hour of cache', () => {
    // Verified against workerd: the asset server already infers text/markdown
    // from the extension, so this rule fixes the answer rather than repairing
    // it. `.md` cannot collide with a page — Astro writes every route as
    // `<route>/index.html`.
    const rule = ruleFor('/*.md');
    expect(rule.set['content-type']).toBe('text/markdown; charset=utf-8');
    expect(rule.set['cache-control']).toBe('public, max-age=3600');
  });

  it('marks the 404 page noindex', () => {
    // Covers DIRECT requests to /404 only. The asset worker matches these
    // rules against the requested pathname, so an ordinary not-found response
    // (some /typo resolving to 404.html) is not reachable from here — only
    // `/*` would match it, and that would noindex the whole site.
    expect(ruleFor('/404').set['x-robots-tag']).toBe('noindex');
  });

  it('does not noindex anything but the 404 page', () => {
    for (const rule of rules) {
      if (rule.path === '/404') continue;
      expect(rule.set['x-robots-tag'], rule.path).toBeUndefined();
    }
  });
});
