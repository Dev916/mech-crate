import { readFileSync, readdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import {
  findMermaidError,
  hashSource,
  rootSvgId,
} from '../../../../scripts/lib/diagram-svg.mjs';
import { DIAGRAMS, DIAGRAM_NAMES, isDiagramName } from '../registry.ts';

/**
 * The diagram SVGs are pre-rendered by `npm run diagrams` and committed — see
 * the rationale at the top of `scripts/render-diagrams.mjs`. Committed build
 * output can drift from its source, so these tests are the thing that makes the
 * arrangement safe: edit a `.mmd` without re-rendering and `npm test` fails.
 *
 * Nothing here launches a browser. It reads the committed artefacts only, so it
 * runs identically in CI, where Chromium is not installed.
 */

const here = dirname(fileURLToPath(import.meta.url));
const DIAGRAM_DIR = join(here, '..');
const SOURCE_DIR = join(DIAGRAM_DIR, 'sources');
const RENDERED_DIR = join(DIAGRAM_DIR, 'rendered');

const VARIANTS = ['light', 'dark'] as const;

const manifest: Record<string, string> = JSON.parse(
  readFileSync(join(RENDERED_DIR, 'manifest.json'), 'utf8'),
);

const sourceNames = readdirSync(SOURCE_DIR)
  .filter((f) => f.endsWith('.mmd'))
  .map((f) => f.replace(/\.mmd$/, ''))
  .sort();

const readSource = (name: string) =>
  readFileSync(join(SOURCE_DIR, `${name}.mmd`), 'utf8');

const readSvg = (name: string, variant: string) =>
  readFileSync(join(RENDERED_DIR, `${name}.${variant}.svg`), 'utf8');

describe('diagram set', () => {
  it('registers exactly the five diagrams the spec calls for', () => {
    expect([...DIAGRAM_NAMES].sort()).toEqual([
      'ai-loop',
      'compose-layering',
      'ecosystem-topology',
      'folder-contract',
      'recipe-install-flow',
    ]);
  });

  it('has one .mmd source per registered diagram, and no orphans', () => {
    expect(sourceNames).toEqual([...DIAGRAM_NAMES].sort());
  });

  it('gives every diagram a title and a caption', () => {
    for (const name of DIAGRAM_NAMES) {
      expect(DIAGRAMS[name].title, name).toMatch(/\S/);
      expect(DIAGRAMS[name].caption.length, name).toBeGreaterThan(40);
    }
  });

  it('narrows unknown names', () => {
    expect(isDiagramName('ai-loop')).toBe(true);
    expect(isDiagramName('does-not-exist')).toBe(false);
  });
});

describe('committed renderings', () => {
  it.each(DIAGRAM_NAMES)('%s is in sync with its source', (name) => {
    // The drift guard. `npm run diagrams` rewrites both; if only the .mmd moved,
    // the hashes disagree and this is the failure that says so.
    expect(manifest[name], `manifest missing ${name}`).toBe(
      hashSource(readSource(name)),
    );
  });

  it('manifest has no entries without a source', () => {
    expect(Object.keys(manifest).sort()).toEqual(sourceNames);
  });

  it.each(
    DIAGRAM_NAMES.flatMap((name) => VARIANTS.map((v) => [name, v] as const)),
  )('%s (%s) is a real, responsive SVG', (name, variant) => {
    const svg = readSvg(name, variant);

    expect(svg.startsWith('<svg ')).toBe(true);
    expect(svg.trimEnd().endsWith('</svg>')).toBe(true);
    // viewBox + width="100%" and no pixel height is what lets it scale.
    expect(svg).toMatch(/\sviewBox="[\d.\s-]+"/);
    expect(svg).toMatch(/^<svg width="100%"/);
    expect(svg).not.toMatch(/^<svg[^>]*\sheight="/);
    // Mermaid's accTitle/accDescr survived, so the figure is described to AT.
    expect(svg).toMatch(/<title id="chart-title-[^"]+">/);
    expect(svg).toMatch(/<desc id="chart-desc-[^"]+">/);
  });

  it.each(
    DIAGRAM_NAMES.flatMap((name) => VARIANTS.map((v) => [name, v] as const)),
  )('%s (%s) is a diagram, not a mermaid error graphic', (name, variant) => {
    expect(findMermaidError(readSvg(name, variant))).toBeNull();
  });
});

describe('inlining safety', () => {
  it('gives every rendering a unique root id', () => {
    // Both variants of all five land in one document on the reference page.
    const ids = DIAGRAM_NAMES.flatMap((name) =>
      VARIANTS.map((variant) => rootSvgId(readSvg(name, variant))),
    );
    expect(ids.every((id) => typeof id === 'string' && id.length > 0)).toBe(true);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it('namespaces edge ids so the light/dark pair cannot collide', () => {
    for (const name of DIAGRAM_NAMES) {
      for (const variant of VARIANTS) {
        const svg = readSvg(name, variant);
        const id = rootSvgId(svg);
        const edgeIds = [...svg.matchAll(/\sid="(L_[^"]*)"/g)];
        // Any surviving bare `L_…` id would repeat across the pair.
        expect(edgeIds, `${name}.${variant} has un-prefixed edge ids`).toEqual(
          [],
        );
        expect(svg.includes(`id="${id}_L_`) || !svg.includes('_L_')).toBe(true);
      }
    }
  });
});
