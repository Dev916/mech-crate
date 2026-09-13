import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { afterAll, describe, expect, it } from 'vitest';
import { CORPUS_DIR, collectCorpusSources } from '../lib/sources.ts';

/**
 * The flat scan is a CONTRACT, not an accident: subdirectories of
 * `docs/development` (e.g. `repos/`, the internal repo profiles) are corpus-only
 * and must never be collected for publication on mechcrate.dev.
 * See docs/superpowers/specs/2026-09-01-repo-profiles-corpus-design.md §3 D3.
 */
const root = mkdtempSync(join(tmpdir(), 'corpus-sources-'));
afterAll(() => rmSync(root, { recursive: true, force: true }));

describe('collectCorpusSources — scope contract', () => {
  it('collects flat docs/development *.md files but never subdirectories', () => {
    mkdirSync(join(root, CORPUS_DIR, 'repos'), { recursive: true });
    writeFileSync(join(root, CORPUS_DIR, 'flat-doc.md'), '# flat\n');
    writeFileSync(
      join(root, CORPUS_DIR, 'repos', 'private-profile.md'),
      '---\ntitle: "x (Repo Profile)"\ncategory: repos\n---\n\nmust never reach the built site\n'
    );
    writeFileSync(join(root, CORPUS_DIR, 'notes.txt'), 'not markdown\n');

    const paths = collectCorpusSources(root).map((s) => s.repoPath);

    expect(paths).toContain(`${CORPUS_DIR}/flat-doc.md`);
    expect(paths.some((p) => p.includes('/repos/'))).toBe(false);
    expect(paths.some((p) => p.endsWith('.txt'))).toBe(false);
  });
});
