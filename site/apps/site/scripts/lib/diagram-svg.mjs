/**
 * Pure helpers shared by the diagram renderer and its test.
 *
 * Deliberately dependency-free: `scripts/render-diagrams.mjs` needs Playwright
 * and a Chromium build, and the test must not. Keeping the hashing and the
 * post-processing here lets `npm test` verify the *committed* SVGs on a machine
 * that could not render one.
 */
import { createHash } from 'node:crypto';

/** Content hash of a `.mmd` source, recorded in `rendered/manifest.json`. */
export function hashSource(source) {
  return createHash('sha256').update(source, 'utf8').digest('hex').slice(0, 16);
}

/**
 * True when Mermaid gave up and drew its "bomb" graphic instead of the diagram.
 *
 * Naive substring matching does not work: every Mermaid SVG embeds a boilerplate
 * `<style>` block that *declares* `.error-icon` and `.error-text` rules whether
 * or not they are used. So the stylesheet is stripped first, and what is left is
 * checked for the markers only a real failure produces — the `error`
 * roledescription on the root element, the rendered error copy, and the version
 * stamp Mermaid prints beneath it.
 *
 * @returns the matched marker, or `null` when the SVG is a real diagram.
 */
export function findMermaidError(svg) {
  const markup = svg.replace(/<style[\s\S]*?<\/style>/g, '');
  const patterns = [
    /aria-roledescription="error"/i,
    /Syntax error/i,
    /mermaid version\s/i,
    /class="error-(icon|text)"/i,
  ];
  for (const pattern of patterns) {
    const match = markup.match(pattern);
    if (match) return match[0];
  }
  return null;
}

/**
 * Path coordinates come out of d3 at full float precision — `1370.5008544921875`
 * where `1370.5` is indistinguishable at any zoom. On edges that route around a
 * cluster Mermaid emits hundreds of points, so this is not cosmetic: it took the
 * ecosystem diagram from 155 kB to under half that.
 */
export function trimPrecision(pathData) {
  return pathData.replace(/-?\d+\.\d+/g, (n) =>
    String(Math.round(Number(n) * 100) / 100),
  );
}

/** The root `<svg id>` Mermaid assigned, used to namespace everything else. */
export function rootSvgId(svg) {
  return svg.match(/^<svg[^>]*?\sid="([^"]+)"/)?.[1] ?? null;
}

/**
 * Post-process one Mermaid SVG so it is safe to inline, twice, into a page that
 * already has four other diagrams on it:
 *
 * - drop the fixed pixel `width`/`height`/`max-width` and lean on the `viewBox`,
 *   so a diagram scales down on a phone instead of scrolling the article sideways;
 * - namespace the `L_from_to_n` edge ids, the one id family Mermaid leaves
 *   un-prefixed, so the light and dark copies cannot produce duplicate DOM ids;
 * - round path coordinates.
 */
export function normalize(svg) {
  const id = rootSvgId(svg) ?? 'mmd';

  return svg
    .replace(/^(<svg[^>]*?)\s+width="[^"]*"/, '$1')
    .replace(/^(<svg[^>]*?)\s+height="[^"]*"/, '$1')
    .replace(/^(<svg[^>]*?)\s+style="[^"]*"/, '$1')
    .replace(/^<svg /, '<svg width="100%" ')
    .replace(/\bid="(L_[^"]*)"/g, `id="${id}_$1"`)
    .replace(/\sd="([^"]+)"/g, (_, d) => ` d="${trimPrecision(d)}"`);
}
