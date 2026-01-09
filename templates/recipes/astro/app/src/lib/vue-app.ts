/**
 * Vue App Entry Point for Astro
 * Configures Pinia, PrimeVue, and global plugins
 */
import type { App } from 'vue';
import { createPinia } from 'pinia';
import PrimeVue from 'primevue/config';
import Aura from '@primevue/themes/aura';
import ToastService from 'primevue/toastservice';
import ConfirmationService from 'primevue/confirmationservice';

export default (app: App) => {
  // Pinia state management
  const pinia = createPinia();
  app.use(pinia);

  // PrimeVue with Aura theme
  app.use(PrimeVue, {
    theme: {
      preset: Aura,
      options: {
        prefix: 'p',
        darkModeSelector: '.dark',
        cssLayer: {
          name: 'primevue',
          order: 'tailwind-base, primevue, tailwind-utilities',
        },
      },
    },
    ripple: true,
    inputStyle: 'outlined',
  });

  // PrimeVue services
  app.use(ToastService);
  app.use(ConfirmationService);
};
