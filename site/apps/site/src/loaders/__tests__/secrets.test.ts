import { describe, expect, it } from 'vitest';
import { findSecrets } from '../lib/secrets.ts';

const rules = (text: string) => findSecrets(text).map((f) => f.rule);

describe('secret lint — postgres DSNs', () => {
  it('allows a localhost DSN', () => {
    expect(findSecrets('postgres://localhost')).toEqual([]);
    expect(findSecrets('postgres://postgres@localhost:5432/mx_rag')).toEqual([]);
    expect(findSecrets('postgres://127.0.0.1:5432/db')).toEqual([]);
  });

  it('fails on a remote DSN with credentials', () => {
    const findings = findSecrets('DATABASE_URL=postgres://user:pass@host.neon.tech/mx_rag');
    expect(findings).toHaveLength(1);
    expect(findings[0]!.rule).toBe('postgres-dsn');
    expect(findings[0]!.line).toBe(1);
  });

  it('fails on a remote DSN with no credentials', () => {
    expect(rules('postgres://db.example.com:5432/prod')).toEqual(['postgres-dsn']);
  });

  it('allows visibly redacted authorities', () => {
    expect(findSecrets('postgres://...neon.tech/mx_rag')).toEqual([]);
    expect(findSecrets('postgres://${PGUSER}:${PGPASS}@${PGHOST}/db')).toEqual([]);
    expect(findSecrets('postgres://<user>:<pass>@<host>/db')).toEqual([]);
  });

  it('does not confuse postgresql:// sample URLs for postgres:// ones', () => {
    expect(findSecrets('postgresql://user:pass@db:5432/mydb')).toEqual([]);
  });

  it('reports the correct line, offset by the frontmatter', () => {
    const text = ['intro', '', 'postgres://user:pass@host.neon.tech/db'].join('\n');
    expect(findSecrets(text, 9)[0]!.line).toBe(11);
  });
});

describe('secret lint — API keys and tokens', () => {
  it('fails on an OpenAI-style key', () => {
    const key = `sk-${'A1b2C3d4E5f6G7h8'.repeat(3)}`;
    expect(rules(`OPENAI_API_KEY=${key}`)).toEqual(['openai-api-key']);
  });

  it('fails on an AWS access key id', () => {
    expect(rules('AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE')).toEqual(['aws-access-key-id']);
  });

  it('fails on a bearer token', () => {
    expect(rules('Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9')).toEqual([
      'bearer-token',
    ]);
  });

  it('never redacts into the log more than a short prefix', () => {
    const key = `sk-${'A1b2C3d4E5f6G7h8'.repeat(3)}`;
    const [finding] = findSecrets(key);
    expect(finding!.excerpt).not.toContain(key);
    expect(finding!.excerpt.length).toBeLessThan(30);
  });
});

describe('secret lint — true negatives from the real corpus', () => {
  // These strings all appear in published docs. A naive `sk-[A-Za-z0-9]` rule
  // matches every one of them, which is why the rule is boundary-anchored and
  // length-gated.
  it.each([
    'const assessor = new AIRiskAssessor("./models/risk-model.json");',
    'benefit: "Risk-free way to validate investor demand"',
    'frequency: "Risk-based (anytime, but typically every 3-5 years)"',
    'A2A 1.0 defines an 8-state task-lifecycle and task-hygiene discipline',
    'no published task-hygiene discipline (TTLs punted everywhere)',
  ])('does not fire on %s', (text) => {
    expect(findSecrets(text)).toEqual([]);
  });

  it('does not fire on the short teaching example in SHELL_SCRIPTING_GUIDE.md', () => {
    // Deliberately-bad-practice sample: `sk-` + 16 chars, well short of a real key.
    expect(findSecrets('DANGEROUS_API_KEY="sk-1234567890abcdef"  # NEVER DO THIS')).toEqual([]);
  });

  it('does not fire on prose containing the word Bearer', () => {
    expect(findSecrets('Send the Bearer header with your token.')).toEqual([]);
  });
});

describe('secret lint — reporting', () => {
  it('sorts findings by line', () => {
    const text = [
      'AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE',
      'ok',
      'postgres://user:pass@host.neon.tech/db',
    ].join('\n');
    expect(findSecrets(text).map((f) => f.line)).toEqual([1, 3]);
  });
});
