# {{SERVICE_NAME}} - Astro 5 + Vue 3 SSR

Full-stack Astro 5 application with Vue 3 islands, SSR, and Apple-inspired design.

## Stack

- **Astro 5** - Islands architecture, SSR with Node adapter
- **Vue 3** - Composition API, `<script setup>`
- **shadcn-vue** - UI primitives (via Radix Vue)
- **PrimeVue 4** - Advanced UI components
- **Tailwind CSS** - Utility-first styling
- **Pinia** - Global state management
- **Drizzle ORM** - Type-safe database queries
- **PostgreSQL** - Primary database
- **Redis** - Caching and sessions
- **Docker** - Containerized deployment

## Getting Started

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Start production server
npm run start
```

## Docker

```bash
# Development
docker compose -f docker/compose/{{SERVICE_NAME}}.yml -f docker/compose/{{SERVICE_NAME}}.dev.yml up

# Production
docker compose -f docker/compose/{{SERVICE_NAME}}.yml up -d
```

## Project Structure

```
src/
├── components/      # Vue components
│   ├── ui/          # shadcn-vue primitives
│   └── layout/      # Layout components
├── layouts/         # Astro layouts
├── pages/           # Astro pages & API routes
│   └── api/         # API endpoints
├── stores/          # Pinia stores
├── lib/             # Utilities
│   ├── db/          # Database (Drizzle)
│   └── redis/       # Redis client
├── styles/          # Global CSS
└── types/           # TypeScript types
```

## Features

- **SSR by default** - Fast initial loads, SEO-friendly
- **Vue islands** - Interactive components hydrate on demand
- **Type-safe everywhere** - TypeScript strict mode enabled
- **Global state** - Pinia store with theme, language, notifications
- **Database ready** - PostgreSQL with Drizzle ORM migrations
- **Caching** - Redis for sessions and data caching
- **Apple design** - Carefully crafted typography and spacing
- **Cloudflare ready** - Multi-stage Docker builds

## Environment Variables

Astro uses Vite's environment variable handling with a key distinction:

- **`PUBLIC_` prefix** - Available on both server AND client
- **No prefix** - Server-side only (for security)

### Server-Only Variables (never exposed to client)

```env
# These are ONLY accessible in Astro frontmatter and API routes
NODE_ENV=development
PORT=4321
DATABASE_URL=postgres://user:pass@localhost:5432/db
REDIS_URL=redis://localhost:6379
SESSION_SECRET=your-secret-key
API_SECRET_KEY=your-api-key
```

### Public Variables (accessible everywhere)

```env
# These are accessible in both server code AND client-side Vue components
PUBLIC_APP_NAME={{SERVICE_NAME}}
PUBLIC_APP_URL=http://localhost:4321
PUBLIC_API_BASE_URL=http://localhost:4321/api
PUBLIC_ENABLE_ANALYTICS=false
PUBLIC_ENABLE_DEBUG_MODE=true
```

### Usage Examples

**In Astro frontmatter (server-side):**

```astro
---
// Server-side only vars (safe to use)
const dbUrl = import.meta.env.DATABASE_URL;
const secret = import.meta.env.SESSION_SECRET;

// Public vars also work here
const appName = import.meta.env.PUBLIC_APP_NAME;
---
```

**In API routes (server-side):**

```typescript
// src/pages/api/data.ts
export const GET: APIRoute = async () => {
  // All env vars are accessible here
  const dbUrl = import.meta.env.DATABASE_URL;
  const apiKey = import.meta.env.API_SECRET_KEY;
  // ...
};
```

**In Vue components (client-side):**

```vue
<script setup lang="ts">
// ✅ PUBLIC_ vars work in Vue components
const appName = import.meta.env.PUBLIC_APP_NAME;
const apiUrl = import.meta.env.PUBLIC_API_BASE_URL;

// ❌ Server-only vars are undefined here (as expected!)
const dbUrl = import.meta.env.DATABASE_URL; // undefined
</script>
```

### Type Safety

Environment variables are typed in `src/env.d.ts`:

```typescript
interface ImportMetaEnv {
  // Server-only
  readonly DATABASE_URL: string;
  readonly SESSION_SECRET: string;

  // Public (client + server)
  readonly PUBLIC_APP_NAME: string;
  readonly PUBLIC_API_BASE_URL: string;
}
```

## Vue Islands (Hydration Directives)

Use client directives to control when Vue components hydrate:

```astro
<!-- Load immediately -->
<Hero client:load />

<!-- Load when visible -->
<Features client:visible />

<!-- Load when browser is idle -->
<Analytics client:idle />

<!-- Load based on media query -->
<MobileMenu client:media="(max-width: 768px)" />
```
