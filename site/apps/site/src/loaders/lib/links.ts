/**
 * Relative-link rewriting for corpus docs.
 *
 * The corpus is authored to be read in the repo, so its links are relative file
 * paths. On the site they must become either:
 *   - a site route      — the target is another published corpus doc
 *   - a GitHub blob URL — the target is some other file in the repo
 *                         (`tests/KNOWN_BROKEN.md`, `../scripts/…`, or a corpus
 *                         doc held back with `publish: false`)
 *   - a build failure   — a `.md` target that resolves to nothing at all
 *
 * Absolute URLs, custom schemes (`sediment://…`), root-relative paths and bare
 * anchors are left exactly as authored. Code regions are never touched.
 */

import { codeRegions, isInsideRegion, lineAt } from './markdown.ts';
import { CorpusError } from './types.ts';

export const GITHUB_BLOB_BASE = 'https://github.com/Dev916/mech-crate/blob/main/';

export interface RewriteOptions {
  /** Repo-relative path of the document being rewritten. */
  repoPath: string;
  /** Markdown body (frontmatter already stripped). */
  body: string;
  /** 1-based line in the source file where `body` starts. */
  bodyStartLine: number;
  /** repo-relative path → published site route, for every publishable corpus doc. */
  routeByRepoPath: ReadonlyMap<string, string>;
  /** Does this repo-relative path exist on disk? */
  repoFileExists: (repoPath: string) => boolean;
  /** Base for GitHub blob links. Overridable for tests. */
  githubBlobBase?: string;
}

export interface RewriteResult {
  body: string;
  warnings: string[];
}

/** `scheme:` prefix per RFC 3986 — covers http, mailto, sediment, data, … */
const HAS_SCHEME = /^[A-Za-z][A-Za-z0-9+.-]*:/;

/**
 * Matches the target of an inline markdown link or image: the `](` opener plus
 * the destination token. The optional title (`"…"`) is left alone.
 */
const LINK_TARGET = /\]\(\s*(<[^<>]*>|[^()\s]+)/g;

export function rewriteLinks(options: RewriteOptions): RewriteResult {
  const { repoPath, body, bodyStartLine, routeByRepoPath, repoFileExists } = options;
  const githubBlobBase = options.githubBlobBase ?? GITHUB_BLOB_BASE;

  const code = codeRegions(body);
  const warnings: string[] = [];

  let out = '';
  let cursor = 0;
  let match: RegExpExecArray | null;
  LINK_TARGET.lastIndex = 0;

  while ((match = LINK_TARGET.exec(body)) !== null) {
    if (isInsideRegion(code, match.index)) continue;

    const rawToken = match[1]!;
    const bracketed = rawToken.startsWith('<') && rawToken.endsWith('>');
    const target = bracketed ? rawToken.slice(1, -1) : rawToken;

    const replacement = resolveTarget({
      target,
      repoPath,
      routeByRepoPath,
      repoFileExists,
      githubBlobBase,
      line: () => lineAt(body, match!.index) + bodyStartLine - 1,
      warn: (message) => warnings.push(message),
    });
    if (replacement === null) continue;

    const tokenStart = match.index + match[0]!.length - rawToken.length;
    out += body.slice(cursor, tokenStart);
    out += bracketed ? `<${replacement}>` : replacement;
    cursor = tokenStart + rawToken.length;
  }

  out += body.slice(cursor);
  return { body: out, warnings };
}

interface ResolveArgs {
  target: string;
  repoPath: string;
  routeByRepoPath: ReadonlyMap<string, string>;
  repoFileExists: (repoPath: string) => boolean;
  githubBlobBase: string;
  line: () => number;
  warn: (message: string) => void;
}

/** Returns the replacement destination, or `null` to leave the link untouched. */
function resolveTarget(args: ResolveArgs): string | null {
  const { target, repoPath, routeByRepoPath, repoFileExists, githubBlobBase } = args;

  if (target === '' || target.startsWith('#')) return null; // bare anchor
  if (target.startsWith('/')) return null; // already site-absolute
  if (target.startsWith('//')) return null; // protocol-relative
  if (HAS_SCHEME.test(target)) return null; // http:, mailto:, sediment:, …

  const hashIndex = target.indexOf('#');
  const pathPart = hashIndex === -1 ? target : target.slice(0, hashIndex);
  const anchor = hashIndex === -1 ? '' : target.slice(hashIndex);
  if (pathPart === '') return null;

  const resolved = resolveRepoPath(repoPath, pathPart);
  if (resolved === null) {
    // Escapes the repository root — not something we can rewrite.
    if (isMarkdown(pathPart)) {
      throw new CorpusError(
        repoPath,
        `broken link \`${target}\` — resolves outside the repository`,
        args.line()
      );
    }
    args.warn(`${repoPath}:${args.line()}: link \`${target}\` escapes the repository — left as-is`);
    return null;
  }

  const route = routeByRepoPath.get(resolved);
  if (route !== undefined) return `${route}${anchor}`;

  if (repoFileExists(resolved)) {
    return `${githubBlobBase}${encodeRepoPath(resolved)}${anchor}`;
  }

  if (isMarkdown(pathPart)) {
    throw new CorpusError(
      repoPath,
      `broken link \`${target}\` — no corpus doc or repository file at \`${resolved}\``,
      args.line()
    );
  }

  args.warn(`${repoPath}:${args.line()}: unresolved relative link \`${target}\` — left as-is`);
  return null;
}

function isMarkdown(pathPart: string): boolean {
  return /\.mdx?$/i.test(pathPart);
}

/**
 * Resolve `relative` against the directory of `repoPath`, POSIX-style.
 * Returns `null` when the result escapes the repository root.
 */
export function resolveRepoPath(repoPath: string, relative: string): string | null {
  const decoded = safeDecode(relative);
  const base = repoPath.split('/').slice(0, -1);
  const segments = decoded.split('/');
  const stack = [...base];

  for (const segment of segments) {
    if (segment === '' || segment === '.') continue;
    if (segment === '..') {
      if (stack.length === 0) return null;
      stack.pop();
      continue;
    }
    stack.push(segment);
  }

  return stack.length === 0 ? null : stack.join('/');
}

function safeDecode(value: string): string {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
}

/** Re-encode path segments for a URL without mangling the separators. */
function encodeRepoPath(repoPath: string): string {
  return repoPath.split('/').map(encodeURIComponent).join('/');
}
