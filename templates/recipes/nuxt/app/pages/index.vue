<template>
  <div class="hero min-h-[60vh]">
    <div class="hero-content text-center">
      <div class="max-w-md">
        <h1 class="text-5xl font-bold">{{SERVICE_NAME}}</h1>
        <p class="py-6">
          Your Nuxt 3 application is ready. Start building something amazing.
        </p>
        <div class="flex gap-4 justify-center">
          <button class="btn btn-primary" @click="fetchStatus">
            Check API Status
          </button>
          <NuxtLink to="/about" class="btn btn-outline">
            Learn More
          </NuxtLink>
        </div>
        
        <div v-if="status" class="mt-8 alert alert-success">
          <span>API Status: {{ status.status }} at {{ status.timestamp }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const status = ref<{ status: string; timestamp: string } | null>(null)

async function fetchStatus() {
  try {
    status.value = await $fetch('/api/health')
  } catch (e) {
    console.error('Failed to fetch status:', e)
  }
}
</script>
