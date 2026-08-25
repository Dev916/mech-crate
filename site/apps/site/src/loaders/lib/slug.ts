/**
 * Slug and route construction for corpus pages.
 *
 * Corpus filenames are inconsistent by history: `mx-mcp~usage.md`,
 * `MX_RUST_CLI_AND_MCP_SERVER.md`, `appendix-fsm.md`. Routes must be stable,
 * lowercase and URL-safe, so every filename stem goes through `sanitizeSlug`.
 */

/** Route prefix owned by the corpus loader (collection-relative, no leading slash). */
export const CORPUS_ID_PREFIX = 'docs/corpus';

/** Category used when a doc declares none. */
export const UNCATEGORIZED = 'uncategorized';

/**
 * Lowercase, collapse separators, strip anything that is not `[a-z0-9-]`.
 *
 *   `mx-mcp~usage`              → `mx-mcp-usage`
 *   `MX_RUST_CLI_AND_MCP_SERVER`→ `mx-rust-cli-and-mcp-server`
 *   `Apple  Design   Guide`     → `apple-design-guide`
 */
export function sanitizeSlug(input: string): string {
  return input
    .normalize('NFKD')
    .toLowerCase()
    // `~`, whitespace, `_`, `.` and any other separator-ish char become hyphens.
    .replace(/[^a-z0-9]+/g, '-')
    // Collapse runs and trim.
    .replace(/-{2,}/g, '-')
    .replace(/^-+|-+$/g, '');
}

/** The filename stem (no directory, no `.md`) for a repo-relative path. */
export function fileStem(repoPath: string): string {
  const base = repoPath.split('/').pop() ?? repoPath;
  return base.replace(/\.[^.]+$/, '');
}

/**
 * Human-facing title derived from a filename, used only when frontmatter has no
 * `title` (the spec's "falls back to filename" rule). The build warns whenever
 * this fires so a new doc cannot be silently mis-titled.
 *
 *   `docs/docs-command.md`  → `Docs Command`
 *   `docs/development/INDEX.md` → `Index`
 */
export function titleFromFilename(repoPath: string): string {
  const words = sanitizeSlug(fileStem(repoPath)).split('-').filter(Boolean);
  if (words.length === 0) return fileStem(repoPath);
  return words.map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
}

/** Content-collection entry id, e.g. `docs/corpus/theory/appendix-fsm`. */
export function corpusEntryId(category: string, slug: string): string {
  return `${CORPUS_ID_PREFIX}/${category}/${slug}`;
}

/**
 * Public route, e.g. `/docs/corpus/theory/appendix-fsm/`.
 *
 * Astro collapses a trailing `index` segment into its directory, so a doc whose
 * filename stem is `index` (e.g. `docs/development/INDEX.md`) is served at
 * `/docs/corpus/<category>/`. Intra-corpus links must point at the collapsed
 * form or they 404.
 */
export function corpusRoute(category: string, slug: string): string {
  return slug === 'index'
    ? `/${CORPUS_ID_PREFIX}/${category}/`
    : `/${corpusEntryId(category, slug)}/`;
}
