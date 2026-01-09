import type { ApiResponse } from '@/types';

/**
 * API Client for type-safe data fetching
 * 
 * Works on both server (SSR) and client (islands) contexts.
 * Provides consistent error handling and response typing.
 */

const DEFAULT_BASE_URL = typeof window !== 'undefined' 
  ? '' 
  : (import.meta.env.APP_URL || 'http://localhost:4321');

interface FetchOptions extends RequestInit {
  baseUrl?: string;
  params?: Record<string, string | number | boolean | undefined>;
}

/**
 * Build URL with query params
 */
function buildUrl(path: string, params?: FetchOptions['params'], baseUrl = DEFAULT_BASE_URL): string {
  const url = new URL(path, baseUrl);
  
  if (params) {
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined) {
        url.searchParams.set(key, String(value));
      }
    });
  }
  
  return url.toString();
}

/**
 * Type-safe fetch wrapper
 */
async function fetchApi<T>(
  path: string,
  options: FetchOptions = {}
): Promise<ApiResponse<T>> {
  const { baseUrl, params, ...fetchOptions } = options;
  const url = buildUrl(path, params, baseUrl);

  try {
    const response = await fetch(url, {
      ...fetchOptions,
      headers: {
        'Content-Type': 'application/json',
        ...fetchOptions.headers,
      },
    });

    const data = await response.json() as ApiResponse<T>;

    if (!response.ok) {
      return {
        success: false,
        error: data.error || {
          code: 'HTTP_ERROR',
          message: `Request failed with status ${response.status}`,
        },
      };
    }

    return data;
  } catch (error) {
    console.error('[API Client] Request failed:', error);
    return {
      success: false,
      error: {
        code: 'NETWORK_ERROR',
        message: error instanceof Error ? error.message : 'Network request failed',
      },
    };
  }
}

/**
 * API Client methods
 */
export const api = {
  /**
   * GET request
   */
  async get<T>(path: string, options?: Omit<FetchOptions, 'method' | 'body'>): Promise<ApiResponse<T>> {
    return fetchApi<T>(path, { ...options, method: 'GET' });
  },

  /**
   * POST request
   */
  async post<T>(path: string, body?: unknown, options?: Omit<FetchOptions, 'method' | 'body'>): Promise<ApiResponse<T>> {
    return fetchApi<T>(path, {
      ...options,
      method: 'POST',
      body: body ? JSON.stringify(body) : undefined,
    });
  },

  /**
   * PUT request
   */
  async put<T>(path: string, body?: unknown, options?: Omit<FetchOptions, 'method' | 'body'>): Promise<ApiResponse<T>> {
    return fetchApi<T>(path, {
      ...options,
      method: 'PUT',
      body: body ? JSON.stringify(body) : undefined,
    });
  },

  /**
   * PATCH request
   */
  async patch<T>(path: string, body?: unknown, options?: Omit<FetchOptions, 'method' | 'body'>): Promise<ApiResponse<T>> {
    return fetchApi<T>(path, {
      ...options,
      method: 'PATCH',
      body: body ? JSON.stringify(body) : undefined,
    });
  },

  /**
   * DELETE request
   */
  async delete<T>(path: string, options?: Omit<FetchOptions, 'method' | 'body'>): Promise<ApiResponse<T>> {
    return fetchApi<T>(path, { ...options, method: 'DELETE' });
  },
};

export default api;
