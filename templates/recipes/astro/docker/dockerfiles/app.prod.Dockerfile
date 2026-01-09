# ========================================
# {{SERVICE_NAME}} - Production Dockerfile
# Optimized for Cloudflare Container deployments
# ========================================

ARG NODE_VERSION=22

# ========================================
# Base stage
# ========================================
FROM node:${NODE_VERSION}-alpine AS base
WORKDIR /app

RUN apk add --no-cache libc6-compat curl && \
    corepack enable

# ========================================
# Dependencies stage
# ========================================
FROM base AS deps

COPY package.json package-lock.json* pnpm-lock.yaml* yarn.lock* ./

RUN \
    if [ -f pnpm-lock.yaml ]; then pnpm install --frozen-lockfile --prod; \
    elif [ -f yarn.lock ]; then yarn install --frozen-lockfile --production; \
    elif [ -f package-lock.json ]; then npm ci --omit=dev; \
    else npm install --omit=dev; \
    fi

# ========================================
# Build dependencies (includes devDependencies)
# ========================================
FROM base AS build-deps

COPY package.json package-lock.json* pnpm-lock.yaml* yarn.lock* ./

RUN \
    if [ -f pnpm-lock.yaml ]; then pnpm install --frozen-lockfile; \
    elif [ -f yarn.lock ]; then yarn install --frozen-lockfile; \
    elif [ -f package-lock.json ]; then npm ci; \
    else npm install; \
    fi

# ========================================
# Builder stage
# ========================================
FROM base AS builder

COPY --from=build-deps /app/node_modules ./node_modules
COPY . .

ENV NODE_ENV=production

# Type check and build
RUN npm run build

# ========================================
# Production stage - Ultra minimal
# ========================================
FROM node:${NODE_VERSION}-alpine AS runner

WORKDIR /app

ENV NODE_ENV=production
ENV HOST=0.0.0.0
ENV PORT=4321

# Security: non-root user
RUN addgroup --system --gid 1001 nodejs && \
    adduser --system --uid 1001 astro

# Copy only what's needed
COPY --from=deps --chown=astro:nodejs /app/node_modules ./node_modules
COPY --from=builder --chown=astro:nodejs /app/dist ./dist
COPY --from=builder --chown=astro:nodejs /app/package.json ./

USER astro

EXPOSE 4321

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:4321/api/health || exit 1

CMD ["node", "./dist/server/entry.mjs"]
