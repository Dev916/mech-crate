import { defineConfig } from 'astro/config';
import node from '@astrojs/node';
import vue from '@astrojs/vue';
import tailwind from '@astrojs/tailwind';

// https://astro.build/config
export default defineConfig({
  output: 'server',

  adapter: node({
    mode: 'standalone',
  }),

  integrations: [
    vue({
      appEntrypoint: '/src/lib/vue-app',
      jsx: false,
    }),
    tailwind({
      applyBaseStyles: false,
    }),
  ],

  server: {
    host: true,
    port: 4321,
  },

  vite: {
    ssr: {
      noExternal: ['primevue', '@primevue/themes'],
    },
    optimizeDeps: {
      include: ['vue', 'pinia', '@vueuse/core', 'radix-vue'],
    },
  },

  // Environment variables:
  // - Variables prefixed with PUBLIC_ are available on both client and server
  // - Non-prefixed variables are server-side only (for security)
  // Access via import.meta.env.VARIABLE_NAME
});
