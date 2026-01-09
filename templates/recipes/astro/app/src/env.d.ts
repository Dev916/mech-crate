/// <reference path="../.astro/types.d.ts" />
/// <reference types="astro/client" />

/**
 * Environment Variables Type Definitions
 *
 * Astro uses Vite's built-in environment variable support:
 * - Variables prefixed with PUBLIC_ are available on both client and server
 * - Non-prefixed variables are server-side only (for security)
 *
 * Access via: import.meta.env.VARIABLE_NAME
 */
interface ImportMetaEnv {
  // ============================================
  // SERVER-SIDE ONLY Variables
  // ============================================
  // These are ONLY accessible in Astro frontmatter and API routes
  // They will be `undefined` in client-side Vue/React components

  /** PostgreSQL connection string (server-only) */
  readonly DATABASE_URL: string;

  /** Redis connection string (server-only) */
  readonly REDIS_URL: string;

  /** Node environment */
  readonly NODE_ENV: 'development' | 'production' | 'test';

  /** Server port */
  readonly PORT: string;

  /** Session secret for authentication (server-only) */
  readonly SESSION_SECRET: string;

  /** API secret key (server-only) */
  readonly API_SECRET_KEY: string;

  // ============================================
  // PUBLIC Variables (available on client AND server)
  // ============================================
  // These are accessible everywhere, including Vue components
  // Prefix with PUBLIC_ to expose to client

  /** Application name (public) */
  readonly PUBLIC_APP_NAME: string;

  /** Application URL (public) */
  readonly PUBLIC_APP_URL: string;

  /** API base URL for client-side requests (public) */
  readonly PUBLIC_API_BASE_URL: string;

  /** Enable analytics tracking (public) */
  readonly PUBLIC_ENABLE_ANALYTICS: string;

  /** Enable debug mode (public) */
  readonly PUBLIC_ENABLE_DEBUG_MODE: string;

  /** CDN URL for static assets (public) */
  readonly PUBLIC_CDN_URL: string;

  /** Sentry DSN for error tracking (public) */
  readonly PUBLIC_SENTRY_DSN: string;

  // ============================================
  // Built-in Astro/Vite Variables
  // ============================================

  /** True during server-side rendering */
  readonly SSR: boolean;

  /** True during development */
  readonly DEV: boolean;

  /** True during production build */
  readonly PROD: boolean;

  /** Build mode ('development' | 'production') */
  readonly MODE: string;

  /** Base URL path */
  readonly BASE_URL: string;

  /** Site URL (if configured in astro.config.mjs) */
  readonly SITE: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
