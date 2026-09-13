import type { APIRoute } from 'astro';

/**
 * Health endpoint — the one app file this recipe owns.
 *
 * Deliberately dependency-free. The framework scaffolder (`create-astro`) owns
 * `package.json`, `tsconfig.json` and the app payload; this recipe owns the
 * infra plus this endpoint, so it must run on a *fresh* scaffold with nothing
 * installed but `astro` itself. It used to import `@/lib/db` and `@/lib/redis`
 * from a payload the recipe never actually installed, under a path alias the
 * scaffolded `tsconfig.json` never defines — so `/api/health` answered 500 on
 * every new service and the container healthcheck never passed
 * (bd:mech-crate-874).
 *
 * No `prerender` export on purpose: with Astro's default static output the route
 * is prerendered to `dist/api/health`, and with an adapter (`astro add node`) it
 * is served on demand. Both shapes answer 200, so the healthcheck in
 * `docker/compose/<service>.yml` holds either way.
 *
 * GET /api/health
 */
export const GET: APIRoute = () =>
  new Response(
    JSON.stringify({
      status: 'ok',
      service: '{{SERVICE_NAME}}',
      timestamp: new Date().toISOString(),
    }),
    {
      status: 200,
      headers: {
        'content-type': 'application/json',
        'cache-control': 'no-store',
      },
    },
  );
