import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import type { Ref, ComputedRef } from 'vue';

/**
 * Theme type
 */
export type Theme = 'light' | 'dark' | 'system';

/**
 * Language type for bilingual support
 */
export type Language = 'en' | 'ja';

/**
 * Notification type
 */
export interface Notification {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  title: string;
  message?: string;
  duration?: number;
}

/**
 * Global App Store
 * 
 * FRP-style reactive state management using Pinia + Composition API.
 * All state flows unidirectionally from stores to components.
 */
export const useAppStore = defineStore('app', () => {
  // ============================================================
  // State (Reactive References)
  // ============================================================
  
  const theme: Ref<Theme> = ref('system');
  const language: Ref<Language> = ref('en');
  const isLoading: Ref<boolean> = ref(false);
  const sidebarOpen: Ref<boolean> = ref(true);
  const notifications: Ref<Notification[]> = ref([]);
  
  // User state (hydrated from server)
  const user: Ref<{
    id: string;
    email: string;
    name?: string;
    avatarUrl?: string;
  } | null> = ref(null);

  // ============================================================
  // Computed Properties (Derived State)
  // ============================================================
  
  const isDark: ComputedRef<boolean> = computed(() => {
    if (theme.value === 'system') {
      if (typeof window !== 'undefined') {
        return window.matchMedia('(prefers-color-scheme: dark)').matches;
      }
      return false;
    }
    return theme.value === 'dark';
  });
  
  const isAuthenticated: ComputedRef<boolean> = computed(() => user.value !== null);
  
  const hasNotifications: ComputedRef<boolean> = computed(() => notifications.value.length > 0);

  // ============================================================
  // Actions (State Mutations)
  // ============================================================
  
  /**
   * Set application theme
   */
  function setTheme(newTheme: Theme): void {
    theme.value = newTheme;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('theme', newTheme);
    }
  }
  
  /**
   * Toggle theme between light and dark
   */
  function toggleTheme(): void {
    setTheme(isDark.value ? 'light' : 'dark');
  }
  
  /**
   * Set application language
   */
  function setLanguage(lang: Language): void {
    language.value = lang;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('language', lang);
    }
  }
  
  /**
   * Toggle sidebar visibility
   */
  function toggleSidebar(): void {
    sidebarOpen.value = !sidebarOpen.value;
  }
  
  /**
   * Set loading state
   */
  function setLoading(state: boolean): void {
    isLoading.value = state;
  }
  
  /**
   * Set user (from server hydration or login)
   */
  function setUser(userData: typeof user.value): void {
    user.value = userData;
  }
  
  /**
   * Clear user (logout)
   */
  function clearUser(): void {
    user.value = null;
  }
  
  /**
   * Add notification
   */
  function notify(notification: Omit<Notification, 'id'>): string {
    const id = Math.random().toString(36).substring(2, 9);
    notifications.value.push({ ...notification, id });
    
    // Auto-dismiss after duration
    const duration = notification.duration ?? 5000;
    if (duration > 0) {
      setTimeout(() => dismissNotification(id), duration);
    }
    
    return id;
  }
  
  /**
   * Dismiss notification by ID
   */
  function dismissNotification(id: string): void {
    notifications.value = notifications.value.filter((n) => n.id !== id);
  }
  
  /**
   * Clear all notifications
   */
  function clearNotifications(): void {
    notifications.value = [];
  }
  
  /**
   * Initialize store from localStorage
   */
  function hydrate(): void {
    if (typeof localStorage !== 'undefined') {
      const storedTheme = localStorage.getItem('theme') as Theme | null;
      if (storedTheme) {
        theme.value = storedTheme;
      }
      
      const storedLanguage = localStorage.getItem('language') as Language | null;
      if (storedLanguage) {
        language.value = storedLanguage;
      }
    }
  }

  // ============================================================
  // Watchers (Side Effects)
  // ============================================================
  
  // Apply dark mode class to document
  if (typeof window !== 'undefined') {
    watch(isDark, (dark) => {
      document.documentElement.classList.toggle('dark', dark);
    }, { immediate: true });
    
    // Listen for system theme changes
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      if (theme.value === 'system') {
        // Force reactivity update
        theme.value = 'system';
      }
    });
  }

  // ============================================================
  // Return Public API
  // ============================================================
  
  return {
    // State
    theme,
    language,
    isLoading,
    sidebarOpen,
    notifications,
    user,
    
    // Computed
    isDark,
    isAuthenticated,
    hasNotifications,
    
    // Actions
    setTheme,
    toggleTheme,
    setLanguage,
    toggleSidebar,
    setLoading,
    setUser,
    clearUser,
    notify,
    dismissNotification,
    clearNotifications,
    hydrate,
  };
});
