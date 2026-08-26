import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { CorpusSource } from '../lib/types.ts';

const FIXTURE_DIR = join(dirname(fileURLToPath(import.meta.url)), '..', '__fixtures__');

/** Read a fixture as a corpus source rooted at `docs/development/<name>`. */
export function fixture(name: string, overrides: Partial<CorpusSource> = {}): CorpusSource {
  return {
    repoPath: `docs/development/${name}`,
    raw: readFileSync(join(FIXTURE_DIR, name), 'utf8'),
    ...overrides,
  };
}

/**
 * Stand-in for the repository filesystem. Only these paths "exist" — enough to
 * exercise the corpus / repo-file / dangling branches of the link rewriter.
 */
export const FAKE_REPO_FILES = new Set([
  'tests/KNOWN_BROKEN.md',
  'scripts/package.sh',
  'docs/development/held-back.md',
]);

export const fakeRepoFileExists = (repoPath: string): boolean => FAKE_REPO_FILES.has(repoPath);
