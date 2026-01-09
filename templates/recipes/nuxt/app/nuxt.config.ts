// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-04-03',
  devtools: { enabled: true },
  
  ssr: {{SSR_ENABLED}},
  
  modules: [
    '@nuxtjs/tailwindcss',
  ],
  
  css: ['~/assets/css/main.css'],
  
  runtimeConfig: {
    // Server-only keys
    apiSecret: process.env.API_SECRET || '',
    
    // Public keys (exposed to client)
    public: {
      apiBase: process.env.NUXT_PUBLIC_API_BASE || '/api',
    },
  },
  
  nitro: {
    preset: 'node-server',
  },
  
  app: {
    head: {
      title: '{{SERVICE_NAME}}',
      meta: [
        { charset: 'utf-8' },
        { name: 'viewport', content: 'width=device-width, initial-scale=1' },
      ],
    },
  },
})
