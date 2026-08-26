import { describe, expect, it } from 'vitest';
import { rewriteLinks, resolveRepoPath } from '../lib/links.ts';
import { CorpusError } from '../lib/types.ts';
import { fakeRepoFileExists } from './fixtures.ts';

const GITHUB = 'https://github.com/Dev916/mech-crate/blob/main/';

const ROUTES = new Map<string, string>([
  ['docs/development/appendix-fsm.md', '/docs/corpus/patterns/appendix-fsm/'],
  ['docs/development/mx-mcp~usage.md', '/docs/corpus/process/mx-mcp-usage/'],
  ['docs/router.md', '/docs/corpus/framework-guides/router/'],
]);

function rewrite(body: string, repoPath = 'docs/development/source.md') {
  return rewriteLinks({
    repoPath,
    body,
    bodyStartLine: 1,
    routeByRepoPath: ROUTES,
    repoFileExists: fakeRepoFileExists,
    githubBlobBase: GITHUB,
  });
}

describe('resolveRepoPath', () => {
  it('resolves relative targets against the document directory', () => {
    expect(resolveRepoPath('docs/development/a.md', './b.md')).toBe('docs/development/b.md');
    expect(resolveRepoPath('docs/development/a.md', 'b.md')).toBe('docs/development/b.md');
    expect(resolveRepoPath('docs/development/a.md', '../router.md')).toBe('docs/router.md');
    expect(resolveRepoPath('docs/development/a.md', '../../tests/KNOWN_BROKEN.md')).toBe(
      'tests/KNOWN_BROKEN.md'
    );
  });

  it('returns null when the target escapes the repository root', () => {
    expect(resolveRepoPath('docs/development/a.md', '../../../../secrets.md')).toBeNull();
  });
});

describe('rewriteLinks — intra-corpus targets', () => {
  it('rewrites a relative .md link to the target doc site route', () => {
    const { body } = rewrite('See [FSM](./appendix-fsm.md) for details.');
    expect(body).toBe('See [FSM](/docs/corpus/patterns/appendix-fsm/) for details.');
  });

  it('rewrites bare filenames too', () => {
    const { body } = rewrite('See [FSM](appendix-fsm.md).');
    expect(body).toContain('](/docs/corpus/patterns/appendix-fsm/)');
  });

  it('preserves the anchor', () => {
    const { body } = rewrite('[Usage](../development/mx-mcp~usage.md#env-vars)');
    expect(body).toBe('[Usage](/docs/corpus/process/mx-mcp-usage/#env-vars)');
  });

  it('rewrites links that walk up out of docs/development', () => {
    const { body } = rewrite('[Router](../router.md)');
    expect(body).toBe('[Router](/docs/corpus/framework-guides/router/)');
  });
});

describe('rewriteLinks — repo-relative targets', () => {
  it('rewrites a repo file that is not a corpus doc to a GitHub blob URL', () => {
    const { body } = rewrite('[known-broken](../../tests/KNOWN_BROKEN.md)');
    expect(body).toBe(`[known-broken](${GITHUB}tests/KNOWN_BROKEN.md)`);
  });

  it('rewrites non-markdown repo files as well', () => {
    const { body } = rewrite('[packager](../../scripts/package.sh)');
    expect(body).toBe(`[packager](${GITHUB}scripts/package.sh)`);
  });

  it('sends a held-back corpus doc to GitHub rather than a dead route', () => {
    const { body } = rewrite('[held back](./held-back.md)');
    expect(body).toBe(`[held back](${GITHUB}docs/development/held-back.md)`);
  });
});

describe('rewriteLinks — links left alone', () => {
  it.each([
    ['absolute http', '[x](https://example.invalid/a.md)'],
    ['mailto', '[x](mailto:someone@example.invalid)'],
    ['custom scheme', '[x](sediment://file_00000000246c71f5)'],
    ['bare anchor', '[x](#a-heading)'],
    ['site-absolute', '[x](/docs/start/)'],
  ])('leaves %s untouched', (_label, input) => {
    expect(rewrite(input).body).toBe(input);
  });

  it('never rewrites inside fenced code blocks', () => {
    const input = [
      'Text [FSM](./appendix-fsm.md) first.',
      '',
      '```solidity',
      'bytes32[] memory result = new bytes32[](count);',
      'string[] memory args = new string[](appendix-fsm.md);',
      '```',
      '',
      'Trailing [FSM](./appendix-fsm.md).',
    ].join('\n');
    const { body } = rewrite(input);
    expect(body).toContain('new bytes32[](count);');
    expect(body).toContain('new string[](appendix-fsm.md);');
    expect(body.match(/\/docs\/corpus\/patterns\/appendix-fsm\//g)).toHaveLength(2);
  });

  it('never rewrites inside inline code spans', () => {
    const input = 'Use `new uint256[](assetIds.length)` and `[x](./appendix-fsm.md)` verbatim.';
    expect(rewrite(input).body).toBe(input);
  });

  it('passes template syntax through untouched', () => {
    const input = 'Set `postgres://${DB_HOST}` and {{ handlebars }} and ${shell}.';
    expect(rewrite(input).body).toBe(input);
  });
});

describe('rewriteLinks — failures and warnings', () => {
  it('fails the build with file and line for a dangling markdown link', () => {
    const body = ['line one', '', 'a [dangling](./nope.md) link'].join('\n');
    let thrown: unknown;
    try {
      rewriteLinks({
        repoPath: 'docs/development/source.md',
        body,
        bodyStartLine: 6,
        routeByRepoPath: ROUTES,
        repoFileExists: fakeRepoFileExists,
        githubBlobBase: GITHUB,
      });
    } catch (error) {
      thrown = error;
    }
    expect(thrown).toBeInstanceOf(CorpusError);
    const error = thrown as CorpusError;
    // body line 3 + bodyStartLine 6 - 1 = file line 8
    expect(error.line).toBe(8);
    expect(error.repoPath).toBe('docs/development/source.md');
    expect(error.message).toContain('docs/development/source.md:8:');
    expect(error.message).toMatch(/broken link/);
  });

  it('warns but does not fail on a dangling non-markdown link', () => {
    const { body, warnings } = rewrite('[a picture](./diagram.png)');
    expect(body).toBe('[a picture](./diagram.png)');
    expect(warnings).toHaveLength(1);
    expect(warnings[0]).toMatch(/unresolved relative link/);
  });

  it('fails when a markdown link escapes the repository root', () => {
    expect(() => rewrite('[escape](../../../../elsewhere.md)')).toThrow(CorpusError);
  });
});
