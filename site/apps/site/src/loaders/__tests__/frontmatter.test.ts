import { describe, expect, it } from 'vitest';
import {
  isHeldBack,
  normalizeMetadata,
  parseFrontmatter,
  splitFrontmatter,
} from '../lib/frontmatter.ts';
import { CorpusError } from '../lib/types.ts';
import { fixture } from './fixtures.ts';

describe('splitFrontmatter', () => {
  it('separates the YAML block from the body and reports the body start line', () => {
    const { yaml, body, bodyStartLine } = splitFrontmatter('---\ntitle: X\n---\n\n# Heading\n');
    expect(yaml).toBe('title: X');
    expect(body).toBe('\n# Heading\n');
    // `---`(1) `title: X`(2) `---`(3) → body starts on line 4.
    expect(bodyStartLine).toBe(4);
  });

  it('treats a document with no frontmatter as all body', () => {
    const { yaml, body, bodyStartLine } = splitFrontmatter('# Heading\n\ntext\n');
    expect(yaml).toBeNull();
    expect(body).toBe('# Heading\n\ntext\n');
    expect(bodyStartLine).toBe(1);
  });

  it('does not mistake a horizontal rule mid-document for frontmatter', () => {
    const { yaml } = splitFrontmatter('# Heading\n\n---\n\nmore\n');
    expect(yaml).toBeNull();
  });
});

describe('parseFrontmatter', () => {
  it('maps every documented corpus field', () => {
    const source = fixture('published-doc.md');
    const { data } = parseFrontmatter(source.repoPath, source.raw);
    expect(data.title).toBe('Finite State Machines (FSM)');
    expect(data.category).toBe('patterns');
    expect(data.complexity).toBe('advanced');
    expect(data.use_cases).toHaveLength(2);
    expect(data.provenance).toBe('researched');
    expect(data.sources).toEqual(['https://example.invalid/fsm-paper']);
  });

  it('fails with the offending file and line when the YAML will not parse', () => {
    const source = fixture('bad-frontmatter.md');
    let thrown: unknown;
    try {
      parseFrontmatter(source.repoPath, source.raw);
    } catch (error) {
      thrown = error;
    }
    expect(thrown).toBeInstanceOf(CorpusError);
    const error = thrown as CorpusError;
    expect(error.repoPath).toBe('docs/development/bad-frontmatter.md');
    expect(error.line).toBeGreaterThan(1);
    expect(error.message).toMatch(/unparseable frontmatter/);
    expect(error.message).toContain('docs/development/bad-frontmatter.md:');
  });

  it('rejects frontmatter that is not a mapping', () => {
    expect(() => parseFrontmatter('docs/x.md', '---\n- a\n- b\n---\n\nbody\n')).toThrow(CorpusError);
  });
});

describe('isHeldBack', () => {
  it('holds back only on an explicit boolean false', () => {
    expect(isHeldBack({ publish: false })).toBe(true);
    expect(isHeldBack({ publish: true })).toBe(false);
    expect(isHeldBack({ publish: 'false' })).toBe(false);
    expect(isHeldBack({})).toBe(false);
  });
});

describe('normalizeMetadata', () => {
  it('maps summary to description and normalises list fields', () => {
    const meta = normalizeMetadata('docs/development/x.md', {
      title: 'Title',
      category: 'API Design',
      summary: 'A summary.',
      use_cases: ['one', 'two'],
      languages: ['rust'],
    });
    expect(meta.title).toBe('Title');
    expect(meta.description).toBe('A summary.');
    expect(meta.category).toBe('api-design');
    expect(meta.useCases).toEqual(['one', 'two']);
    expect(meta.languages).toEqual(['rust']);
    expect(meta.warnings).toEqual([]);
    expect(meta.inferredTitle).toBe(false);
    expect(meta.inferredCategory).toBe(false);
  });

  it('falls back to filename + uncategorized and warns, rather than dropping the doc', () => {
    const meta = normalizeMetadata('docs/development/no-frontmatter.md', {});
    expect(meta.title).toBe('No Frontmatter');
    expect(meta.category).toBe('uncategorized');
    expect(meta.inferredTitle).toBe(true);
    expect(meta.inferredCategory).toBe(true);
    expect(meta.warnings).toHaveLength(2);
    expect(meta.warnings.join('\n')).toMatch(/no frontmatter `title`/);
    expect(meta.warnings.join('\n')).toMatch(/no frontmatter `category`/);
  });

  it('uses the caller-supplied default category for allowlisted root guides', () => {
    const meta = normalizeMetadata('docs/router.md', {}, { defaultCategory: 'framework-guides' });
    expect(meta.category).toBe('framework-guides');
    expect(meta.title).toBe('Router');
    // Still a warning — a missing category is always worth surfacing.
    expect(meta.inferredCategory).toBe(true);
  });

  it('lets frontmatter override the default category', () => {
    const meta = normalizeMetadata(
      'docs/router.md',
      { category: 'infra' },
      { defaultCategory: 'framework-guides' }
    );
    expect(meta.category).toBe('infra');
    expect(meta.inferredCategory).toBe(false);
  });

  it('coerces a YAML date to an ISO day', () => {
    const meta = normalizeMetadata('docs/x.md', {
      title: 'T',
      category: 'c',
      researched: new Date('2026-08-14T00:00:00Z'),
    });
    expect(meta.researched).toBe('2026-08-14');
  });
});
