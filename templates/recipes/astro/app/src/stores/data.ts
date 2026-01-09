import { defineStore } from 'pinia';
import { ref, computed, shallowRef } from 'vue';
import type { Ref, ComputedRef, ShallowRef } from 'vue';
import { api } from '@/lib/api/client';
import type { User, PaginatedResponse, ApiResponse } from '@/types';

/**
 * Data State
 * 
 * Generic state shape for async data with loading/error handling.
 * Follows FRP principles - state flows unidirectionally.
 */
interface DataState<T> {
  data: T | null;
  isLoading: boolean;
  error: string | null;
  lastFetched: number | null;
}

/**
 * Create initial data state
 */
function createDataState<T>(): DataState<T> {
  return {
    data: null,
    isLoading: false,
    error: null,
    lastFetched: null,
  };
}

/**
 * Data Store
 * 
 * Centralized store for server-fetched data with caching,
 * optimistic updates, and error handling.
 */
export const useDataStore = defineStore('data', () => {
  // ============================================================
  // State
  // ============================================================
  
  const users: Ref<DataState<PaginatedResponse<User>>> = ref(createDataState());
  
  // Cache TTL in milliseconds (5 minutes)
  const CACHE_TTL = 5 * 60 * 1000;

  // ============================================================
  // Computed
  // ============================================================
  
  const userList: ComputedRef<User[]> = computed(() => 
    users.value.data?.items ?? []
  );
  
  const usersLoading: ComputedRef<boolean> = computed(() => 
    users.value.isLoading
  );
  
  const usersError: ComputedRef<string | null> = computed(() => 
    users.value.error
  );
  
  const usersStale: ComputedRef<boolean> = computed(() => {
    if (!users.value.lastFetched) return true;
    return Date.now() - users.value.lastFetched > CACHE_TTL;
  });

  // ============================================================
  // Actions
  // ============================================================
  
  /**
   * Fetch users with caching
   */
  async function fetchUsers(force = false): Promise<void> {
    // Skip if cached and not forced
    if (!force && !usersStale.value && users.value.data) {
      return;
    }
    
    users.value = {
      ...users.value,
      isLoading: true,
      error: null,
    };
    
    const response = await api.get<PaginatedResponse<User>>('/api/users');
    
    if (response.success && response.data) {
      users.value = {
        data: response.data,
        isLoading: false,
        error: null,
        lastFetched: Date.now(),
      };
    } else {
      users.value = {
        ...users.value,
        isLoading: false,
        error: response.error?.message ?? 'Failed to fetch users',
      };
    }
  }
  
  /**
   * Create a new user (optimistic update)
   */
  async function createUser(userData: { email: string; name?: string }): Promise<boolean> {
    const response = await api.post<User>('/api/users', userData);
    
    if (response.success && response.data) {
      // Optimistic: add to local state
      if (users.value.data) {
        users.value.data.items = [response.data, ...users.value.data.items];
        users.value.data.pagination.total += 1;
      }
      return true;
    }
    
    return false;
  }
  
  /**
   * Clear all cached data
   */
  function clearCache(): void {
    users.value = createDataState();
  }
  
  /**
   * Invalidate specific resource
   */
  function invalidateUsers(): void {
    users.value.lastFetched = null;
  }

  // ============================================================
  // Return Public API
  // ============================================================
  
  return {
    // State
    users,
    
    // Computed
    userList,
    usersLoading,
    usersError,
    usersStale,
    
    // Actions
    fetchUsers,
    createUser,
    clearCache,
    invalidateUsers,
  };
});
