/**
 * Secret lint for published corpus content.
 *
 * The site publishes by default, so the cheap insurance is a scan of everything
 * that actually ships (frontmatter values + rendered body, code fences included —
 * a leaked DSN in a code sample is still a leaked DSN). Any hit fails the build
 * with the offending file and line.
 *
 * Spec patterns: `postgres://(?!localhost)`, `sk-`, `AKIA`, bearer tokens.
 *
 * Two refinements, both deliberate, because the naive forms are unusable against
 * a 70-document prose corpus:
 *
 *  1. `sk-`/`AKIA`/`Bearer` are anchored at a word boundary and require a
 *     credential-length tail. Bare `sk-[A-Za-z0-9]` matches ordinary English —
 *     "ri{sk-f}ree", "ta{sk-l}ifecycle", "ri{sk-m}odel.json" — so it would fire
 *     on dozens of true negatives and get switched off, which is worse than no
 *     lint. Real OpenAI keys carry a 48+ char tail and AWS key ids are AKIA + 16.
 *
 *  2. `postgres://` is fatal unless the authority is a local host or is visibly
 *     redacted (`...`, `${VAR}`, `<placeholder>`, `***`). `postgres://localhost`
 *     is documentation; `postgres://user:pass@host.neon.tech` is a leak.
 */

import { lineAt } from './markdown.ts';

export interface SecretFinding {
  /** Stable rule id, e.g. `openai-api-key`. */
  rule: string;
  /** Human-readable description of what was matched. */
  description: string;
  /** 1-based line within the scanned text. */
  line: number;
  /** The matched text, redacted to a short prefix so the log never echoes it. */
  excerpt: string;
}

interface PatternRule {
  rule: string;
  description: string;
  pattern: RegExp;
}

const PATTERN_RULES: PatternRule[] = [
  {
    rule: 'openai-api-key',
    description: 'OpenAI-style API key (`sk-…`)',
    // Word-boundary anchored; 20+ char tail. Real keys are `sk-` + 48.
    pattern: /(?<![A-Za-z0-9_-])sk-[A-Za-z0-9_-]{20,}/g,
  },
  {
    rule: 'aws-access-key-id',
    description: 'AWS access key id (`AKIA…`)',
    pattern: /(?<![A-Za-z0-9])AKIA[0-9A-Z]{16}(?![0-9A-Z])/g,
  },
  {
    rule: 'bearer-token',
    description: 'Bearer token',
    pattern: /(?<![A-Za-z0-9])Bearer\s+[A-Za-z0-9._-]{20,}/g,
  },
];

/** Hosts that are always safe to publish in a connection string. */
const LOCAL_HOSTS = new Set([
  'localhost',
  '127.0.0.1',
  '0.0.0.0',
  '::1',
  '[::1]',
  'db',
  'postgres',
  'host.docker.internal',
]);

/** Markers that make an authority visibly a placeholder rather than a real target. */
const REDACTION_MARKERS = ['...', '…', '${', '<', '***', 'xxx', 'your-', 'your_'];

const POSTGRES_URL = /postgres:\/\/([^\s'"`)\]}>,;]*)/g;

/** Scan `text` for secrets. `startLine` offsets reported lines into the source file. */
export function findSecrets(text: string, startLine = 1): SecretFinding[] {
  const findings: SecretFinding[] = [];

  for (const { rule, description, pattern } of PATTERN_RULES) {
    pattern.lastIndex = 0;
    let match: RegExpExecArray | null;
    while ((match = pattern.exec(text)) !== null) {
      findings.push({
        rule,
        description,
        line: lineAt(text, match.index) + startLine - 1,
        excerpt: redact(match[0]!),
      });
    }
  }

  POSTGRES_URL.lastIndex = 0;
  let match: RegExpExecArray | null;
  while ((match = POSTGRES_URL.exec(text)) !== null) {
    if (isPublishableDsn(match[1] ?? '')) continue;
    findings.push({
      rule: 'postgres-dsn',
      description: 'non-local Postgres connection string',
      line: lineAt(text, match.index) + startLine - 1,
      excerpt: redact(match[0]!),
    });
  }

  return findings.sort((a, b) => a.line - b.line || a.rule.localeCompare(b.rule));
}

/**
 * True when a `postgres://` authority is safe to publish: a local host, or an
 * obviously redacted placeholder.
 */
function isPublishableDsn(rest: string): boolean {
  const authority = rest.split(/[/?#]/)[0] ?? '';
  if (authority === '') return true; // `postgres://` on its own — a bare scheme mention.

  const lower = authority.toLowerCase();
  if (REDACTION_MARKERS.some((marker) => lower.includes(marker))) return true;

  const hostPort = lower.includes('@') ? lower.slice(lower.lastIndexOf('@') + 1) : lower;
  const host = hostPort.startsWith('[')
    ? hostPort.slice(0, hostPort.indexOf(']') + 1)
    : (hostPort.split(':')[0] ?? '');

  return LOCAL_HOSTS.has(host);
}

/** Keep enough of a match to locate it, never enough to use it. */
function redact(match: string): string {
  const head = match.slice(0, 12);
  return match.length > 12 ? `${head}…(${match.length} chars)` : head;
}

/** Format findings into the message used for the fatal build error. */
export function formatFindings(repoPath: string, findings: SecretFinding[]): string {
  return findings
    .map((f) => `${repoPath}:${f.line}: ${f.description} — ${f.excerpt} [${f.rule}]`)
    .join('\n');
}
