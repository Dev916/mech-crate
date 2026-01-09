<script setup lang="ts">
/**
 * Toast Component
 * 
 * Displays notifications from the app store.
 * Uses Vue's Teleport to render at the document root.
 */
import { computed } from 'vue';
import { useAppStore } from '@/stores/app';
import { cn } from '@/lib/utils';
import { X, Info, CheckCircle, AlertTriangle, AlertCircle } from 'lucide-vue-next';

const store = useAppStore();

const iconMap = {
  info: Info,
  success: CheckCircle,
  warning: AlertTriangle,
  error: AlertCircle,
};

const colorMap = {
  info: 'bg-apple-blue/10 text-apple-blue border-apple-blue/20',
  success: 'bg-apple-green/10 text-apple-green border-apple-green/20',
  warning: 'bg-apple-orange/10 text-apple-orange border-apple-orange/20',
  error: 'bg-apple-red/10 text-apple-red border-apple-red/20',
};
</script>

<template>
  <Teleport to="body">
    <div 
      class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm w-full"
      role="region"
      aria-label="Notifications"
    >
      <TransitionGroup name="toast">
        <div
          v-for="notification in store.notifications"
          :key="notification.id"
          :class="cn(
            'relative flex items-start gap-3 p-4 rounded-xl border backdrop-blur-sm shadow-elevation-2 animate-slide-in',
            colorMap[notification.type]
          )"
          role="alert"
        >
          <!-- Icon -->
          <component 
            :is="iconMap[notification.type]" 
            class="w-5 h-5 shrink-0 mt-0.5" 
          />
          
          <!-- Content -->
          <div class="flex-1 min-w-0">
            <p class="font-medium text-sm">
              {{ notification.title }}
            </p>
            <p v-if="notification.message" class="text-sm opacity-80 mt-0.5">
              {{ notification.message }}
            </p>
          </div>
          
          <!-- Dismiss button -->
          <button
            class="shrink-0 p-1 rounded-md hover:bg-black/10 dark:hover:bg-white/10 transition-colors"
            @click="store.dismissNotification(notification.id)"
            aria-label="Dismiss notification"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(100%);
}

.toast-move {
  transition: transform 0.3s ease;
}
</style>
