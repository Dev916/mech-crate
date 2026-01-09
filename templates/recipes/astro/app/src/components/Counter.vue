<script setup lang="ts">
/**
 * Interactive Counter Component
 * 
 * Demonstrates Vue 3 reactivity within Astro's island architecture.
 * This component only hydrates on the client when visible.
 */
import { ref, computed } from 'vue';
import Button from '@/components/ui/Button.vue';
import Card from '@/components/ui/Card.vue';
import { Plus, Minus, RotateCcw } from 'lucide-vue-next';

const count = ref(0);
const step = ref(1);

const isPositive = computed(() => count.value > 0);
const isNegative = computed(() => count.value < 0);
const isZero = computed(() => count.value === 0);

function increment() {
  count.value += step.value;
}

function decrement() {
  count.value -= step.value;
}

function reset() {
  count.value = 0;
}
</script>

<template>
  <Card class="max-w-md mx-auto overflow-hidden">
    <!-- Header with gradient -->
    <div class="bg-gradient-to-r from-apple-blue to-apple-teal p-6 text-white">
      <h3 class="text-title-3 font-semibold">Interactive Counter</h3>
      <p class="text-subhead opacity-90">Vue 3 island component</p>
    </div>

    <div class="p-6 space-y-6">
      <!-- Count Display -->
      <div class="text-center">
        <div 
          class="text-display font-bold transition-colors duration-200"
          :class="{
            'text-apple-green': isPositive,
            'text-apple-red': isNegative,
            'text-foreground': isZero,
          }"
        >
          {{ count }}
        </div>
        <p class="text-footnote text-muted-foreground mt-1">
          Current count
        </p>
      </div>

      <!-- Controls -->
      <div class="flex items-center justify-center gap-4">
        <Button 
          variant="outline" 
          size="icon"
          @click="decrement"
          aria-label="Decrement"
        >
          <Minus class="w-5 h-5" />
        </Button>

        <Button 
          variant="ghost" 
          size="icon"
          @click="reset"
          :disabled="isZero"
          aria-label="Reset"
        >
          <RotateCcw class="w-5 h-5" />
        </Button>

        <Button 
          variant="outline" 
          size="icon"
          @click="increment"
          aria-label="Increment"
        >
          <Plus class="w-5 h-5" />
        </Button>
      </div>

      <!-- Step Selector -->
      <div class="flex items-center justify-center gap-2">
        <span class="text-footnote text-muted-foreground">Step:</span>
        <div class="flex gap-1">
          <button
            v-for="s in [1, 5, 10]"
            :key="s"
            class="px-3 py-1 text-footnote rounded-md transition-colors"
            :class="step === s 
              ? 'bg-primary text-primary-foreground' 
              : 'bg-muted text-muted-foreground hover:bg-muted/80'"
            @click="step = s"
          >
            {{ s }}
          </button>
        </div>
      </div>
    </div>
  </Card>
</template>
