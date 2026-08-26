import { describe, expect, it } from 'vitest';
import { buildCorpus } from '../lib/pipeline.ts';
import { CorpusError } from '../lib/types.ts';
import { fakeRepoFileExists, fixture } from './fixtures.ts';

const GITHUB = 'https://github.com/Dev916/mech-crate/blob/main/';

function build(names: string[], overrides: Record<string, { defaultCategory?: string }> = {}) {
  return buildCorpus(
    names.map((name) => fixture(name, overrides[name] ?? {})),
    { repoFileExists: fakeRepoFileExists, githubBlobBase: GITHUB }
  );
}

describe('buildCorpus — filtering', () => {
  it('drops docs marked `publish: false` and reports them as skipped', () => {
    const result = build(['published-doc.md', 'linked-target.md', 'held-back.md']);
    expect(result.published.map((d) => d.repoPath)).toEqual([
      'docs/development/published-doc.md',
      'docs/development/linked-target.md',
    ]);
    expect(result.skipped).toEqual([
      { repoPath: 'docs/development/held-back.md', reason: 'publish: false' },
    ]);
    // Provably absent: no id, route or body from the held-back doc survives.
    const serialized = JSON.stringify(result.published);
    expect(serialized).not.toContain('held-back');
    expect(serialized).not.toContain('must never reach the built site');
  });
});

describe('buildCorpus — routing', () => {
  it('files each doc under /docs/corpus/<category>/<slug>/ with a sanitized slug', () => {
    const result = build(['published-doc.md', 'linked-target.md', 'mx-mcp~usage.md']);
    expect(result.published.map((d) => d.route)).toEqual([
      '/docs/corpus/patterns/published-doc/',
      '/docs/corpus/patterns/linked-target/',
      '/docs/corpus/process/mx-mcp-usage/',
    ]);
    expect(result.published.map((d) => d.id)).toEqual([
      'docs/corpus/patterns/published-doc',
      'docs/corpus/patterns/linked-target',
      'docs/corpus/process/mx-mcp-usage',
    ]);
  });

  it('applies the allowlist default category to root guides with no frontmatter', () => {
    const result = buildCorpus(
      [
        {
          repoPath: 'docs/router.md',
          raw: fixture('no-frontmatter.md').raw,
          defaultCategory: 'framework-guides',
        },
      ],
      { repoFileExists: fakeRepoFileExists, githubBlobBase: GITHUB }
    );
    expect(result.published[0]!.route).toBe('/docs/corpus/framework-guides/router/');
  });

  it('fails on a route collision rather than silently dropping a page', () => {
    const source = fixture('published-doc.md');
    expect(() =>
      buildCorpus([source, { ...source, repoPath: 'docs/published_doc.md' }], {
        repoFileExists: fakeRepoFileExists,
      })
    ).toThrow(/route collision/);
  });
});

describe('buildCorpus — frontmatter mapping and fallbacks', () => {
  it('maps title/summary onto page data and preserves the rest under `corpus`', () => {
    const [doc] = build(['published-doc.md', 'linked-target.md']).published;
    expect(doc!.data.title).toBe('Finite State Machines (FSM)');
    expect(doc!.data.description).toBe(
      'How to model lifecycles as explicit machines instead of boolean flags.'
    );
    expect(doc!.data.editUrl).toBe(`${GITHUB}docs/development/published-doc.md`);
    expect(doc!.data.corpus).toMatchObject({
      category: 'patterns',
      slug: 'published-doc',
      repoPath: 'docs/development/published-doc.md',
      complexity: 'advanced',
      languages: ['rust', 'typescript'],
      provenance: 'researched',
      researched: '2026-08-14',
      sources: ['https://example.invalid/fsm-paper'],
      inferredTitle: false,
      inferredCategory: false,
    });
    expect(doc!.data.corpus.useCases).toHaveLength(2);
  });

  it('warns — but still publishes — a doc with no title or category', () => {
    const result = build(['no-frontmatter.md']);
    expect(result.published).toHaveLength(1);
    const [doc] = result.published;
    expect(doc!.data.title).toBe('No Frontmatter');
    expect(doc!.category).toBe('uncategorized');
    expect(doc!.data.corpus.inferredTitle).toBe(true);
    expect(doc!.data.corpus.inferredCategory).toBe(true);
    expect(result.warnings).toHaveLength(2);
  });

  it('fails the build on unparseable frontmatter, naming file and line', () => {
    let thrown: unknown;
    try {
      build(['bad-frontmatter.md']);
    } catch (error) {
      thrown = error;
    }
    expect(thrown).toBeInstanceOf(CorpusError);
    expect((thrown as CorpusError).message).toMatch(
      /docs\/development\/bad-frontmatter\.md:\d+: unparseable frontmatter/
    );
  });
});

