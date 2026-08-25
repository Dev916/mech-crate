import { defineConfig } from 'vitest/config';

/**
 * Unit tests for the corpus content pipeline (`src/loaders/lib/**`).
 *
 * Plain Vite — no Astro plugin — because every unit under test is a pure module
 * with no `astro:*` imports. The Astro-facing half (`src/loaders/corpus.ts`) is
 * verified by `astro build` itself.
 */
export default defineConfig({
  test: {
    include: ['src/loaders/__tests__/**/*.test.ts'],
    environment: 'node',
  },
});
