/**
 * Core application types
 */

// Re-export database types
export type { User, NewUser, Session, NewSession, Setting } from '@/lib/db/schema';

/**
 * API Response wrapper
 */
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: unknown;
  };
  meta?: {
    timestamp: string;
    requestId?: string;
  };
}

/**
 * Pagination parameters
 */
export interface PaginationParams {
  page?: number;
  limit?: number;
  cursor?: string;
}

/**
 * Paginated response
 */
export interface PaginatedResponse<T> {
  items: T[];
  pagination: {
    total: number;
    page: number;
    limit: number;
    hasMore: boolean;
    nextCursor?: string;
  };
}

/**
 * Sort direction
 */
export type SortDirection = 'asc' | 'desc';

/**
 * Generic filter params
 */
export interface FilterParams<T extends string = string> {
  field: T;
  operator: 'eq' | 'ne' | 'gt' | 'gte' | 'lt' | 'lte' | 'like' | 'in';
  value: unknown;
}

/**
 * Theme types
 */
export type Theme = 'light' | 'dark' | 'system';

/**
 * Language types for i18n
 */
export type Language = 'en' | 'ja';

/**
 * Notification types
 */
export type NotificationType = 'info' | 'success' | 'warning' | 'error';

export interface Notification {
  id: string;
  type: NotificationType;
  title: string;
  message?: string;
  duration?: number;
  dismissible?: boolean;
}