describe('buildCorpus — link rewriting', () => {
  it('rewrites intra-corpus links, repo links, and leaves the rest alone', () => {
    const [doc] = build(['published-doc.md', 'linked-target.md']).published;
    const body = doc!.body;
    expect(body).toContain('[pattern playbook](/docs/corpus/patterns/linked-target/)');
    expect(body).toContain(`[known-broken lane](${GITHUB}tests/KNOWN_BROKEN.md)`);
    expect(body).toContain('[the intro](#finite-state-machines)');
    expect(body).toContain('[external links](https://example.invalid/)');
    expect(body).toContain('[citations](sediment://file_00000000246c71f5)');
  });

  it('resolves forward references — link index is built before any rewrite', () => {
    // `linked-target.md` links back to `published-doc.md`, which is listed first.
    const published = build(['published-doc.md', 'linked-target.md']).published;
    expect(published[1]!.body).toContain('[FSM](/docs/corpus/patterns/published-doc/)');
  });

  it('passes fenced code, inline code and template syntax through untouched', () => {
    const [doc] = build(['published-doc.md', 'linked-target.md']).published;
    expect(doc!.body).toContain('new bytes32[](count);');
    expect(doc!.body).toContain('`new uint256[](assetIds.length)`');
    expect(doc!.body).toContain('`${DB_USER}` and {{ handlebars }}');
  });

  it('fails the build on a broken intra-corpus link, naming file and line', () => {
    let thrown: unknown;
    try {
      build(['broken-link.md']);
    } catch (error) {
      thrown = error;
    }
    expect(thrown).toBeInstanceOf(CorpusError);
    const error = thrown as CorpusError;
    expect(error.repoPath).toBe('docs/development/broken-link.md');
    // The link lives on line 11 of the fixture.
    expect(error.line).toBe(11);
    expect(error.message).toMatch(/broken link `\.\/does-not-exist-anywhere\.md`/);
  });
});

describe('buildCorpus — secret lint', () => {
  it('fails the whole pipeline when a published doc carries a fake sk- key', () => {
    let thrown: unknown;
    try {
      build(['leaky-secret.md']);
    } catch (error) {
      thrown = error;
    }
    expect(thrown).toBeInstanceOf(CorpusError);
    const error = thrown as CorpusError;
    expect(error.repoPath).toBe('docs/development/leaky-secret.md');
    expect(error.message).toMatch(/secret lint failed/);
    expect(error.message).toMatch(/openai-api-key/);
    // The failure names the line inside the source file, not inside the body.
    expect(error.message).toMatch(/docs\/development\/leaky-secret\.md:12:/);
  });

  it('does not fire on placeholder DSNs that ship in the real corpus', () => {
    const result = build(['mx-mcp~usage.md']);
    expect(result.published).toHaveLength(1);
  });

  it('skips the lint for a held-back doc — only published content is scanned', () => {
    const leaky = fixture('leaky-secret.md');
    const heldBackLeaky = { ...leaky, raw: leaky.raw.replace('# Leak', '# Leak') };
    // Re-mark the same fixture as held back.
    heldBackLeaky.raw = heldBackLeaky.raw.replace(
      'summary: Fixture only',
      'publish: false\nsummary: Fixture only'
    );
    const result = buildCorpus([heldBackLeaky], {
      repoFileExists: fakeRepoFileExists,
      githubBlobBase: GITHUB,
    });
    expect(result.published).toEqual([]);
    expect(result.skipped).toHaveLength(1);
  });
});
