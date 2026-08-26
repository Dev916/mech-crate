/**
 * Minimal markdown structure helpers.
 *
 * The corpus is full of fenced code blocks containing text that *looks* like
 * markdown link syntax — Solidity's `new bytes32[](count)`, PHP's
 * `$op['next']($content)`, template placeholders like `${DB_USER}` and `{{ }}`.
 * Rewriting those would corrupt published code samples, so every text
 * transformation masks out code regions first and passes them through untouched.
 */

export interface Region {
  /** Inclusive start offset. */
  start: number;
  /** Exclusive end offset. */
  end: number;
}

/**
 * Offsets of every fenced code block and inline code span in `text`.
 * Regions are returned in ascending order and never overlap.
 */
export function codeRegions(text: string): Region[] {
  const regions: Region[] = [];
  const lines = text.split('\n');

  let offset = 0;
  let openFence: string | null = null;
  let fenceStart = 0;

  for (const line of lines) {
    const lineStart = offset;
    const lineEnd = offset + line.length;
    offset = lineEnd + 1; // +1 for the newline consumed by split

    const fence = /^ {0,3}(`{3,}|~{3,})/.exec(line);

    if (openFence !== null) {
      const closes =
        fence !== null &&
        fence[1]![0] === openFence[0] &&
        fence[1]!.length >= openFence.length &&
        line.slice(fence[0]!.length).trim() === '';
      if (closes) {
        regions.push({ start: fenceStart, end: lineEnd });
        openFence = null;
      }
      continue;
    }

    if (fence !== null) {
      // A backtick fence's info string may not contain backticks; a tilde
      // fence's may. Anything else opens a block.
      const info = line.slice(fence[0]!.length);
      if (fence[1]![0] === '`' && info.includes('`')) {
        // Not a fence — fall through to inline-code handling.
      } else {
        openFence = fence[1]!;
        fenceStart = lineStart;
        continue;
      }
    }

    for (const span of inlineCodeSpans(line)) {
      regions.push({ start: lineStart + span.start, end: lineStart + span.end });
    }
  }

  // Unterminated fence: treat the remainder of the document as code.
  if (openFence !== null) regions.push({ start: fenceStart, end: text.length });

  return regions;
}

/** Inline code spans within a single line, matched by equal-length backtick runs. */
function inlineCodeSpans(line: string): Region[] {
  const runs: Array<{ start: number; len: number }> = [];
  const re = /`+/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(line)) !== null) runs.push({ start: m.index, len: m[0].length });

  const spans: Region[] = [];
  let i = 0;
  while (i < runs.length) {
    const open = runs[i]!;
    let j = i + 1;
    while (j < runs.length && runs[j]!.len !== open.len) j++;
    if (j < runs.length) {
      const close = runs[j]!;
      spans.push({ start: open.start, end: close.start + close.len });
      i = j + 1;
    } else {
      i++;
    }
  }
  return spans;
}

/** True when `offset` falls inside any of the (sorted, non-overlapping) regions. */
export function isInsideRegion(regions: Region[], offset: number): boolean {
  let lo = 0;
  let hi = regions.length - 1;
  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    const r = regions[mid]!;
    if (offset < r.start) hi = mid - 1;
    else if (offset >= r.end) lo = mid + 1;
    else return true;
  }
  return false;
}

/** 1-based line number of `offset` within `text`. */
export function lineAt(text: string, offset: number): number {
  let line = 1;
  const limit = Math.min(offset, text.length);
  for (let i = 0; i < limit; i++) if (text.charCodeAt(i) === 10) line++;
  return line;
}
