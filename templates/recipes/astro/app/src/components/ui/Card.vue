<script setup lang="ts">
/**
 * Card Component
 * 
 * A versatile card container with Apple-inspired styling.
 * Supports glass morphism variant for modern UI.
 */
import { computed } from 'vue';
import { cn } from '@/lib/utils';

interface Props {
  variant?: 'default' | 'glass' | 'bordered';
  interactive?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  interactive: false,
});

const baseStyles = 'rounded-xl overflow-hidden';

const variantStyles = {
  default: 'bg-card text-card-foreground shadow-elevation-1',
  glass: 'glass',
  bordered: 'border border-border bg-card text-card-foreground',
};

const interactiveStyles = 'cursor-pointer hover:shadow-elevation-2 transition-shadow duration-200';

const classes = computed(() => 
  cn(
    baseStyles,
    variantStyles[props.variant],
    props.interactive && interactiveStyles,
  )
);
</script>

<template>
  <div :class="classes">
    <slot />
  </div>
</template>
