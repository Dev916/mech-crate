import { defineConfig } from 'astro/config';

// https://astro.build/config
//
// Static output: the site is a documentation site deployed to Cloudflare as a
// static bundle (see docs/superpowers/specs/2026-08-20-mechcrate-site-design.md).
//
// Port 4321 / host binding match what the mx astro recipe's dev override
// expects (docker/compose/site.dev.yml: PORT=4321, Traefik loadbalancer port
// 4321, source mounted at /app).
export default defineConfig({
  site: 'https://mechcrate.dev',

  server: {
    host: true,
    port: 4321,
  },
});
