<script setup lang="ts">
/**
 * BilingualText Component
 * 
 * Displays text in both English and Japanese, with language switching
 * driven by the global app store. Useful for internationalized UIs
 * with a primary/secondary language display pattern.
 * 
 * Usage:
 *   <BilingualText en="Hello" ja="こんにちは" />
 *   <BilingualText en="Welcome" ja="ようこそ" :showBoth="true" />
 */
import { computed } from 'vue';
import { useAppStore } from '@/stores/app';

interface Props {
  en: string;
  ja: string;
  showBoth?: boolean;
  separator?: string;
  enClass?: string;
  jaClass?: string;
}

const props = withDefaults(defineProps<Props>(), {
  showBoth: false,
  separator: ' / ',
});

const store = useAppStore();

const primaryText = computed(() => 
  store.language === 'ja' ? props.ja : props.en
);

const secondaryText = computed(() => 
  store.language === 'ja' ? props.en : props.ja
);
</script>

<template>
  <span v-if="showBoth" class="bilingual-text">
    <span :class="['bilingual-primary', enClass]">{{ primaryText }}</span>
    <span class="bilingual-separator text-muted-foreground">{{ separator }}</span>
    <span :class="['bilingual-secondary text-muted-foreground text-sm', jaClass]">{{ secondaryText }}</span>
  </span>
  <span v-else>{{ primaryText }}</span>
</template>

<style scoped>
.bilingual-text {
  display: inline-flex;
  align-items: baseline;
  gap: 0.25rem;
}

.bilingual-separator {
  opacity: 0.5;
}
</style>
