import { defineConfig } from 'vitest/config';

/**
 * Unit tests for the corpus content pipeline (`src/loaders/lib/**`) and the
 * pure helpers behind the corpus page chrome (`src/components/corpus.ts`).
 *
 * Plain Vite — no Astro plugin — because every unit under test is a pure module
 * whose only `astro:*` references are type-only imports (erased at transform).
 * The Astro-facing halves (`src/loaders/corpus.ts`, the `.astro` components) are
 * verified by `astro build` itself.
 */
export default defineConfig({
  test: {
    include: ['src/**/__tests__/**/*.test.ts'],
    environment: 'node',
  },
});
