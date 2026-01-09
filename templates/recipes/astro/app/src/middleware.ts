import { defineMiddleware, sequence } from 'astro:middleware';

/**
 * Security headers middleware
 */
const securityHeaders = defineMiddleware(async (context, next) => {
  const response = await next();
  
  // Security headers
  response.headers.set('X-Content-Type-Options', 'nosniff');
  response.headers.set('X-Frame-Options', 'DENY');
  response.headers.set('X-XSS-Protection', '1; mode=block');
  response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
  
  // Cache control for static assets
  if (context.url.pathname.startsWith('/_astro/')) {
    response.headers.set('Cache-Control', 'public, max-age=31536000, immutable');
  }
  
  return response;
});

/**
 * Request logging middleware (development only)
 */
const requestLogger = defineMiddleware(async (context, next) => {
  const start = performance.now();
  const response = await next();
  const duration = Math.round(performance.now() - start);
  
  if (import.meta.env.DEV) {
    console.log(
      `[${context.request.method}] ${context.url.pathname} - ${response.status} (${duration}ms)`
    );
  }
  
  return response;
});

/**
 * API error handler middleware
 */
const apiErrorHandler = defineMiddleware(async (context, next) => {
  try {
    return await next();
  } catch (error) {
    // Only handle API routes
    if (context.url.pathname.startsWith('/api/')) {
      console.error('[API Error]', error);
      
      return new Response(
        JSON.stringify({
          success: false,
          error: {
            code: 'INTERNAL_ERROR',
            message: error instanceof Error ? error.message : 'An unexpected error occurred',
          },
        }),
        {
          status: 500,
          headers: { 'Content-Type': 'application/json' },
        }
      );
    }
    
    // Re-throw for page routes to show error page
    throw error;
  }
});

// Chain middlewares
export const onRequest = sequence(requestLogger, securityHeaders, apiErrorHandler);
