<script setup lang="ts">
/**
 * Spinner Component
 * 
 * Loading indicator with Apple-style animation.
 */
import { computed } from 'vue';
import { cn } from '@/lib/utils';

interface Props {
  size?: 'sm' | 'md' | 'lg';
  color?: 'default' | 'primary' | 'white';
}

const props = withDefaults(defineProps<Props>(), {
  size: 'md',
  color: 'default',
});

const sizeClasses = {
  sm: 'w-4 h-4',
  md: 'w-6 h-6',
  lg: 'w-8 h-8',
};

const colorClasses = {
  default: 'text-muted-foreground',
  primary: 'text-primary',
  white: 'text-white',
};

const classes = computed(() => 
  cn('animate-spin', sizeClasses[props.size], colorClasses[props.color])
);
</script>

<template>
  <svg 
    :class="classes" 
    xmlns="http://www.w3.org/2000/svg" 
    fill="none" 
    viewBox="0 0 24 24"
    aria-hidden="true"
  >
    <circle
      class="opacity-25"
      cx="12"
      cy="12"
      r="10"
      stroke="currentColor"
      stroke-width="4"
    />
    <path
      class="opacity-75"
      fill="currentColor"
      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
    />
  </svg>
</template>
