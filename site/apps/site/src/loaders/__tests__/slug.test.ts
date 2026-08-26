import { describe, expect, it } from 'vitest';
import {
  corpusEntryId,
  corpusRoute,
  fileStem,
  sanitizeSlug,
  titleFromFilename,
} from '../lib/slug.ts';

describe('sanitizeSlug', () => {
  it('replaces tildes, underscores and spaces with a single hyphen', () => {
    expect(sanitizeSlug('mx-mcp~usage')).toBe('mx-mcp-usage');
    expect(sanitizeSlug('MX_RUST_CLI_AND_MCP_SERVER')).toBe('mx-rust-cli-and-mcp-server');
    expect(sanitizeSlug('Apple  Design   Guide')).toBe('apple-design-guide');
  });

  it('collapses repeated separators and trims the edges', () => {
    expect(sanitizeSlug('__weird--name~~here__')).toBe('weird-name-here');
    expect(sanitizeSlug('a..b')).toBe('a-b');
  });

  it('is idempotent and already-clean slugs pass through untouched', () => {
    expect(sanitizeSlug('appendix-fsm')).toBe('appendix-fsm');
    expect(sanitizeSlug(sanitizeSlug('mx-mcp~usage'))).toBe('mx-mcp-usage');
  });

  it('drops characters that are not URL-safe', () => {
    expect(sanitizeSlug('café/notes?v=2')).toBe('cafe-notes-v-2');
  });

  it('returns an empty string when nothing survives', () => {
    expect(sanitizeSlug('~~~')).toBe('');
  });
});

describe('fileStem', () => {
  it('drops the directory and the extension', () => {
    expect(fileStem('docs/development/mx-mcp~usage.md')).toBe('mx-mcp~usage');
    expect(fileStem('docs/router.md')).toBe('router');
  });
});

describe('titleFromFilename', () => {
  it('humanises the filename for the missing-title fallback', () => {
    expect(titleFromFilename('docs/docs-command.md')).toBe('Docs Command');
    expect(titleFromFilename('docs/development/INDEX.md')).toBe('Index');
    expect(titleFromFilename('docs/router.md')).toBe('Router');
  });
});

describe('route construction', () => {
  it('nests corpus pages under /docs/corpus/<category>/<slug>/', () => {
    expect(corpusEntryId('patterns', 'appendix-fsm')).toBe('docs/corpus/patterns/appendix-fsm');
    expect(corpusRoute('patterns', 'appendix-fsm')).toBe('/docs/corpus/patterns/appendix-fsm/');
  });

  it('collapses an `index` slug into its category directory, as Astro does', () => {
    // `docs/development/INDEX.md` is served at /docs/corpus/uncategorized/, so a
    // link to /docs/corpus/uncategorized/index/ would 404.
    expect(corpusEntryId('uncategorized', 'index')).toBe('docs/corpus/uncategorized/index');
    expect(corpusRoute('uncategorized', 'index')).toBe('/docs/corpus/uncategorized/');
  });
});
