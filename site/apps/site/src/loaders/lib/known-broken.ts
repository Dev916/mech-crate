/**
 * The known-broken TDD lane, parsed out of `tests/KNOWN_BROKEN.md`.
 *
 * The lane index is maintained in the repository, not here: a row is added when
 * a defect gets a red test and removed when the fix lands. Publishing a
 * hand-copied version of that table would guarantee it goes stale, so the site
 * reads the real file at build time and renders whatever it says today.
 *
 * Same contract as the corpus pipeline (`./pipeline.ts`): pure functions here,
 * filesystem access in `src/loaders/known-broken.ts`, and every failure is a
 * `CorpusError` naming the file — a missing or malformed lane index fails the
 * build rather than publishing an empty table.
 */

import { CorpusError } from './types.ts';

/** Repo-relative path of the lane index. */
export const KNOWN_BROKEN_PATH = 'tests/KNOWN_BROKEN.md';

/** Heading the lane table lives under. */
const MAPPING_HEADING = /^##\s+Mapping\s*$/;

/** A bd issue id, e.g. `mech-crate-z5i`. */
const BD_ID = /^[a-z][a-z0-9]*(?:-[a-z0-9]+)+$/;

export interface KnownBrokenRow {
  /** bd issue id, e.g. `mech-crate-z5i`. */
  bdId: string;
  /** Test name, as written in the index (may carry markdown). */
  test: string;
  /** Repo-relative path of the file holding the test, backticks stripped. */
  where: string;
  /** What the test asserts once the defect is fixed (may carry markdown). */
  asserts: string;
  /** `unit`, `integration`, `integration (DB)`, … */
  tier: string;
}

/** The `make test-known-broken` scoreboard recorded in the index. */
export interface KnownBrokenScoreboard {
  run: number;
  passed: number;
  failed: number;
  skipped: number;
}

export interface KnownBrokenLane {
  rows: KnownBrokenRow[];
  scoreboard?: KnownBrokenScoreboard;
}

/**
 * Parse the lane index. Throws `CorpusError` when the file is not the shape the
 * page renders — an index whose table moved or emptied is a bookkeeping failure
 * worth breaking the build over.
 */
export function parseKnownBrokenLane(raw: string): KnownBrokenLane {
  const lines = raw.split('\n');

  const headingIndex = lines.findIndex((line) => MAPPING_HEADING.test(line));
  if (headingIndex === -1) {
    throw new CorpusError(KNOWN_BROKEN_PATH, 'no `## Mapping` heading — the lane table cannot be located');
  }

  // The first pipe table after the heading is the lane table.
  let start = headingIndex + 1;
  while (start < lines.length && !isTableLine(lines[start])) {
    if (/^#{1,6}\s/.test(lines[start] ?? '')) {
      throw new CorpusError(
        KNOWN_BROKEN_PATH,
        'no table under `## Mapping` — the next heading arrived first',
        start + 1
      );
    }
    start++;
  }
  if (start >= lines.length) {
    throw new CorpusError(KNOWN_BROKEN_PATH, 'no table under `## Mapping`', headingIndex + 1);
  }

  let end = start;
  while (end < lines.length && isTableLine(lines[end])) end++;

  const table = lines.slice(start, end);
  const header = splitRow(table[0]!);
  if (header.length < 5 || header[0]!.toLowerCase() !== 'bd id') {
    throw new CorpusError(
      KNOWN_BROKEN_PATH,
      `unexpected lane table header \`${header.join(' | ')}\` — expected \`bd id | Test | Where | Asserts … | Tier\``,
      start + 1
    );
  }

  const rows: KnownBrokenRow[] = [];
  // table[1] is the `|---|---|` delimiter.
  for (let i = 2; i < table.length; i++) {
    const line = start + i + 1;
    const cells = splitRow(table[i]!);
    if (cells.length < 5) {
      throw new CorpusError(
        KNOWN_BROKEN_PATH,
        `lane row has ${cells.length} cells, expected at least 5`,
        line
      );
    }
    const bdId = stripCode(cells[0]!);
    if (!BD_ID.test(bdId)) {
      throw new CorpusError(KNOWN_BROKEN_PATH, `\`${bdId}\` is not a bd issue id`, line);
    }
    rows.push({
      bdId,
      test: cells[1]!,
      where: stripCode(cells[2]!),
      asserts: cells[3]!,
      tier: stripCode(cells[4]!),
    });
  }

  if (rows.length === 0) {
    throw new CorpusError(KNOWN_BROKEN_PATH, 'the lane table has no rows', start + 1);
  }

  return { rows, ...(parseScoreboard(raw) ?? {}) };
}

function parseScoreboard(raw: string): { scoreboard: KnownBrokenScoreboard } | undefined {
  const m = /(\d+)\s+tests?\s+run:\s*(\d+)\s+passed,\s*(\d+)\s+failed,\s*(\d+)\s+skipped/s.exec(
    raw.replace(/\n/g, ' ')
  );
  if (m === null) return undefined;
  return {
    scoreboard: {
      run: Number(m[1]),
      passed: Number(m[2]),
      failed: Number(m[3]),
      skipped: Number(m[4]),
    },
  };
}

function isTableLine(line: string | undefined): boolean {
  return line !== undefined && line.trimStart().startsWith('|');
}

/** Cells of a pipe-table row, leading/trailing delimiters dropped. */
function splitRow(line: string): string[] {
  const trimmed = line.trim().replace(/^\|/, '').replace(/\|$/, '');
  return trimmed.split('|').map((cell) => cell.trim());
}

/** `` `foo` `` → `foo`. */
function stripCode(cell: string): string {
  return cell.replace(/^`+|`+$/g, '').trim();
}

/**
 * Render the inline markdown that appears in lane cells (code spans, bold,
 * italic, links) to HTML.
 *
 * Everything is HTML-escaped first and code-span contents are held out of the
 * later passes, so a backticked `<T>` or `&` survives verbatim and no cell can
 * inject markup. Deliberately not a markdown library: the input is one table in
 * one repository file, and a 40-line renderer is cheaper to reason about than a
 * dependency.
 */
export function renderInline(text: string): string {
  const codes: string[] = [];
  // NUL-delimited sentinel: it cannot occur in the source file, survives HTML
  // escaping untouched, and carries no markdown meaning of its own.
  const held = text.replace(/`([^`]+)`/g, (_all, code: string) => {
    codes.push(code);
    return `\u0000${codes.length - 1}\u0000`;
  });

  let html = escapeHtml(held)
    .replace(/\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g, '<a href="$2">$1</a>')
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    .replace(/\*([^*]+)\*/g, '<em>$1</em>');

  html = html.replace(/\u0000(\d+)\u0000/g, (_all, index: string) => {
    return `<code>${escapeHtml(codes[Number(index)] ?? '')}</code>`;
  });

  return html;
}

function escapeHtml(text: string): string {
  return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}
