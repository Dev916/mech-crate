<script setup lang="ts">
import { ref } from 'vue';
import Card from '@/components/ui/Card.vue';
import { 
  Zap, 
  Layers, 
  Database, 
  Globe, 
  Palette, 
  Shield 
} from 'lucide-vue-next';

const features = ref([
  {
    icon: Zap,
    title: 'Lightning Fast',
    description: 'Zero-JS by default with Astro. Only ship JavaScript when you need interactivity.',
    color: 'from-apple-yellow to-apple-orange',
  },
  {
    icon: Layers,
    title: 'Vue 3 Islands',
    description: 'Interactive Vue components hydrate on demand. Best of both static and dynamic worlds.',
    color: 'from-apple-green to-apple-teal',
  },
  {
    icon: Database,
    title: 'Full-Stack Ready',
    description: 'PostgreSQL + Redis connections built-in. Type-safe database queries with Drizzle ORM.',
    color: 'from-apple-blue to-apple-indigo',
  },
  {
    icon: Globe,
    title: 'Edge Deployable',
    description: 'Cloudflare Containers ready. Multi-stage Docker builds for dev and production.',
    color: 'from-apple-indigo to-apple-purple',
  },
  {
    icon: Palette,
    title: 'Apple-Inspired Design',
    description: 'Beautiful UI with shadcn-vue, PrimeVue, and carefully crafted Tailwind theme.',
    color: 'from-apple-purple to-apple-pink',
  },
  {
    icon: Shield,
    title: 'Type-Safe Everywhere',
    description: 'TypeScript throughout. Zod validation, strict mode enabled, zero runtime surprises.',
    color: 'from-apple-pink to-apple-red',
  },
]);
</script>

<template>
  <section class="py-24">
    <div class="text-center mb-16">
      <h2 class="text-title-1 font-semibold text-foreground mb-4">
        Everything you need
      </h2>
      <p class="text-body-large text-muted-foreground max-w-2xl mx-auto">
        A complete foundation for building modern web applications 
        with best practices baked in.
      </p>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      <Card 
        v-for="(feature, index) in features" 
        :key="feature.title"
        class="group hover:shadow-elevation-3 transition-all duration-300"
        :style="{ animationDelay: `${index * 100}ms` }"
      >
        <div class="p-6">
          <!-- Icon -->
          <div 
            class="w-12 h-12 rounded-xl bg-gradient-to-br mb-4 flex items-center justify-center transition-transform group-hover:scale-110"
            :class="feature.color"
          >
            <component :is="feature.icon" class="w-6 h-6 text-white" />
          </div>

          <!-- Content -->
          <h3 class="text-title-3 font-semibold text-foreground mb-2">
            {{ feature.title }}
          </h3>
          <p class="text-subhead text-muted-foreground">
            {{ feature.description }}
          </p>
        </div>
      </Card>
    </div>
  </section>
</template>
