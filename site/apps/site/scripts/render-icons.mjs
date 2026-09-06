#!/usr/bin/env node
/**
 * Renders the site's raster icon set into `public/`.
 *
 * A one-shot generator, like `render-diagrams.mjs`: the outputs are committed
 * and served as static assets, so the build never runs this. Re-run it by hand
 * when the brand assets change:
 *
 *     node scripts/render-icons.mjs
 *
 * Two sources, on purpose:
 *
 *   - **`public/favicon.svg`** feeds `favicon.ico` (16/32/48). That file is
 *     already the crate mark rather than the full mascot, for the reason its own
 *     comment gives — "at favicon size the raccoon is mud" — so the .ico is the
 *     same mark rasterised, not a second, worse drawing of it. Transparent, so
 *     it sits on light and dark browser chrome alike.
 *   - **`src/assets/mechcrate-logo.png`** feeds `apple-touch-icon.png` (180×180)
 *     and the two manifest icons (192, 512). Those are rendered at sizes where
 *     the raccoon reads, and Apple composites its home-screen icon on an opaque
 *     tile, so they get the site's dark ground and ~10% padding rather than
 *     transparency.
 *
 * `favicon.ico` is written as a PNG-in-ICO container — the format every browser
 * since IE11 reads, and the only one that keeps 48×48 alpha in a few kilobytes.
 * The 22-byte header/directory maths is small enough to do here rather than take
 * a dependency for.
 *
 * sharp comes in with Astro's image pipeline (`astro`'s optional dependency,
 * used by its default image service), which is why it needs no entry of its own
 * in package.json. It is a build-time tool here and never reaches the browser.
 */

import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import sharp from 'sharp';

const appRoot = join(dirname(fileURLToPath(import.meta.url)), '..');
const publicDir = join(appRoot, 'public');

/** `--sl-color-black`, hsl(224 10% 10%) — the site's dark ground. */
const BACKGROUND = { r: 23, g: 24, b: 28, alpha: 1 };

/** Sizes packed into favicon.ico. 48 is what Windows and some readers pick. */
const ICO_SIZES = [16, 32, 48];

/** Share of an opaque tile the mark occupies; the rest is padding. */
const TILE_INSET = 0.8;

/** Rasterise the SVG mark at `size`, transparent. */
async function markPng(svg, size) {
  return sharp(svg, { density: 384 }).resize(size, size, { fit: 'contain' }).png().toBuffer();
}

/** The logo centred on an opaque brand-dark tile of `size`, with padding. */
async function tilePng(logo, size) {
  const inner = Math.round(size * TILE_INSET);
  const mark = await sharp(logo)
    .resize(inner, inner, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .toBuffer();

  return sharp({
    create: { width: size, height: size, channels: 4, background: BACKGROUND },
  })
    .composite([{ input: mark, gravity: 'centre' }])
    .png()
    .toBuffer();
}

/**
 * Pack PNG buffers into an ICO container.
 *
 * Layout: a 6-byte header (`0 0`, type 1 = icon, image count), then one 16-byte
 * directory entry per image, then the PNG payloads. A dimension of 256 or more
 * is encoded as 0; ours are all smaller.
 */
function ico(images) {
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0); // reserved
  header.writeUInt16LE(1, 2); // type: icon
  header.writeUInt16LE(images.length, 4);

  const directory = Buffer.alloc(16 * images.length);
  let offset = header.length + directory.length;

  images.forEach(({ size, data }, index) => {
    const at = index * 16;
    directory.writeUInt8(size >= 256 ? 0 : size, at); // width
    directory.writeUInt8(size >= 256 ? 0 : size, at + 1); // height
    directory.writeUInt8(0, at + 2); // palette entries (0 = truecolour)
    directory.writeUInt8(0, at + 3); // reserved
    directory.writeUInt16LE(1, at + 4); // colour planes
    directory.writeUInt16LE(32, at + 6); // bits per pixel
    directory.writeUInt32LE(data.length, at + 8);
    directory.writeUInt32LE(offset, at + 12);
    offset += data.length;
  });

  return Buffer.concat([header, directory, ...images.map((image) => image.data)]);
}

async function main() {
  await mkdir(publicDir, { recursive: true });

  const svg = await readFile(join(publicDir, 'favicon.svg'));
  const logo = await readFile(join(appRoot, 'src', 'assets', 'mechcrate-logo.png'));

  const icoImages = await Promise.all(
    ICO_SIZES.map(async (size) => ({ size, data: await markPng(svg, size) }))
  );
  await writeFile(join(publicDir, 'favicon.ico'), ico(icoImages));

  for (const [name, size] of [
    ['apple-touch-icon.png', 180],
    ['icon-192.png', 192],
    ['icon-512.png', 512],
  ]) {
    await writeFile(join(publicDir, name), await tilePng(logo, size));
  }

  const written = ['favicon.ico', 'apple-touch-icon.png', 'icon-192.png', 'icon-512.png'];
  console.log(`icons: wrote ${written.join(', ')} to public/`);
}

await main();
