/**
 * Unit tests for the corpus H1 de-duplication (`src/loaders/lib/headings.ts`).
 *
 * Two halves, and the second is the one that matters: a strip that fires when it
 * should not silently deletes a real heading from three published surfaces at
 * once. So the "leaves it alone" cases are asserted at least as hard as the
 * "strips it" cases, and the last block runs the rule over the real corpus to
 * check the outcome against the repository's actual contents rather than a
 * fixture's.
 */

import { describe, expect, it } from 'vitest';

import {
  headingMatchesTitle,
  headingWords,
  stripDuplicateH1,
  withoutAppendixLabel,
} from '../lib/headings.ts';
import { buildCorpus } from '../lib/pipeline.ts';
import { collectCorpusSources, defaultRepoRoot, repoFileExistsIn } from '../lib/sources.ts';

describe('headingWords', () => {
  it('lowercases and splits on every run of non-alphanumerics', () => {
    expect(headingWords('Category Theory: Mathematical Composition')).toEqual([
      'category',
      'theory',
      'mathematical',
      'composition',
    ]);
  });

  it('flattens markdown emphasis, ampersands and em dashes to word breaks', () => {
    expect(headingWords('**Living Pattern Playbook — Rust, Laravel & Nuxt**')).toEqual([
      'living',
      'pattern',
      'playbook',
      'rust',
      'laravel',
      'nuxt',
    ]);
  });

  it('drops emoji and Pandoc anchor suffixes', () => {
    expect(headingWords('🍎 Apple Design {#apple-design}')).toEqual([
      'apple',
      'design',
      'apple',
      'design',
    ]);
  });

  it('returns nothing for a string with no word characters', () => {
    expect(headingWords('— · —')).toEqual([]);
  });
});

describe('withoutAppendixLabel', () => {
  it('discards a bare `Appendix:` label', () => {
    expect(withoutAppendixLabel('Appendix: FRP in Rust')).toBe('FRP in Rust');
  });

  it('discards a designator with the label', () => {
    expect(withoutAppendixLabel('Appendix K: Algebraic Effects & Optics')).toBe(
      'Algebraic Effects & Optics'
    );
    expect(withoutAppendixLabel('Appendix A.1 — Streams')).toBe('Streams');
  });

  it('accepts an em dash, en dash or hyphen as the separator', () => {
    expect(withoutAppendixLabel('Appendix — Streams')).toBe('Streams');
    expect(withoutAppendixLabel('Appendix – Streams')).toBe('Streams');
    expect(withoutAppendixLabel('Appendix - Streams')).toBe('Streams');
  });

  it('sees through leading emphasis markers', () => {
    expect(withoutAppendixLabel('**Appendix: FRP in PHP**')).toBe('FRP in PHP**');
  });

  it('keeps a heading whose label has no separator', () => {
    // Without punctuation there is no way to tell a label from a first word.
    expect(withoutAppendixLabel('Appendix Streams Deep Dive')).toBe(
      'Appendix Streams Deep Dive'
    );
  });

  it('keeps the word when it is not leading', () => {
    expect(withoutAppendixLabel('Solana RPC appendix (LLM-safe)')).toBe(
      'Solana RPC appendix (LLM-safe)'
    );
  });

  it('keeps an unrelated label', () => {
    expect(withoutAppendixLabel('Part Two: Streams Deep Dive')).toBe(
      'Part Two: Streams Deep Dive'
    );
  });
});

describe('headingMatchesTitle', () => {
  it('matches identical text through different punctuation', () => {
    expect(
      headingMatchesTitle(
        '**Business Logic Placement & Model-Level Architecture**',
        'Business Logic Placement & Model-Level Architecture'
      )
    ).toBe(true);
  });

  it('matches when the heading is a word-prefix of a longer title', () => {
    // The audit's example: the body says less than the frontmatter does.
    expect(
      headingMatchesTitle(
        'LLM Token & Cache Efficiency Engineering',
        'LLM Token & Cache Efficiency Engineering for Agentic Coding'
      )
    ).toBe(true);
  });

  it('matches when the title is a word-prefix of a longer heading', () => {
    expect(
      headingMatchesTitle(
        'Apple-Inspired UI/UX Design Guidelines for LLMs',
        'Apple-Inspired UI/UX Design Guidelines'
      )
    ).toBe(true);
  });

  it('matches through a leading appendix label', () => {
    // The label is filing metadata from the guide these documents were split
    // out of; the heading under it is the title verbatim.
    expect(
      headingMatchesTitle('Appendix: Streams Deep Dive', 'Streams Deep Dive')
    ).toBe(true);
    expect(
      headingMatchesTitle('Appendix K: Algebraic Effects & Optics', 'Algebraic Effects & Optics')
    ).toBe(true);
  });

  it('refuses a heading that merely starts differently', () => {
    // Any other prefix makes the title a *suffix*, not a prefix. Guessing here
    // is how a real heading gets deleted, so the rule declines.
    expect(
      headingMatchesTitle('Part Two: Streams Deep Dive', 'Streams Deep Dive')
    ).toBe(false);
  });

  it('refuses a heading whose appendix word is prose, not a label', () => {
    expect(
      headingMatchesTitle('Solana RPC appendix (LLM-safe)', 'Solana RPC (LLM-safe)')
    ).toBe(false);
  });

  it('refuses a heading that is nothing but an appendix label', () => {
    // Discarding the label leaves no words, so there is nothing to match on.
    expect(headingMatchesTitle('Appendix:', 'Streams Deep Dive')).toBe(false);
  });

  it('refuses two unrelated headings', () => {
    expect(
      headingMatchesTitle('LLM Build Instructions', 'Astro + Vue Frontend Build Scaffold')
    ).toBe(false);
  });

  it('refuses a partial-word overlap', () => {
    // Prefixes are whole words: `Cat` is not the start of `Category`.
    expect(headingMatchesTitle('Cat Theory', 'Category Theory')).toBe(false);
  });

  it('refuses when either side normalises to nothing', () => {
    expect(headingMatchesTitle('', 'Anything')).toBe(false);
    expect(headingMatchesTitle('Anything', '   ')).toBe(false);
  });
});

