import type { Config } from 'tailwindcss'
import daisyui from 'daisyui'

export default {
  content: [
    './components/**/*.{js,vue,ts}',
    './layouts/**/*.vue',
    './pages/**/*.vue',
    './plugins/**/*.{js,ts}',
    './app.vue',
  ],
  theme: {
    extend: {},
  },
  plugins: [daisyui],
  daisyui: {
    themes: ['light', 'dark', 'cupcake'],
  },
} satisfies Config
