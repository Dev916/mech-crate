/**
 * Health endpoint: the one app file this recipe owns.
 *
 * Deliberately dependency-free. The framework scaffolder (`nuxi init`) owns
 * `package.json`, `nuxt.config.ts` and the app tree; this recipe owns the infra
 * plus this endpoint, so it must run on a fresh scaffold with nothing installed
 * but `nuxt` itself (bd:mech-crate-whj, the bd:mech-crate-874 call).
 * `defineEventHandler` is a Nitro auto-import, so there is nothing to import.
 *
 * Lives at the project root `server/`, which Nuxt 4 still reads even though the
 * scaffolded app source sits under `app/`.
 *
 * GET /api/health
 */
export default defineEventHandler(() => {
  return {
    status: 'ok',
    service: '{{SERVICE_NAME}}',
    timestamp: new Date().toISOString(),
    uptime: process.uptime(),
  }
})
