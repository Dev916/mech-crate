import { readFileSync, readdirSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

/**
 * Keeps em dashes out of the copy a reader actually sees.
 *
 * An em dash every second sentence is the house style of a language model, not
 * of a person, and a page full of them reads as generated whatever the prose
 * underneath is worth. The site had ~250 of them. They are gone, and this is the
 * thing that stops them coming back one merged PR at a time.
 *
 * The rule is about *rewriting*, not about the character: swapping "—" for "-"
 * or ";" reads worse than leaving it alone. Break the sentence in two, or use a
 * comma, a colon, or parentheses — whichever the sentence actually wants. If
 * this test fails, that is the fix, not a find-and-replace.
 *
 * Deliberately out of scope, and why:
 *
 * - `docs/development/` (the 67 corpus documents), which is repo-owned research
 *   prose published verbatim, a separate concern with its own editorial history.
 * - fenced code blocks and inline code spans, where a dash is literal command or
 *   terminal text rather than punctuation.
 * - source comments, read by contributors rather than by visitors.
 * - the node labels in `components/diagrams/sources/*.mmd`. Those are authored
 *   copy and they do still carry dashes, but the rendered SVGs are committed
 *   build output, so changing a label means re-running `npm run diagrams` and
 *   rewriting all ten files. Worth doing; not worth bundling into a copy edit.
 *   The figcaptions around them, in `diagrams/registry.ts`, are covered here.
 *
 * En dashes (U+2013) in numeric ranges like "2019–2024" are correct typography
 * and are not checked here. Only U+2014 is.
 */

const EM_DASH = '—';

const here = dirname(fileURLToPath(import.meta.url));
const APP_ROOT = join(here, '..', '..');
const SRC = join(APP_ROOT, 'src');
const DOCS = join(SRC, 'content', 'docs');

/** Every authored `.md`/`.mdx` under the docs collection. */
function docsPages(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) return docsPages(full);
    return /\.mdx?$/.test(entry.name) ? [full] : [];
  });
}

/**
 * Markdown minus the parts that are not prose. Frontmatter stays — `description:`
 * is what feeds the meta description and the OG card, so it is copy too.
 */
function proseOf(markdown: string): string {
  return markdown
    .replace(/^(```|~~~)[\s\S]*?^\1.*$/gm, '') // fenced code
    .replace(/`[^`\n]*`/g, ''); // inline code spans
}

/**
 * Source minus its comments. A leading `//` only: an inline `//` is far more
 * likely to be the middle of a URL than the start of a comment.
 */
function renderedCopyOf(source: string): string {
  return source
    .replace(/\{?\/\*[\s\S]*?\*\/\}?/g, '')
    .replace(/^\s*\/\/.*$/gm, '')
    .replace(/`[^`\n]*`/g, '');
}

const SOURCE_FILES = [
  join(SRC, 'site-meta.ts'),
  join(SRC, 'pages', 'index.astro'),
  join(SRC, 'components', 'diagrams', 'registry.ts'), // the figcaptions

  join(SRC, 'pages', 'docs', 'corpus', 'index.astro'),
  join(SRC, 'pages', 'docs', 'corpus', '[category].astro'),
  join(APP_ROOT, 'astro.config.mjs'),
];

const label = (file: string) => relative(APP_ROOT, file);

describe('authored copy carries no em dashes', () => {
  const pages = docsPages(DOCS);

  it('finds the docs collection', () => {
    // Guards against the walker silently matching nothing and the suite passing
    // for the wrong reason.
    expect(pages.length).toBeGreaterThan(20);
  });

  it.each(pages.map((f) => [label(f), f] as const))('%s', (_name, file) => {
    const offenders = proseOf(readFileSync(file, 'utf8'))
      .split('\n')
      .filter((line) => line.includes(EM_DASH));
    expect(offenders, `rewrite these lines rather than substituting the dash`).toEqual([]);
  });

  it.each(SOURCE_FILES.map((f) => [label(f), f] as const))('%s', (_name, file) => {
    const offenders = renderedCopyOf(readFileSync(file, 'utf8'))
      .split('\n')
      .filter((line) => line.includes(EM_DASH));
    expect(offenders, `rewrite these lines rather than substituting the dash`).toEqual([]);
  });
});