describe('stripDuplicateH1', () => {
  it('removes the heading and the blank run under it', () => {
    const out = stripDuplicateH1('# Docker Assembly Guide\n\nBody text.\n', 'Docker Assembly Guide');
    expect(out.body).toBe('Body text.\n');
    expect(out.linesRemoved).toBe(2);
  });

  it('counts blank lines above the heading as removed too', () => {
    const out = stripDuplicateH1('\n\n# Title\n\nBody.\n', 'Title');
    expect(out.body).toBe('Body.\n');
    expect(out.linesRemoved).toBe(4);
  });

  it('tolerates a closing hash sequence', () => {
    expect(stripDuplicateH1('# Title #\n\nBody.\n', 'Title').body).toBe('Body.\n');
  });

  it('removes a heading that matches once its appendix label is discarded', () => {
    const out = stripDuplicateH1('# Appendix: Streams Deep Dive\n\nBody.\n', 'Streams Deep Dive');
    expect(out).toEqual({ body: 'Body.\n', linesRemoved: 2 });
  });

  it('leaves a non-matching heading in place', () => {
    const body = '# Part Two: Streams Deep Dive\n\nBody.\n';
    expect(stripDuplicateH1(body, 'Streams Deep Dive')).toEqual({ body, linesRemoved: 0 });
  });

  it('leaves an H2 in place even when it matches', () => {
    const body = '## Title\n\nBody.\n';
    expect(stripDuplicateH1(body, 'Title')).toEqual({ body, linesRemoved: 0 });
  });

  it('leaves a matching heading that is not the first content in place', () => {
    // Only the *leading* heading is the duplicate of the page title; one further
    // down is a section of the document.
    const body = 'Intro paragraph.\n\n# Title\n\nBody.\n';
    expect(stripDuplicateH1(body, 'Title')).toEqual({ body, linesRemoved: 0 });
  });

  it('leaves a heading inside an opening code fence in place', () => {
    // The fence is the first content, so nothing is stripped — no need to mask
    // code regions, because only line one is ever examined.
    const body = '```md\n# Title\n```\n';
    expect(stripDuplicateH1(body, 'Title')).toEqual({ body, linesRemoved: 0 });
  });

  it('leaves a `#`-without-space line in place — that is not a heading', () => {
    const body = '#Title\n\nBody.\n';
    expect(stripDuplicateH1(body, 'Title')).toEqual({ body, linesRemoved: 0 });
  });

  it('leaves an empty body alone', () => {
    expect(stripDuplicateH1('', 'Title')).toEqual({ body: '', linesRemoved: 0 });
    expect(stripDuplicateH1('\n\n', 'Title')).toEqual({ body: '\n\n', linesRemoved: 0 });
  });

  it('survives a document that is nothing but its title', () => {
    expect(stripDuplicateH1('# Title\n', 'Title')).toEqual({ body: '', linesRemoved: 2 });
  });
});

describe('the real corpus', () => {
  const repoRoot = defaultRepoRoot();
  const { published } = buildCorpus(collectCorpusSources(repoRoot), {
    repoFileExists: repoFileExistsIn(repoRoot),
  });

  /** The body's first non-blank line, or '' for an empty body. */
  function firstLine(body: string): string {
    return body.split('\n').find((line) => line.trim() !== '') ?? '';
  }

  it('publishes no document whose body still opens with its own title', () => {
    const offenders = published
      .filter((doc) => {
        const line = firstLine(doc.body);
        const match = /^ {0,3}#[ \t]+(.*)$/.exec(line);
        return match !== null && headingMatchesTitle(match[1]!, doc.data.title);
      })
      .map((doc) => doc.repoPath);

    expect(offenders).toEqual([]);
  });

  it('actually fires — a majority of the corpus opened with a duplicate heading', () => {
    // Guards against the rule quietly becoming a no-op: this is the fix's whole
    // reason to exist, so it is asserted as a fact about the corpus.
    const sources = collectCorpusSources(repoRoot);
    const raw = new Map(sources.map((s) => [s.repoPath, s.raw]));

    const stripped = published.filter((doc) => {
      const source = raw.get(doc.repoPath)!;
      const body = source.replace(/^﻿?---\r?\n[\s\S]*?\r?\n?---[ \t]*(\r?\n|$)/, '');
      return firstLine(body) !== firstLine(doc.body);
    });

    expect(stripped.length).toBeGreaterThan(published.length / 2);
  });

  it('leaves every document with a body', () => {
    for (const doc of published) {
      expect(doc.body.trim(), doc.repoPath).not.toBe('');
    }
  });
});
