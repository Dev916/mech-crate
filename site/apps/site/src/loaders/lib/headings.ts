/**
 * Corpus H1 de-duplication.
 *
 * Every corpus document is authored as a standalone markdown file, so it opens
 * with its own `# Title` line. Starlight renders the frontmatter `title` as the
 * page's `<h1>`, which means the published page carries the heading twice — two
 * `<h1>` elements per corpus route, which is the thing search engines read as a
 * confused document outline.
 *
 * The fix is at the source, not in the renderer: the body's *leading* level-1
 * heading is dropped when it says the same thing as the frontmatter title. That
 * body is the single value three surfaces are built from — the rendered HTML,
 * `llms-full.txt` (and its sixteen splits), and the `.md` twins — so removing
 * the duplicate here removes it everywhere. Agents do not need the line twice
 * either: the twin already opens with `# <title>` of its own
 * (`src/lib/md-twin.ts`), and `llms-full.txt`'s separator block names the
 * document before its body.
 *
 * "Says the same thing" is deliberately conservative, because a wrong strip
 * silently deletes a real heading:
 *
 *   - only the FIRST non-blank line of the body is considered, and only when it
 *     is an ATX level-1 heading (`# `). A document that opens with prose, a
 *     fence, or an `##` keeps everything;
 *   - the comparison is on normalised *words* — lowercased, punctuation and
 *     markdown emphasis flattened to separators — so `**Business Logic
 *     Placement & Model-Level Architecture**` matches its plainer title;
 *   - one word list must be a whole-word PREFIX of the other, in either
 *     direction. That is what lets a body heading of `LLM Token & Cache
 *     Efficiency Engineering` be dropped in favour of the longer title `…for
 *     Agentic Coding`, and equally lets a title that is the shorter of the two
 *     win. A heading that merely *starts differently* — `Part Two: Streams Deep
 *     Dive` against a title of `Streams Deep Dive` — is left alone rather than
 *     guessed at;
 *   - the one exception to that last point is a leading *appendix label*.
 *     Fourteen corpus documents were split out of a larger guide and kept the
 *     label they had as a chapter: `# Appendix: FRP in Rust` under a title of
 *     `FRP in Rust`, `# Appendix K: Algebraic Effects & Optics` under
 *     `Algebraic Effects & Optics`. The label is filing metadata, not part of
 *     the heading, so it is discarded before the prefix comparison — but only
 *     when it is genuinely a label: the word `Appendix`, an optional short
 *     designator, and a punctuation separator. `Solana RPC appendix (LLM-safe)`
 *     keeps its word, because there the word is prose.
 *
 * See docs/superpowers/specs/2026-09-05-seo-geo-design.md → "6. Polish".
 */

/**
 * A heading or title reduced to comparable words: lowercased, with every run of
 * non-alphanumeric characters treated as a separator.
 *
 * Emphasis markers, colons, em dashes, ampersands, Pandoc `{#anchor}` suffixes
 * and emoji all collapse to word breaks, which is what makes
 * `🍎 Apple Design Quick Implementation Guide for LLMs` and
 * `Apple Design Quick Implementation Guide` comparable at all.
 */
export function headingWords(text: string): string[] {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, ' ')
    .trim()
    .split(' ')
    .filter((word) => word !== '');
}

/** Is `a` a whole-word prefix of `b`? */
function isWordPrefix(a: readonly string[], b: readonly string[]): boolean {
  if (a.length > b.length) return false;
  return a.every((word, index) => word === b[index]);
}

/**
 * A leading appendix label: the word itself, an optional short designator
 * (`K`, `A1`, `3`), and a punctuation separator that proves it is a label
 * rather than the first word of a sentence. Leading emphasis markers are
 * tolerated, because `**Appendix: …**` is the same heading in bold.
 */
const APPENDIX_LABEL = /^[\s*_]*appendix(?:\s+[a-z0-9][a-z0-9.]{0,3})?\s*[:—–-]\s*/i;

/**
 * `Appendix K: Algebraic Effects & Optics` → `Algebraic Effects & Optics`.
 *
 * Returns the text unchanged when there is no label to discard.
 */
export function withoutAppendixLabel(heading: string): string {
  return heading.replace(APPENDIX_LABEL, '');
}

/**
 * Do a body heading and a frontmatter title say the same thing?
 *
 * True when either word list is a whole-word prefix of the other — tried on the
 * heading as written, and again with a leading appendix label discarded. Empty
 * lists never match: a document with no title, or a heading that normalises to
 * nothing (an `# Appendix:` with no subject, say), must keep whatever it has.
 */
export function headingMatchesTitle(heading: string, title: string): boolean {
  const t = headingWords(title);
  if (t.length === 0) return false;

  const unlabelled = withoutAppendixLabel(heading);
  const candidates = unlabelled === heading ? [heading] : [heading, unlabelled];

  return candidates.some((candidate) => {
    const h = headingWords(candidate);
    if (h.length === 0) return false;
    return isWordPrefix(h, t) || isWordPrefix(t, h);
  });
}

export interface StrippedBody {
  /** The body, with the duplicate heading (and the blank lines under it) gone. */
  body: string;
  /**
   * How many leading lines were removed. Callers that track where the body
   * starts in its source file add this to keep that offset honest.
   */
  linesRemoved: number;
}

/** `# Heading text` → `Heading text`, or null when the line is not an ATX H1. */
function atxH1Text(line: string): string | null {
  // `#` followed by whitespace, per CommonMark; up to three leading spaces are
  // still a heading, four make it an indented code block. A closing sequence of
  // `#`s is optional and not part of the text.
  const match = /^ {0,3}#[ \t]+(.*)$/.exec(line);
  if (match === null) return null;
  return match[1]!.replace(/[ \t]+#+[ \t]*$/, '').trim();
}

/**
 * Drop the body's leading level-1 heading when it duplicates `title`.
 *
 * Returns the body unchanged (and `linesRemoved: 0`) in every case the rule does
 * not clearly apply — no heading, a heading below level 1, or a heading that
 * does not match. Blank lines immediately following the removed heading go with
 * it, so the body still starts at content rather than at whitespace.
 */
export function stripDuplicateH1(body: string, title: string): StrippedBody {
  const lines = body.split('\n');

  // The first line with anything on it. Leading blank lines are not content, so
  // a heading below them is still "leading".
  let first = 0;
  while (first < lines.length && lines[first]!.trim() === '') first++;
  if (first === lines.length) return { body, linesRemoved: 0 };

  const heading = atxH1Text(lines[first]!);
  if (heading === null) return { body, linesRemoved: 0 };
  if (!headingMatchesTitle(heading, title)) return { body, linesRemoved: 0 };

  // Take the heading and the blank run under it; keep the content that follows.
  let next = first + 1;
  while (next < lines.length && lines[next]!.trim() === '') next++;

  return { body: lines.slice(next).join('\n'), linesRemoved: next };
}
