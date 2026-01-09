<script setup lang="ts">
/**
 * EnvDemo Component
 *
 * Demonstrates how to access PUBLIC_ prefixed environment variables
 * in client-side Vue components.
 *
 * IMPORTANT: Only variables prefixed with PUBLIC_ are accessible here!
 * - ✅ import.meta.env.PUBLIC_APP_NAME  (works)
 * - ❌ import.meta.env.DATABASE_URL     (undefined - server only!)
 */

import { ref, onMounted } from 'vue';

// Props passed from Astro (can include server-computed values)
const props = defineProps<{
  appName?: string;
  apiUrl?: string;
}>();

// Accessing PUBLIC_ env vars directly in client-side code
// These are statically replaced at build time by Vite
const publicAppName = import.meta.env.PUBLIC_APP_NAME;
const publicAppUrl = import.meta.env.PUBLIC_APP_URL;
const publicApiUrl = import.meta.env.PUBLIC_API_BASE_URL;
const debugMode = import.meta.env.PUBLIC_ENABLE_DEBUG_MODE === 'true';

// Demonstrate that non-PUBLIC_ vars are undefined on client
const databaseUrl = import.meta.env.DATABASE_URL; // Will be undefined!
const sessionSecret = import.meta.env.SESSION_SECRET; // Will be undefined!

const mounted = ref(false);
const apiStatus = ref<'loading' | 'success' | 'error'>('loading');

onMounted(async () => {
  mounted.value = true;

  // Example: Using PUBLIC_API_BASE_URL to make a client-side API call
  if (publicApiUrl) {
    try {
      const response = await fetch(`${publicApiUrl}/health`);
      apiStatus.value = response.ok ? 'success' : 'error';
    } catch {
      apiStatus.value = 'error';
    }
  }
});
</script>

<template>
  <div class="max-w-2xl mx-auto p-6 bg-primary/5 rounded-xl border border-primary/20">
    <h3 class="font-semibold text-lg mb-4 text-foreground flex items-center gap-2">
      <span>🌐</span>
      <span>Client-Side Environment Variables (Vue Component)</span>
    </h3>

    <div class="space-y-4">
      <!-- PUBLIC_ variables (accessible) -->
      <div class="p-4 bg-emerald-500/10 rounded-lg border border-emerald-500/20">
        <h4 class="text-sm font-medium text-emerald-600 dark:text-emerald-400 mb-3">
          ✅ PUBLIC_ Variables (accessible on client)
        </h4>
        <dl class="space-y-2 text-sm font-mono">
          <div class="flex flex-wrap justify-between gap-2">
            <dt class="text-muted-foreground">PUBLIC_APP_NAME:</dt>
            <dd class="text-foreground">{{ publicAppName || '(not set)' }}</dd>
          </div>
          <div class="flex flex-wrap justify-between gap-2">
            <dt class="text-muted-foreground">PUBLIC_APP_URL:</dt>
            <dd class="text-foreground">{{ publicAppUrl || '(not set)' }}</dd>
          </div>
          <div class="flex flex-wrap justify-between gap-2">
            <dt class="text-muted-foreground">PUBLIC_API_BASE_URL:</dt>
            <dd class="text-foreground">{{ publicApiUrl || '(not set)' }}</dd>
          </div>
          <div class="flex flex-wrap justify-between gap-2">
            <dt class="text-muted-foreground">PUBLIC_ENABLE_DEBUG_MODE:</dt>
            <dd class="text-foreground">{{ debugMode ? 'true' : 'false' }}</dd>
          </div>
        </dl>
      </div>

      <!-- Non-PUBLIC variables (inaccessible) -->
      <div class="p-4 bg-rose-500/10 rounded-lg border border-rose-500/20">
        <h4 class="text-sm font-medium text-rose-600 dark:text-rose-400 mb-3">
          ❌ Server-Only Variables (undefined on client)
        </h4>
        <dl class="space-y-2 text-sm font-mono">
          <div class="flex flex-wrap justify-between gap-2">
            <dt class="text-muted-foreground">DATABASE_URL:</dt>
            <dd class="text-rose-500">{{ databaseUrl || 'undefined (correct!)' }}</dd>
          </div>
          <div class="flex flex-wrap justify-between gap-2">
            <dt class="text-muted-foreground">SESSION_SECRET:</dt>
            <dd class="text-rose-500">{{ sessionSecret || 'undefined (correct!)' }}</dd>
          </div>
        </dl>
        <p class="mt-3 text-xs text-muted-foreground">
          This is the expected behavior - sensitive variables should never be exposed to the client!
        </p>
      </div>

      <!-- Props from Astro -->
      <div class="p-4 bg-blue-500/10 rounded-lg border border-blue-500/20">
        <h4 class="text-sm font-medium text-blue-600 dark:text-blue-400 mb-3">
          📦 Props Passed from Astro Server
        </h4>
        <dl class="space-y-2 text-sm font-mono">
          <div class="flex flex-wrap justify-between gap-2">
            <dt class="text-muted-foreground">appName (prop):</dt>
            <dd class="text-foreground">{{ props.appName || '(not passed)' }}</dd>
          </div>
          <div class="flex flex-wrap justify-between gap-2">
            <dt class="text-muted-foreground">apiUrl (prop):</dt>
            <dd class="text-foreground">{{ props.apiUrl || '(not passed)' }}</dd>
          </div>
        </dl>
        <p class="mt-3 text-xs text-muted-foreground">
          Server can pass computed values to client components via props.
        </p>
      </div>

      <!-- Runtime check -->
      <div v-if="mounted" class="p-4 bg-card rounded-lg border border-border">
        <h4 class="text-sm font-medium text-foreground mb-3">
          🔄 Runtime API Check
        </h4>
        <div class="flex items-center gap-2 text-sm">
          <span class="text-muted-foreground">API Status:</span>
          <span v-if="apiStatus === 'loading'" class="text-yellow-500">
            ⏳ Checking...
          </span>
          <span v-else-if="apiStatus === 'success'" class="text-emerald-500">
            ✅ Connected
          </span>
          <span v-else class="text-rose-500">
            ❌ Not available
          </span>
        </div>
        <p class="mt-2 text-xs text-muted-foreground">
          Using PUBLIC_API_BASE_URL to check <code>{{ publicApiUrl }}/health</code>
        </p>
      </div>
    </div>
  </div>
</template>
