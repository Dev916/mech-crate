# ========================================
# {{SERVICE_NAME}} - Astro SSR Application
# Multi-stage build for development and production
# ========================================

ARG NODE_VERSION=22

# ========================================
# Base stage - Common dependencies
# ========================================
FROM node:${NODE_VERSION}-alpine AS base
WORKDIR /app

# Install system dependencies
RUN apk add --no-cache \
    libc6-compat \
    curl

# Enable corepack for pnpm/yarn support
RUN corepack enable

# ========================================
# Dependencies stage
# ========================================
FROM base AS deps

COPY package.json package-lock.json* pnpm-lock.yaml* yarn.lock* ./

# Install dependencies based on lockfile
RUN \
    if [ -f pnpm-lock.yaml ]; then pnpm install --frozen-lockfile; \
    elif [ -f yarn.lock ]; then yarn install --frozen-lockfile; \
    elif [ -f package-lock.json ]; then npm ci; \
    else npm install; \
    fi

# ========================================
# Builder stage - Build the application
# ========================================
FROM base AS builder

COPY --from=deps /app/node_modules ./node_modules
COPY . .

# Set build-time environment variables
ENV NODE_ENV=production

# Build the Astro application
RUN npm run build

# ========================================
# Production stage - Minimal runtime
# ========================================
FROM base AS production

ENV NODE_ENV=production
ENV HOST=0.0.0.0
ENV PORT=4321

# Create non-root user for security
RUN addgroup --system --gid 1001 nodejs && \
    adduser --system --uid 1001 astro

# Copy built application
COPY --from=builder --chown=astro:nodejs /app/dist ./dist
COPY --from=builder --chown=astro:nodejs /app/node_modules ./node_modules
COPY --from=builder --chown=astro:nodejs /app/package.json ./

# The static-output entry below runs Astro's own server, which writes a lock file
# under .astro on startup; /app itself is root-owned, so without this the
# non-root user dies with EACCES before serving a single request.
RUN mkdir -p /app/.astro && chown astro:nodejs /app/.astro

USER astro

EXPOSE 4321

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:4321/api/health || exit 1

# Serve whichever shape the build produced.
#
# `create-astro --template minimal` — the scaffolder this recipe runs — ships no
# adapter, so `astro build` emits a STATIC `dist/` and *no* `dist/server/entry.mjs`.
# This stage used to exec that entry unconditionally, which crash-looped on every
# freshly scaffolded app. Adding the adapter is the app's call (`astro add node`),
# not the recipe's — the scaffolder owns `astro.config.mjs` — so run the SSR entry
# when the app built one and otherwise serve the static build with the `preview`
# script the scaffold already declares. Both paths use only what is already in the
# image, and both answer `/api/health`: static output prerenders it to
# `dist/api/health` (bd:mech-crate-47j).
CMD ["sh", "-c", "if [ -f ./dist/server/entry.mjs ]; then exec node ./dist/server/entry.mjs; else exec npm run preview -- --host 0.0.0.0 --port ${PORT:-4321}; fi"]

# ========================================
# Development stage - Hot reload
# ========================================
FROM base AS development

ENV NODE_ENV=development

COPY --from=deps /app/node_modules ./node_modules
COPY . .

EXPOSE 4321
EXPOSE 24678

# `--host 0.0.0.0` is not optional: create-astro's `dev` script is a bare
# `astro dev`, which binds localhost *inside the container*. The healthcheck
# (curl localhost from within) still passed, so the service looked healthy while
# Traefik got a 502 on every request — the same shape as the nuxt dev stage, which
# already passes the flag through.
CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]
