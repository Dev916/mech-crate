import type { APIRoute } from 'astro';
import { db } from '@/lib/db';
import { redis } from '@/lib/redis';

/**
 * Health Check API Endpoint
 *
 * This demonstrates SERVER-SIDE environment variable access.
 * In API routes (server-side code), we can access ALL env vars:
 * - DATABASE_URL, REDIS_URL, SESSION_SECRET (server-only)
 * - PUBLIC_* vars (also available here)
 *
 * GET /api/health
 */
export const GET: APIRoute = async () => {
  // ============================================
  // Server-side environment variable access
  // ============================================
  // These are ONLY available in server-side code (API routes, Astro frontmatter)
  const nodeEnv = import.meta.env.NODE_ENV || 'development';
  const databaseUrl = import.meta.env.DATABASE_URL;
  const redisUrl = import.meta.env.REDIS_URL;

  // PUBLIC_ vars are also available server-side
  const appName = import.meta.env.PUBLIC_APP_NAME || 'App';
  const debugMode = import.meta.env.PUBLIC_ENABLE_DEBUG_MODE === 'true';

  const checks = {
    timestamp: new Date().toISOString(),
    status: 'healthy' as 'healthy' | 'degraded' | 'unhealthy',
    app: {
      name: appName,
      environment: nodeEnv,
      debugMode,
    },
    services: {
      database: {
        status: 'unknown' as 'healthy' | 'unhealthy' | 'unknown',
        latency: 0,
        // Only show connection info in debug mode
        ...(debugMode && { configured: !!databaseUrl }),
      },
      redis: {
        status: 'unknown' as 'healthy' | 'unhealthy' | 'unknown',
        latency: 0,
        ...(debugMode && { configured: !!redisUrl }),
      },
    },
  };

  // Check database (using DATABASE_URL from server env)
  if (databaseUrl) {
    try {
      const start = performance.now();
      await db.execute`SELECT 1`;
      checks.services.database = {
        ...checks.services.database,
        status: 'healthy',
        latency: Math.round(performance.now() - start),
      };
    } catch (error) {
      checks.services.database = {
        ...checks.services.database,
        status: 'unhealthy',
        latency: 0,
      };
      checks.status = 'degraded';

      // Log error server-side (never expose to client)
      if (debugMode) {
        console.error('[Health] Database check failed:', error);
      }
    }
  }

  // Check Redis (using REDIS_URL from server env)
  if (redisUrl) {
    try {
      const start = performance.now();
      await redis.ping();
      checks.services.redis = {
        ...checks.services.redis,
        status: 'healthy',
        latency: Math.round(performance.now() - start),
      };
    } catch (error) {
      checks.services.redis = {
        ...checks.services.redis,
        status: 'unhealthy',
        latency: 0,
      };
      checks.status = 'degraded';

      if (debugMode) {
        console.error('[Health] Redis check failed:', error);
      }
    }
  }

  // If both services are unhealthy, mark as unhealthy
  if (
    checks.services.database.status === 'unhealthy' &&
    checks.services.redis.status === 'unhealthy'
  ) {
    checks.status = 'unhealthy';
  }

  return new Response(JSON.stringify(checks, null, 2), {
    status: checks.status === 'unhealthy' ? 503 : 200,
    headers: {
      'Content-Type': 'application/json',
      'Cache-Control': 'no-store',
    },
  });
};
