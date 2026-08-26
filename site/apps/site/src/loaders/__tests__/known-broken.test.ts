/**
 * The known-broken lane parser, plus a smoke test against the real
 * `tests/KNOWN_BROKEN.md` — the same read `astro build` performs.
 */

import { describe, expect, it } from 'vitest';
import {
  KNOWN_BROKEN_PATH,
  parseKnownBrokenLane,
  renderInline,
} from '../lib/known-broken.ts';
import { loadKnownBrokenLane } from '../known-broken.ts';
import { CorpusError } from '../lib/types.ts';

const TABLE = [
  '# Known-Broken TDD Lane',
  '',
  '## House rules',
  '',
  'Nothing tabular here.',
  '',
  '## Mapping',
  '',
  '| bd id | Test | Where | Asserts (once fixed) | Tier |',
  '|---|---|---|---|---|',
  '| mech-crate-z5i | `upgrade::tests::works` | `crates/mx-lib/src/upgrade/mod.rs` | discovery succeeds | unit |',
  '| mech-crate-4jw | `store::tests::kb_lexical` | `crates/mx-lib/src/corpus/store.rs` | separates by **5×**. *Retire if purged.* | integration (DB) |',
  '',
  '**Scoreboard** (`make test-known-broken`): `14 tests run: 0 passed, 14 failed,',
  '189 skipped`.',
  '',
].join('\n');

describe('parseKnownBrokenLane', () => {
  it('reads every row of the Mapping table', () => {
    const lane = parseKnownBrokenLane(TABLE);
    expect(lane.rows.map((r) => r.bdId)).toEqual(['mech-crate-z5i', 'mech-crate-4jw']);
  });

  it('strips the code fencing from the id, path and tier cells', () => {
    const [first] = parseKnownBrokenLane(TABLE).rows;
    expect(first!.where).toBe('crates/mx-lib/src/upgrade/mod.rs');
    expect(first!.tier).toBe('unit');
    expect(first!.bdId).toBe('mech-crate-z5i');
  });

  it('keeps the markdown in the prose cells for the renderer', () => {
    const [, second] = parseKnownBrokenLane(TABLE).rows;
    expect(second!.test).toBe('`store::tests::kb_lexical`');
    expect(second!.asserts).toContain('**5×**');
  });

  it('picks up the scoreboard counts', () => {
    expect(parseKnownBrokenLane(TABLE).scoreboard).toEqual({
      run: 14,
      passed: 0,
      failed: 14,
      skipped: 189,
    });
  });

  it('skips tables that appear before the Mapping heading', () => {
    const withDecoyTable = TABLE.replace(
      'Nothing tabular here.',
      '| a | b |\n|---|---|\n| 1 | 2 |'
    );
    expect(parseKnownBrokenLane(withDecoyTable).rows).toHaveLength(2);
  });

  it('omits the scoreboard rather than inventing one when the index has none', () => {
    const lane = parseKnownBrokenLane(TABLE.replace(/\*\*Scoreboard[\s\S]*$/, ''));
    expect(lane.scoreboard).toBeUndefined();
    expect(lane.rows).toHaveLength(2);
  });

  it('fails loudly when the Mapping heading is gone', () => {
    expect(() => parseKnownBrokenLane(TABLE.replace('## Mapping', '## Index'))).toThrow(
      CorpusError
    );
  });

  it('fails loudly when the table under Mapping is empty', () => {
    const emptied = TABLE.split('\n')
      .filter((line) => !line.startsWith('| mech-crate-'))
      .join('\n');
    expect(() => parseKnownBrokenLane(emptied)).toThrow(/no rows/);
  });

  it('fails loudly when a row does not start with a bd id', () => {
    expect(() => parseKnownBrokenLane(TABLE.replace('mech-crate-z5i |', 'TBD |'))).toThrow(
      /not a bd issue id/
    );
  });

  it('names the file in every error it raises', () => {
    try {
      parseKnownBrokenLane('# nothing here\n');
      expect.unreachable();
    } catch (error) {
      expect((error as Error).message).toContain(KNOWN_BROKEN_PATH);
    }
  });
});

describe('renderInline', () => {
  it('renders code spans, bold, italic and links', () => {
    expect(renderInline('`mx cf` is **gone**, *for now* — [bd](https://example.com/i)')).toBe(
      '<code>mx cf</code> is <strong>gone</strong>, <em>for now</em> — <a href="https://example.com/i">bd</a>'
    );
  });

  it('escapes HTML inside and outside code spans', () => {
    expect(renderInline('`Vec<T>` & <b>raw</b>')).toBe(
      '<code>Vec&lt;T&gt;</code> &amp; &lt;b&gt;raw&lt;/b&gt;'
    );
  });

  it('leaves markdown syntax inside a code span alone', () => {
    expect(renderInline('`a **b** c`')).toBe('<code>a **b** c</code>');
  });

  it('does not mangle bare numbers that look like sentinels', () => {
    expect(renderInline('measured 2.18 / 0.0062 today')).toBe('measured 2.18 / 0.0062 today');
  });
});

describe('loadKnownBrokenLane against the real repository', () => {
  const lane = loadKnownBrokenLane();

  it('reads tests/KNOWN_BROKEN.md and finds rows', () => {
    expect(lane.rows.length).toBeGreaterThan(0);
  });

  it('gives every row a bd id, a test, a repo path and a tier', () => {
    for (const row of lane.rows) {
      expect(row.bdId).toMatch(/^mech-crate-[a-z0-9]+$/);
      expect(row.test).not.toBe('');
      expect(row.where).toMatch(/^crates\/.+\.rs$/);
      expect(row.tier).not.toBe('');
    }
  });

  it('lists each bd id exactly once', () => {
    const ids = lane.rows.map((row) => row.bdId);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it('fails loudly when the lane index is missing', () => {
    expect(() => loadKnownBrokenLane('/nonexistent-repo-root')).toThrow(/tests\/KNOWN_BROKEN\.md/);
  });
});
