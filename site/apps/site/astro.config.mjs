import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

import { mechcrateSitemap } from './src/integrations/sitemap.ts';

// https://astro.build/config
//
// Static output: the site is a documentation site deployed to Cloudflare as a
// static bundle (see docs/superpowers/specs/2026-08-20-mechcrate-site-design.md).
//
// Routing split:
//   /              → custom landing page (src/pages/index.astro), NOT a Starlight doc
//   /docs/**       → Starlight, sourced from src/content/docs/docs/**
//   /api/health    → prerendered health endpoint for the compose healthcheck
//
// Starlight owns every route generated from the `docs` content collection, and
// the collection's base directory is `src/content/docs/`. Nesting the authored
// pages one level deeper (`src/content/docs/docs/…`) is what puts the docs
// under the `/docs/` prefix and leaves `/` free for the custom Astro page.
// Sidebar `autogenerate.directory` values are therefore collection-relative and
// carry the same `docs/` prefix.
//
// Port 4321 / host binding match what the mx astro recipe's dev override
// expects (docker/compose/site.dev.yml: PORT=4321, Traefik loadbalancer port
// 4321, source mounted at /app).
export default defineConfig({
  site: 'https://mechcrate.dev',

  integrations: [
    starlight({
      title: 'MechCrate',
      description:
        'mx — a Rust CLI for running a subset of your service ecosystem locally, scaffolding projects with wisdom, and feeding agents a curated techniques corpus.',
      social: [
        {
          icon: 'github',
          label: 'GitHub',
          href: 'https://github.com/Dev916/mech-crate',
        },
      ],
      // Pagefind stays on (Starlight's default) — it is the human-facing search;
      // the RAG corpus is the agent-facing one.
      pagefind: true,
      // Two overrides, both additive — each renders Starlight's own component
      // and wraps it rather than replacing it.
      //
      //   MarkdownContent — corpus page chrome: a banner ("…agents retrieve it
      //     via `rag_context`") and, where the research pipeline recorded one, a
      //     provenance footer. Gated on `data.corpus`, so authored pages are
      //     untouched.
      //   Head — the per-page `og:image` tags. Starlight's `head:` config array
      //     only takes static entries, and every page's card URL is different.
      components: {
        MarkdownContent: './src/components/MarkdownContent.astro',
        Head: './src/components/Head.astro',
      },
      sidebar: [
        { label: 'Start', autogenerate: { directory: 'docs/start' } },
        { label: 'Framework', autogenerate: { directory: 'docs/framework' } },
        { label: 'AI Layer', autogenerate: { directory: 'docs/ai' } },
        {
          label: 'Techniques Corpus',
          // The corpus documents are collection entries, so `autogenerate` groups
          // them by their `category` directory for free. The overview is a
          // generated Astro page (src/pages/docs/corpus/), not a collection
          // entry, so it has to be linked explicitly; the per-category index
          // pages are reached from it and from each doc's banner.
          items: [
            { label: 'Overview', link: '/docs/corpus/' },
            { label: 'Categories', autogenerate: { directory: 'docs/corpus' } },
          ],
        },
        { label: 'Project', autogenerate: { directory: 'docs/project' } },
      ],
    }),

    // Must come after starlight(): Starlight injects its own `@astrojs/sitemap`
    // only when the user has not supplied one, and it decides that by scanning
    // `config.integrations` during its own config:setup. Declaring it here wins
    // that check, so the site gets ONE sitemap — the same URL set Starlight
    // would have produced (its config is empty for a single-locale site), now
    // with a verifiable `<lastmod>` on every entry.
    ...mechcrateSitemap(),
  ],

  server: {
    host: true,
    port: 4321,
  },
});
