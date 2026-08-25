/**
 * Which repository files make up the corpus, and how to find the repo root from
 * inside the nested `site/apps/site` app.
 */

import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { CorpusSource } from './types.ts';

/** Directory whose `*.md` files are all corpus docs. */
export const CORPUS_DIR = 'docs/development';

/**
 * Repo-root guides published alongside the corpus. Explicit allowlist, not a
 * glob: `docs/` also holds internal point-in-time thinking that never ships
 * (`architecture-review-*.md`, `product-structure.md`) — see the spec's
 * "Never published" list.
 */
export const ROOT_GUIDE_ALLOWLIST = [
  'docs/router.md',
  'docs/cloudflare.md',
  'docs/docs-command.md',
] as const;

/** Category applied to allowlisted root guides that declare none themselves. */
export const ROOT_GUIDE_CATEGORY = 'framework-guides';

/**
 * Walk up from `startDir` until a directory looks like the mech-crate root
 * (it holds the corpus directory and a `Cargo.toml`). Falls back to the fixed
 * `../../../` hop from the app root so the loader still works in a stripped
 * checkout.
 */
export function findRepoRoot(startDir: string): string {
  let dir = startDir;
  for (let depth = 0; depth < 12; depth++) {
    if (existsSync(join(dir, CORPUS_DIR)) && existsSync(join(dir, 'Cargo.toml'))) return dir;
    const parent = dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  // `<repo>/site/apps/site` → `<repo>`
  return resolve(startDir, '../../..');
}

/** Repo root resolved relative to this module, independent of `process.cwd()`. */
export function defaultRepoRoot(): string {
  // …/site/apps/site/src/loaders/lib → …/site/apps/site
  const appRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
  return findRepoRoot(appRoot);
}

/** Read every corpus source file under `repoRoot`, in stable order. */
export function collectCorpusSources(repoRoot: string): CorpusSource[] {
  const sources: CorpusSource[] = [];

  const corpusDir = join(repoRoot, CORPUS_DIR);
  const entries = existsSync(corpusDir)
    ? readdirSync(corpusDir, { withFileTypes: true })
        .filter((entry) => entry.isFile() && entry.name.endsWith('.md'))
        .map((entry) => entry.name)
        .sort((a, b) => a.localeCompare(b))
    : [];

  for (const name of entries) {
    const repoPath = `${CORPUS_DIR}/${name}`;
    sources.push({ repoPath, raw: readFileSync(join(repoRoot, repoPath), 'utf8') });
  }

  for (const repoPath of ROOT_GUIDE_ALLOWLIST) {
    const absolute = join(repoRoot, repoPath);
    if (!existsSync(absolute)) continue;
    sources.push({
      repoPath,
      raw: readFileSync(absolute, 'utf8'),
      defaultCategory: ROOT_GUIDE_CATEGORY,
    });
  }

  return sources;
}

/** `repoFileExists` predicate bound to a repository root. */
export function repoFileExistsIn(repoRoot: string): (repoPath: string) => boolean {
  return (repoPath) => existsSync(join(repoRoot, repoPath));
}
