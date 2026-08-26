/**
 * Frontmatter parsing and normalisation for corpus docs.
 *
 * Policy (spec → "Error handling & policies"):
 *   - unparseable frontmatter  → build FAILURE with file/line
 *   - missing `title`/`category` → filename fallback + `uncategorized`, build WARNING
 *   - `publish: false`         → doc is skipped entirely
 */

import { parse as parseYaml, YAMLParseError } from 'yaml';
import { CorpusError, type CorpusFrontmatter } from './types.ts';
import { sanitizeSlug, titleFromFilename, UNCATEGORIZED } from './slug.ts';

export interface SplitDocument {
  /** Raw YAML text between the `---` fences, or null when there is no frontmatter. */
  yaml: string | null;
  /** Markdown body with the frontmatter block removed. */
  body: string;
  /** 1-based line in the original file where `body` starts. */
  bodyStartLine: number;
}

const FRONTMATTER_RE = /^﻿?---\r?\n([\s\S]*?)\r?\n?---[ \t]*(?:\r?\n|$)/;

/** Split a raw document into its YAML frontmatter and markdown body. */
export function splitFrontmatter(raw: string): SplitDocument {
  const match = FRONTMATTER_RE.exec(raw);
  if (match === null) return { yaml: null, body: raw, bodyStartLine: 1 };

  const consumed = match[0]!;
  const body = raw.slice(consumed.length);
  // Number of newlines consumed by the frontmatter block == lines before body.
  const bodyStartLine = (consumed.match(/\n/g)?.length ?? 0) + 1;
  return { yaml: match[1] ?? '', body, bodyStartLine };
}

/**
 * Parse a document's frontmatter.
 * Throws `CorpusError` (with the YAML error's line, offset into the real file)
 * when the block is present but not valid YAML, or is not a mapping.
 */
export function parseFrontmatter(
  repoPath: string,
  raw: string
): { data: CorpusFrontmatter; body: string; bodyStartLine: number } {
  const { yaml, body, bodyStartLine } = splitFrontmatter(raw);
  if (yaml === null) return { data: {}, body, bodyStartLine };

  let parsed: unknown;
  try {
    parsed = parseYaml(yaml);
  } catch (error) {
    // `yaml` reports 1-based lines within the block; the block starts on file line 2.
    const line = error instanceof YAMLParseError ? (error.linePos?.[0]?.line ?? 1) + 1 : 1;
    const detail = error instanceof Error ? error.message.split('\n')[0] : String(error);
    throw new CorpusError(repoPath, `unparseable frontmatter — ${detail}`, line);
  }

  if (parsed === null || parsed === undefined) return { data: {}, body, bodyStartLine };
  if (typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new CorpusError(repoPath, 'unparseable frontmatter — expected a YAML mapping', 2);
  }

  return { data: parsed as CorpusFrontmatter, body, bodyStartLine };
}

/** `publish: false` (and only an explicit boolean false) holds a doc back. */
export function isHeldBack(data: CorpusFrontmatter): boolean {
  return data.publish === false;
}

export interface NormalizedMetadata {
  title: string;
  category: string;
  description?: string;
  complexity?: string;
  languages?: string[];
  useCases?: string[];
  provenance?: string;
  researched?: string;
  sources?: string[];
  inferredTitle: boolean;
  inferredCategory: boolean;
  warnings: string[];
}

/**
 * Map corpus frontmatter onto page data, filling in fallbacks and collecting
 * warnings for anything that had to be synthesised.
 */
export function normalizeMetadata(
  repoPath: string,
  data: CorpusFrontmatter,
  options: { defaultCategory?: string } = {}
): NormalizedMetadata {
  const warnings: string[] = [];

  const rawTitle = asText(data.title);
  const inferredTitle = rawTitle === undefined;
  const title = rawTitle ?? titleFromFilename(repoPath);
  if (inferredTitle) {
    warnings.push(`${repoPath}: no frontmatter \`title\` — falling back to "${title}"`);
  }

  const rawCategory = asText(data.category);
  const inferredCategory = rawCategory === undefined;
  const categorySource = rawCategory ?? options.defaultCategory ?? UNCATEGORIZED;
  const category = sanitizeSlug(categorySource) || UNCATEGORIZED;
  if (inferredCategory) {
    warnings.push(`${repoPath}: no frontmatter \`category\` — filed under "${category}"`);
  }

  return {
    title,
    category,
    description: asText(data.summary),
    complexity: asText(data.complexity),
    languages: asTextList(data.languages),
    useCases: asTextList(data.use_cases),
    provenance: asText(data.provenance),
    researched: asText(data.researched),
    sources: asTextList(data.sources),
    inferredTitle,
    inferredCategory,
    warnings,
  };
}

/** Coerce a scalar frontmatter value to a trimmed non-empty string. */
function asText(value: unknown): string | undefined {
  if (value === null || value === undefined) return undefined;
  if (value instanceof Date) return value.toISOString().slice(0, 10);
  if (typeof value === 'string') {
    const trimmed = value.trim();
    return trimmed === '' ? undefined : trimmed;
  }
  if (typeof value === 'number' || typeof value === 'boolean') return String(value);
  return undefined;
}

/** Coerce a frontmatter value to a list of strings (accepts a bare scalar). */
function asTextList(value: unknown): string[] | undefined {
  if (value === null || value === undefined) return undefined;
  const items = Array.isArray(value) ? value : [value];
  const texts = items.map(asText).filter((v): v is string => v !== undefined);
  return texts.length === 0 ? undefined : texts;
}
