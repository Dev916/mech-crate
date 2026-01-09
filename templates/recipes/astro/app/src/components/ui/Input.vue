<script setup lang="ts">
/**
 * Input Component
 * 
 * A styled input field with Apple-inspired focus states.
 */
import { computed } from 'vue';
import { cn } from '@/lib/utils';

interface Props {
  type?: 'text' | 'email' | 'password' | 'number' | 'search' | 'tel' | 'url';
  placeholder?: string;
  disabled?: boolean;
  modelValue?: string | number;
  error?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  type: 'text',
  disabled: false,
  error: false,
});

const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

const classes = computed(() => 
  cn(
    'flex h-10 w-full rounded-lg border bg-background px-3 py-2 text-body',
    'file:border-0 file:bg-transparent file:text-sm file:font-medium',
    'placeholder:text-muted-foreground',
    'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
    'disabled:cursor-not-allowed disabled:opacity-50',
    'transition-all duration-200',
    props.error 
      ? 'border-destructive focus-visible:ring-destructive' 
      : 'border-input',
  )
);

function onInput(event: Event) {
  const target = event.target as HTMLInputElement;
  emit('update:modelValue', target.value);
}
</script>

<template>
  <input
    :type="type"
    :class="classes"
    :placeholder="placeholder"
    :disabled="disabled"
    :value="modelValue"
    @input="onInput"
  />
</template>
