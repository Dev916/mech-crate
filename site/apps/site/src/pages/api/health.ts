import type { APIRoute } from 'astro';

/**
 * Health check endpoint.
 *
 * The mx astro recipe's compose healthcheck curls `/api/health`
 * (docker/compose/site.dev.yml). This site is a static documentation build with
 * no database or cache, so the endpoint is prerendered and dependency-free —
 * unlike the recipe's default health route, which probes Postgres and Redis.
 *
 * GET /api/health
 */
export const prerender = true;

export const GET: APIRoute = () =>
  new Response(
    JSON.stringify(
      {
        status: 'healthy',
        app: 'site',
      },
      null,
      2,
    ),
    {
      status: 200,
      headers: {
        'Content-Type': 'application/json',
        'Cache-Control': 'no-store',
      },
    },
  );
