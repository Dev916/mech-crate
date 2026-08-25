/**
 * Build-time read of the known-broken lane index.
 *
 * The IO half of `lib/known-broken.ts`, mirroring how `corpus.ts` relates to
 * `lib/pipeline.ts`. Called from `src/components/KnownBrokenLane.astro`, so it
 * runs during `astro build` — a missing or malformed `tests/KNOWN_BROKEN.md`
 * fails the build with the offending path rather than publishing a page that
 * claims there are no open defects.
 */

import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

import {
  KNOWN_BROKEN_PATH,
  parseKnownBrokenLane,
  type KnownBrokenLane,
} from './lib/known-broken.ts';
import { defaultRepoRoot } from './lib/sources.ts';
import { CorpusError } from './lib/types.ts';

export { KNOWN_BROKEN_PATH } from './lib/known-broken.ts';
export type { KnownBrokenLane, KnownBrokenRow } from './lib/known-broken.ts';

/** Read and parse `<repoRoot>/tests/KNOWN_BROKEN.md`. Throws when it is absent. */
export function loadKnownBrokenLane(repoRoot: string = defaultRepoRoot()): KnownBrokenLane {
  const absolute = join(repoRoot, KNOWN_BROKEN_PATH);
  if (!existsSync(absolute)) {
    throw new CorpusError(
      KNOWN_BROKEN_PATH,
      `not found under the repository root (${repoRoot}). The known-broken page is rendered from ` +
        'this file at build time; publishing an empty lane would understate the open defects, so ' +
        'the build stops here.'
    );
  }
  return parseKnownBrokenLane(readFileSync(absolute, 'utf8'));
}
