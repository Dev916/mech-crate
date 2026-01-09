import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import eslintPluginAstro from 'eslint-plugin-astro';
import eslintPluginVue from 'eslint-plugin-vue';

export default [
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...eslintPluginAstro.configs.recommended,
  ...eslintPluginVue.configs['flat/recommended'],
  {
    languageOptions: {
      parserOptions: {
        project: './tsconfig.json',
        extraFileExtensions: ['.astro', '.vue'],
      },
    },
    rules: {
      // TypeScript
      '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/consistent-type-imports': 'error',
      
      // Vue
      'vue/multi-word-component-names': 'off',
      'vue/no-v-html': 'off',
      'vue/require-default-prop': 'off',
      
      // General
      'no-console': ['warn', { allow: ['warn', 'error'] }],
    },
  },
  {
    ignores: ['dist/', 'node_modules/', '.astro/'],
  },
];
