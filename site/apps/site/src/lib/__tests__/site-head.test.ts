/**
 * Unit tests for the shared `<head>` fragments (`src/lib/site-head.ts`).
 *
 * The point of the module is that the docs head and the landing head emit the
 * same bytes, so the assertions are about the exact markup rather than about
 * "contains an icon somewhere". The beacon's empty-token path is asserted hardest:
 * it is the documented way to ship the site without analytics, and a build that
 * broke on an empty constant would make that switch unusable.
 */

import { readFileSync, statSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import { ICON_TAGS, SITEMAP_PATH, SITEMAP_TAG, cfBeaconTag, commonHeadHtml } from '../site-head.ts';
import { CF_BEACON_TOKEN, THEME_COLOR } from '../../site-meta.ts';

describe('cfBeaconTag', () => {
  it('emits the snippet Cloudflare documents, with the token inlined', () => {
    expect(cfBeaconTag('abc123')).toBe(
      '<script type="module" src="https://static.cloudflareinsights.com/beacon.min.js" ' +
        'data-cf-beacon=\'{"token": "abc123"}\'></script>'
    );
  });

  it('emits nothing for an empty token', () => {
    expect(cfBeaconTag('')).toBe('');
  });

  it('emits nothing for a whitespace-only token', () => {
    expect(cfBeaconTag('   \n')).toBe('');
  });

  it('trims a token that arrived with stray whitespace', () => {
    expect(cfBeaconTag(' abc123 ')).toContain('"token": "abc123"');
  });
});

describe('the configured token', () => {
  it('is either empty (analytics off) or a well-formed site token', () => {
    // 32 lowercase hex characters. A wrong-shaped token fails silently in the
    // browser — the beacon posts and nothing is recorded for a month — so the
    // shape is asserted. Empty is a supported configuration, not a defect, and
    // must not turn the suite red for whoever chooses it.
    expect(CF_BEACON_TOKEN === '' || /^[0-9a-f]{32}$/.test(CF_BEACON_TOKEN)).toBe(true);
  });

  it('puts a beacon in the shared head block exactly when it is set', () => {
    const html = commonHeadHtml(CF_BEACON_TOKEN);
    expect(html.includes('static.cloudflareinsights.com/beacon.min.js')).toBe(
      CF_BEACON_TOKEN !== ''
    );
  });
});

describe('commonHeadHtml', () => {
  it('emits the icon set, the manifest and theme-color, in that order', () => {
    const html = commonHeadHtml('');
    expect(html).toBe(ICON_TAGS.join('\n'));
    expect(html).toContain('<link rel="icon" href="/favicon.ico" sizes="16x16 32x32 48x48" />');
    expect(html).toContain(
      '<link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png" />'
    );
    expect(html).toContain('<link rel="manifest" href="/site.webmanifest" />');
    expect(html).toContain(`<meta name="theme-color" content="${THEME_COLOR}" />`);
  });

  it('omits the sitemap link unless asked — Starlight emits its own', () => {
    expect(commonHeadHtml('')).not.toContain('rel="sitemap"');
    expect(commonHeadHtml('', { sitemap: true })).toContain(SITEMAP_TAG);
  });

  it('puts the sitemap link first and the beacon last', () => {
    const html = commonHeadHtml('token', { sitemap: true });
    const lines = html.split('\n');
    expect(lines[0]).toBe(SITEMAP_TAG);
    expect(lines.at(-1)).toContain('cloudflareinsights');
  });

  it('never emits an icon link before the site\'s SVG favicon can win', () => {
    // The .ico is a fallback for browsers that cannot use favicon.svg. Both head
    // owners emit the SVG link ahead of this block; this asserts the block does
    // not smuggle a second SVG link of its own into the race.
    expect(commonHeadHtml('')).not.toContain('favicon.svg');
  });

  it('stays valid with analytics switched off', () => {
    const html = commonHeadHtml('');
    expect(html).not.toContain('<script');
    expect(html.split('\n')).toHaveLength(ICON_TAGS.length);
  });

  it('points at the sitemap index @astrojs/sitemap actually writes', () => {
    expect(SITEMAP_PATH).toBe('/sitemap-index.xml');
  });
});

describe('the static files the head points at', () => {
  const publicDir = join(dirname(fileURLToPath(import.meta.url)), '..', '..', '..', 'public');

  it('ships every icon the block advertises', () => {
    // A `<link rel="manifest">` or an apple-touch-icon with no file behind it is
    // a 404 nobody sees until a phone tries to save the page to a home screen.
    for (const file of [
      'favicon.svg',
      'favicon.ico',
      'apple-touch-icon.png',
      'site.webmanifest',
      'icon-192.png',
      'icon-512.png',
    ]) {
      expect(statSync(join(publicDir, file)).size, file).toBeGreaterThan(0);
    }
  });

  it('packs favicon.ico with 16, 32 and 48 pixel images', () => {
    // ICO header: two reserved bytes, type (1 = icon), image count; then one
    // 16-byte directory entry each, opening with width and height.
    const ico = readFileSync(join(publicDir, 'favicon.ico'));
    expect(ico.readUInt16LE(2)).toBe(1);
    const count = ico.readUInt16LE(4);
    const sizes = Array.from({ length: count }, (_, i) => ico.readUInt8(6 + i * 16));
    expect(sizes).toEqual([16, 32, 48]);
  });

  it('agrees with the manifest about the theme colour', () => {
    // public/site.webmanifest is static JSON served as-is — it cannot import the
    // constant, so this is the one place the two are checked against each other.
    const manifest = JSON.parse(readFileSync(join(publicDir, 'site.webmanifest'), 'utf8'));
    expect(THEME_COLOR).toMatch(/^#[0-9a-f]{6}$/);
    expect(manifest.theme_color).toBe(THEME_COLOR);
    expect(manifest.background_color).toBe(THEME_COLOR);
  });

  it('lists manifest icons that exist at the sizes they claim', () => {
    const manifest = JSON.parse(readFileSync(join(publicDir, 'site.webmanifest'), 'utf8'));
    expect(manifest.icons.map((icon: { sizes: string }) => icon.sizes)).toEqual([
      '192x192',
      '512x512',
    ]);
    for (const icon of manifest.icons as { src: string }[]) {
      expect(statSync(join(publicDir, icon.src.replace(/^\//, ''))).size).toBeGreaterThan(0);
    }
  });
});
