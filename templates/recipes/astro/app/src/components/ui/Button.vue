<script setup lang="ts">
/**
 * Button Component
 * 
 * A shadcn-vue style button with multiple variants and sizes.
 * Uses CVA (class-variance-authority) for variant management.
 */
import { computed } from 'vue';
import { cn } from '@/lib/utils';

interface Props {
  variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
  size?: 'default' | 'sm' | 'lg' | 'icon';
  disabled?: boolean;
  type?: 'button' | 'submit' | 'reset';
  asChild?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  size: 'default',
  disabled: false,
  type: 'button',
  asChild: false,
});

const baseStyles = 'inline-flex items-center justify-center whitespace-nowrap rounded-lg text-body font-medium ring-offset-background transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 active:scale-[0.98]';

const variantStyles = {
  default: 'bg-primary text-primary-foreground hover:bg-primary/90 shadow-elevation-1 hover:shadow-elevation-2',
  destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/90 shadow-elevation-1 hover:shadow-elevation-2',
  outline: 'border border-input bg-background hover:bg-accent hover:text-accent-foreground',
  secondary: 'bg-secondary text-secondary-foreground hover:bg-secondary/80',
  ghost: 'hover:bg-accent hover:text-accent-foreground',
  link: 'text-primary underline-offset-4 hover:underline',
};

const sizeStyles = {
  default: 'h-10 px-4 py-2',
  sm: 'h-9 rounded-md px-3',
  lg: 'h-12 rounded-xl px-8 text-body-large',
  icon: 'h-10 w-10',
};

const classes = computed(() => 
  cn(
    baseStyles,
    variantStyles[props.variant],
    sizeStyles[props.size],
  )
);
</script>

<template>
  <button
    :type="type"
    :class="classes"
    :disabled="disabled"
  >
    <slot />
  </button>
</template>
