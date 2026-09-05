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

  /*
   * The root-id test above proves the ten `<svg>` elements do not collide with
   * each other. It says nothing about the ~380 ids *inside* them — the markers,
   * gradients and filters that `url(#…)` points at. Those all have to be unique
   * across the whole document too, because /docs/framework/diagrams/ inlines
   * both variants of all five diagrams into one page: a repeated marker id
   * makes an arrowhead resolve to whichever twin the parser saw first, which
   * may well be the `display: none` one.
   */
  it('keeps every id unique across both variants of all five diagrams', () => {
    const seen = new Map<string, string>();
    const collisions: string[] = [];
    for (const name of DIAGRAM_NAMES) {
      for (const variant of VARIANTS) {
        const where = `${name}.${variant}`;
        const svg = readSvg(name, variant);
        for (const [, id] of svg.matchAll(/\sid="([^"]+)"/g)) {
          const prior = seen.get(id);
          if (prior && prior !== where) collisions.push(`${id} (${prior} + ${where})`);
          else seen.set(id, where);
        }
      }
    }
    expect(collisions, 'ids repeat across the inlined set').toEqual([]);
    expect(seen.size).toBeGreaterThan(100);
  });

  it('resolves every internal reference within its own file', () => {
    for (const name of DIAGRAM_NAMES) {
      for (const variant of VARIANTS) {
        const svg = readSvg(name, variant);
        const ids = new Set([...svg.matchAll(/\sid="([^"]+)"/g)].map((m) => m[1]));
        const refs = new Set<string>();
        for (const [, r] of svg.matchAll(/url\(#([^)]+)\)/g)) refs.add(r);
        for (const [, r] of svg.matchAll(/(?:xlink:)?href="#([^"]+)"/g)) refs.add(r);
        for (const [, r] of svg.matchAll(/aria-(?:labelledby|describedby)="([^"]+)"/g)) {
          for (const one of r.split(/\s+/)) refs.add(one);
        }
        // A dangling url(#…) is how arrowheads and gradients silently vanish.
        const dangling = [...refs].filter((r) => !ids.has(r));
        expect(dangling, `${name}.${variant} has dangling references`).toEqual([]);
        expect(refs.size).toBeGreaterThan(0);
      }
    }
  });

  it('scopes every rule in the embedded stylesheets to its own root id', () => {
    // An SVG inlined into HTML contributes its <style> to the *document*. One
    // unscoped selector in the dark copy would repaint the light one, and the
    // last diagram on the page would win — a bug that changes with page order.
    for (const name of DIAGRAM_NAMES) {
      for (const variant of VARIANTS) {
        const svg = readSvg(name, variant);
        const block = svg.match(/<style>([\s\S]*?)<\/style>/)?.[1] ?? '';
        expect(block, `${name}.${variant} has no stylesheet`).toMatch(/\S/);
        const withoutAtRules = block.replace(
          /@keyframes[^{]*\{(?:[^{}]*\{[^{}]*\})*[^{}]*\}/g,
          '',
        );
        const unscoped: string[] = [];
        for (const rule of withoutAtRules.split('}')) {
          const brace = rule.indexOf('{');
          if (brace < 0) continue;
          for (const selector of rule.slice(0, brace).split(',')) {
            const trimmed = selector.trim();
            if (trimmed && !trimmed.startsWith('#mmd-')) unscoped.push(trimmed);
          }
        }
        expect(unscoped, `${name}.${variant} leaks selectors`).toEqual([]);
      }
    }
  });
});

describe('scroll affordance', () => {
  /*
   * Regression guard for the reported "diagrams do not always render correctly".
   *
   * Nothing was wrong with the SVGs. The `clamp()` in this component lets a wide
   * diagram outgrow its column and scroll inside `.diagram__frame` instead of
   * shrinking to an illegible size — but macOS and iOS overlay scrollbars are
   * invisible at rest, so a third of the ecosystem topology (63% at 390px) was
   * simply cut off at the frame's edge with nothing to say it could be scrolled.
   *
   * The affordance is CSS-only, so this asserts on the component source. It is a
   * blunt test, but it is the thing that fails if the scroll shadow is dropped.
   */
  const component = readFileSync(join(DIAGRAM_DIR, 'Diagram.astro'), 'utf8');

  it('keeps the frame scrollable', () => {
    expect(component).toMatch(/\.diagram__frame\s*\{[\s\S]*?overflow-x:\s*auto/);
  });

  it('marks the scrollable edge so a clipped diagram does not read as broken', () => {
    // The `local` layers are the covers that ride with the content; without
    // them the shadow is static decoration and signals nothing.
    expect(component).toMatch(/background-attachment:[^;]*local[^;]*;/);
    expect(component.match(/linear-gradient/g)?.length ?? 0).toBeGreaterThanOrEqual(4);
  });

  it('still lets a wide diagram stay legible rather than squeezing it', () => {
    // The clamp is the reason the frame scrolls at all; if it is ever replaced
    // by a plain `max-width: 100%` the affordance above becomes dead code.
    expect(component).toMatch(/width:\s*clamp\(/);
    expect(component).toMatch(/max-width:\s*none/);
  });
});
