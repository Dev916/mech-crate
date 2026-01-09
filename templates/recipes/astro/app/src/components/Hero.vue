<script setup lang="ts">
import { ref, onMounted } from 'vue';
import Button from '@/components/ui/Button.vue';
import { ArrowRight, Sparkles } from 'lucide-vue-next';

const isVisible = ref(false);

onMounted(() => {
  // Trigger entrance animation
  requestAnimationFrame(() => {
    isVisible.value = true;
  });
});
</script>

<template>
  <section 
    class="relative py-24 md:py-32 overflow-hidden"
    :class="{ 'animate-enter': isVisible }"
  >
    <!-- Background gradient -->
    <div class="absolute inset-0 bg-gradient-to-b from-primary/5 via-transparent to-transparent" />
    
    <!-- Decorative elements -->
    <div class="absolute top-1/4 left-1/4 w-96 h-96 bg-apple-blue/10 rounded-full blur-3xl" />
    <div class="absolute bottom-1/4 right-1/4 w-80 h-80 bg-apple-teal/10 rounded-full blur-3xl" />

    <div class="relative container mx-auto px-4 text-center">
      <!-- Badge -->
      <div 
        class="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-primary/10 text-primary mb-8"
        :style="{ transitionDelay: '100ms' }"
      >
        <Sparkles class="w-4 h-4" />
        <span class="text-footnote font-medium">Astro + Vue 3 + SSR</span>
      </div>

      <!-- Headline -->
      <h1 
        class="text-headline md:text-display font-bold text-foreground mb-6"
        :style="{ transitionDelay: '200ms' }"
      >
        Build something
        <span class="text-gradient-primary">extraordinary</span>
      </h1>

      <!-- Subheadline -->
      <p 
        class="text-body-large text-muted-foreground max-w-2xl mx-auto mb-10"
        :style="{ transitionDelay: '300ms' }"
      >
        A modern full-stack starter with Astro's islands architecture, 
        Vue 3 reactivity, and Apple-inspired design. 
        Deploy anywhere with Cloudflare Containers.
      </p>

      <!-- CTAs -->
      <div 
        class="flex flex-col sm:flex-row gap-4 justify-center"
        :style="{ transitionDelay: '400ms' }"
      >
        <Button size="lg" class="group">
          Get Started
          <ArrowRight class="w-4 h-4 ml-2 transition-transform group-hover:translate-x-1" />
        </Button>
        <Button variant="outline" size="lg">
          View Documentation
        </Button>
      </div>

      <!-- Stats -->
      <div 
        class="grid grid-cols-3 gap-8 max-w-xl mx-auto mt-16"
        :style="{ transitionDelay: '500ms' }"
      >
        <div class="text-center">
          <div class="text-title-2 font-bold text-foreground">100%</div>
          <div class="text-footnote text-muted-foreground">Type-safe</div>
        </div>
        <div class="text-center">
          <div class="text-title-2 font-bold text-foreground">SSR</div>
          <div class="text-footnote text-muted-foreground">Ready</div>
        </div>
        <div class="text-center">
          <div class="text-title-2 font-bold text-foreground">Edge</div>
          <div class="text-footnote text-muted-foreground">Deployable</div>
        </div>
      </div>
    </div>
  </section>
</template>
