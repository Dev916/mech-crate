/**
 * Shared types for the corpus content pipeline.
 *
 * The pipeline is split in two halves:
 *   - `src/loaders/lib/*`  — pure functions, no filesystem access, unit tested
 *   - `src/loaders/corpus.ts` — the Astro content-collection loader (does the IO)
 *
 * See docs/superpowers/specs/2026-08-20-mechcrate-site-design.md → "Content pipeline".
 */

/** A raw corpus source file handed to the pure pipeline. */
export interface CorpusSource {
  /**
   * Path relative to the repository root, POSIX separators.
   * e.g. `docs/development/appendix-fsm.md`, `docs/router.md`
   */
  repoPath: string;
  /** Raw file contents, frontmatter included. */
  raw: string;
  /**
   * Category applied when the doc's frontmatter omits one AND a caller-supplied
   * default is appropriate (used for the three allowlisted repo-root guides).
   * When absent, a doc with no `category` falls back to `uncategorized` and
   * emits a build warning.
   */
  defaultCategory?: string;
}

/** Frontmatter fields the site understands. Everything else is dropped. */
export interface CorpusFrontmatter {
  title?: unknown;
  category?: unknown;
  summary?: unknown;
  complexity?: unknown;
  languages?: unknown;
  use_cases?: unknown;
  provenance?: unknown;
  researched?: unknown;
  sources?: unknown;
  publish?: unknown;
}

/** Page data handed to Astro's `parseData` (i.e. the Starlight docs schema). */
export interface CorpusPageData {
  /** Starlight: page title. Falls back to the filename when frontmatter has none. */
  title: string;
  /** Starlight: page description. Sourced from frontmatter `summary`. */
  description?: string;
  /** Starlight: link the "Edit page" affordance at the source doc on GitHub. */
  editUrl: string;
  /** Corpus metadata, preserved verbatim for the page template (Task 4). */
  corpus: {
    category: string;
    slug: string;
    repoPath: string;
    sourceUrl: string;
    complexity?: string;
    languages?: string[];
    useCases?: string[];
    provenance?: string;
    researched?: string;
    sources?: string[];
    /** True when title and/or category were synthesised rather than authored. */
    inferredTitle: boolean;
    inferredCategory: boolean;
  };
}

/** One publishable corpus document, fully processed. */
export interface CorpusDoc {
  /** Content-collection entry id — also the Starlight route path. */
  id: string;
  /** Public route, with leading and trailing slash. */
  route: string;
  repoPath: string;
  category: string;
  slug: string;
  /** Markdown body (frontmatter stripped, links rewritten). */
  body: string;
  /** 1-based line in the source file where `body` starts (for error messages). */
  bodyStartLine: number;
  data: CorpusPageData;
}

export interface CorpusSkip {
  repoPath: string;
  reason: string;
}

export interface CorpusBuildResult {
  published: CorpusDoc[];
  skipped: CorpusSkip[];
  /** Non-fatal problems. The loader forwards these to Astro's logger. */
  warnings: string[];
}

/**
 * A fatal pipeline error. Always carries the offending file and, where the
 * problem is positional, the 1-based line — the spec requires build failures to
 * name file/line rather than fail vaguely.
 */
export class CorpusError extends Error {
  readonly repoPath: string;
  readonly line?: number;

  constructor(repoPath: string, message: string, line?: number) {
    super(line === undefined ? `${repoPath}: ${message}` : `${repoPath}:${line}: ${message}`);
    this.name = 'CorpusError';
    this.repoPath = repoPath;
    this.line = line;
  }
}
