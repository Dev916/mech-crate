# {{SERVICE_NAME}} — Astro

An Astro app scaffolded by `create-astro`, wired into the mx stack.

## Who owns what

The framework scaffolder owns your app: `package.json`, `tsconfig.json`,
`astro.config.mjs`, the starter pages, and every dependency you add. This recipe
owns the infrastructure around it — Docker builds, compose files, env templates,
Traefik routing — plus exactly one app file: `src/pages/api/health.ts`, a
dependency-free health endpoint the container healthcheck probes.

That means this app starts minimal on purpose. Add what you need the normal way:

```bash
npx astro add vue tailwind        # frameworks and integrations
npm install drizzle-orm ioredis   # whatever the app actually uses
```

## Getting Started

```bash
# With Docker (recommended — brings up Postgres and Redis too)
make dev s={{SERVICE_NAME}}

# Or directly
npm install
npm run dev
```

Your service is routed at `http://{{DOMAIN}}` once the stack is up. Check
`http://{{DOMAIN}}/api/health`.

## Docker

```bash
# Development (hot reload, source bind-mounted)
make dev s={{SERVICE_NAME}}

# Production-shaped image
make build s={{SERVICE_NAME}}
```

The Dockerfile's `production` stage serves whatever `astro build` produced. With
the default scaffold that is a static `dist/`, served by the `preview` script.
Run `npx astro add node` and the same stage execs `dist/server/entry.mjs`
instead — no Dockerfile change needed.

## Re-scaffolding

`mx add` never overwrites an app that already has files in it. To start the app
over from a clean scaffold (this **deletes** `apps/{{SERVICE_NAME}}`):

```bash
mx add {{SERVICE_NAME}} --recipe astro --opt force_init=true
```

## Environment Variables

Astro uses Vite's environment variable handling, with one key distinction:

- **`PUBLIC_` prefix** — available on both server AND client
- **No prefix** — server-side only (for security)

### Server-Only Variables (never exposed to client)

```env
NODE_ENV=development
PORT=4321
DATABASE_URL=postgres://user:pass@db:5432/db
REDIS_URL=redis://redis:6379
SESSION_SECRET=your-secret-key
```

### Public Variables (accessible everywhere)

```env
PUBLIC_APP_NAME={{SERVICE_NAME}}
PUBLIC_APP_URL=http://{{DOMAIN}}
PUBLIC_API_BASE_URL=http://{{DOMAIN}}/api
```

Values come from `docker/.config/.env.shared`, `.env.secrets` and
`.env.{{SERVICE_NAME}}`, layered in that order — last one wins.

### Usage

**In Astro frontmatter (server-side):**

```astro
---
const dbUrl = import.meta.env.DATABASE_URL;     // server-only, safe here
const appName = import.meta.env.PUBLIC_APP_NAME;
---
```

**In API routes (server-side):**

```typescript
// src/pages/api/data.ts
export const GET: APIRoute = async () => {
  const dbUrl = import.meta.env.DATABASE_URL;
  // ...
};
```

**In client components:**

```ts
const appName = import.meta.env.PUBLIC_APP_NAME;   // ✅ PUBLIC_ vars reach the client
const dbUrl = import.meta.env.DATABASE_URL;        // ❌ undefined here, as intended
```

## Output Mode

The default scaffold has no adapter, so Astro prerenders everything at build time
— including `src/pages/api/health.ts`, which lands as `dist/api/health`. Add
`export const prerender = false` to a route only *after* configuring an adapter
(`npx astro add node`), or the build will fail.
