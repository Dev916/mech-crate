#!/usr/bin/env node
/**
 * Renders every `src/components/diagrams/sources/*.mmd` to a static SVG pair
 * (light + dark) under `src/components/diagrams/rendered/`.
 *
 * ---------------------------------------------------------------------------
 * WHY THIS SHAPE — the rendering choice, documented (Task 5 asked for it)
 * ---------------------------------------------------------------------------
 * The three realistic options for Mermaid on a Starlight site are:
 *
 *   1. Client-side `mermaid.js`   — ships ~1MB of JS to every reader, diagrams
 *                                    pop in after hydration, and `llms-full.txt`
 *                                    (and any scraper) sees a code fence rather
 *                                    than a picture. Rejected.
 *   2. `rehype-mermaid` at build  — renders to static SVG, but it is a *remark/
 *                                    rehype* plugin, so it only fires inside
 *                                    markdown/MDX. The landing page is a hand
 *                                    written `.astro` page and needs the AI-loop
 *                                    diagram too. It would also drag a Chromium
 *                                    launch into every `astro build`, including
 *                                    CI and Cloudflare. Rejected.
 *   3. Pre-render here, inline there  ← chosen.
 *
 * So: this script is the only thing that needs a browser, it runs on demand
 * (`npm run diagrams`), and its output is committed. `astro build` then does
 * nothing but inline a static SVG string, which means the diagrams work
 * identically in `.astro` pages and in `.mdx`, ship zero client JS, and cost the
 * build nothing. `src/components/diagrams/__tests__/diagrams.test.ts` hashes the
 * `.mmd` sources against `rendered/manifest.json`, so a source edited without a
 * re-render fails `npm test` instead of silently drifting.
 *
 * Usage:
 *   npm run diagrams          # re-render all five
 *   npm run diagrams -- --check   # fail if anything is stale (no writes)
 *
 * Requires the `playwright` chromium build:  npx playwright install chromium
 */
import { readFile, readdir, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { createMermaidRenderer } from 'mermaid-isomorphic';

import { findMermaidError, hashSource, normalize } from './lib/diagram-svg.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const DIAGRAM_DIR = join(here, '..', 'src', 'components', 'diagrams');
const SOURCE_DIR = join(DIAGRAM_DIR, 'sources');
const OUT_DIR = join(DIAGRAM_DIR, 'rendered');
const MANIFEST = join(OUT_DIR, 'manifest.json');

/**
 * Both variants are rendered and both are inlined into the page; CSS shows the
 * one matching Starlight's `data-theme`. Each variant gets its own `prefix` so
 * the two SVGs in a single document can never collide on an element id (Mermaid
 * namespaces its arrow markers with the root `<svg id>`, and `url(#…)` refs
 * resolve against the first match in the document).
 */
const VARIANTS = [
  {
    name: 'light',
    prefix: 'mmd-light',
    mermaidConfig: { theme: 'base', themeVariables: lightTheme() },
  },
  {
    name: 'dark',
    prefix: 'mmd-dark',
    mermaidConfig: { theme: 'base', themeVariables: darkTheme() },
  },
];

/**
 * Node fills come from each diagram's own `classDef` lines (they are part of the
 * authored content and identical in both variants). What the theme controls is
 * the *chrome*: edges, arrowheads, edge labels, subgraph frames and any node
 * that opted out of a class. Those are the values that have to flip.
 */
function lightTheme() {
  return {
    background: '#ffffff',
    primaryColor: '#f8fafc',
    primaryTextColor: '#0f172a',
    primaryBorderColor: '#64748b',
    lineColor: '#475569',
    textColor: '#0f172a',
    fontFamily:
      'ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, sans-serif',
    fontSize: '15px',
    clusterBkg: '#f1f5f9',
    clusterBorder: '#94a3b8',
    edgeLabelBackground: '#ffffff',
    tertiaryColor: '#f1f5f9',
  };
}

function darkTheme() {
  return {
    background: '#0d1117',
    primaryColor: '#1e293b',
    primaryTextColor: '#e2e8f0',
    primaryBorderColor: '#94a3b8',
    lineColor: '#94a3b8',
    textColor: '#e2e8f0',
    fontFamily:
      'ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, sans-serif',
    fontSize: '15px',
    clusterBkg: '#161b22',
    clusterBorder: '#475569',
    edgeLabelBackground: '#0d1117',
    tertiaryColor: '#1e293b',
  };
}

async function main() {
  const checkOnly = process.argv.includes('--check');

  const names = (await readdir(SOURCE_DIR))
    .filter((f) => f.endsWith('.mmd'))
    .map((f) => f.replace(/\.mmd$/, ''))
    .sort();

  if (names.length === 0) throw new Error(`no .mmd sources in ${SOURCE_DIR}`);

  const sources = await Promise.all(
    names.map((n) => readFile(join(SOURCE_DIR, `${n}.mmd`), 'utf8')),
  );
  const manifest = Object.fromEntries(
    names.map((n, i) => [n, hashSource(sources[i])]),
  );

  if (checkOnly) {
    const current = JSON.parse(await readFile(MANIFEST, 'utf8'));
    const stale = names.filter((n) => current[n] !== manifest[n]);
    const orphaned = Object.keys(current).filter((n) => !names.includes(n));
    if (stale.length || orphaned.length) {
      console.error(
        `diagrams out of date — run \`npm run diagrams\`\n  stale: ${stale.join(', ') || 'none'}\n  orphaned: ${orphaned.join(', ') || 'none'}`,
      );
      process.exit(1);
    }
    console.log(`diagrams up to date (${names.length})`);
    return;
  }

  const renderer = createMermaidRenderer();

  for (const variant of VARIANTS) {
    const results = await renderer(sources, {
      prefix: variant.prefix,
      mermaidConfig: variant.mermaidConfig,
    });

    for (const [i, result] of results.entries()) {
      if (result.status === 'rejected') {
        throw new Error(`${names[i]} (${variant.name}): ${result.reason}`);
      }
      const svg = normalize(result.value.svg);
      const failure = findMermaidError(svg);
      if (failure) {
        throw new Error(
          `${names[i]} (${variant.name}): mermaid rendered an error graphic (${failure})`,
        );
      }
      await writeFile(join(OUT_DIR, `${names[i]}.${variant.name}.svg`), svg);
      console.log(`rendered ${names[i]}.${variant.name}.svg`);
    }
  }

  await writeFile(MANIFEST, `${JSON.stringify(manifest, null, 2)}\n`);
  console.log(`wrote ${MANIFEST}`);
}

await main();
