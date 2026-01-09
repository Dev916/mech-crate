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
FROM base AS runner

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

USER astro

EXPOSE 4321

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:4321/api/health || exit 1

CMD ["node", "./dist/server/entry.mjs"]

# ========================================
# Development stage - Hot reload
# ========================================
FROM base AS development

ENV NODE_ENV=development

COPY --from=deps /app/node_modules ./node_modules
COPY . .

EXPOSE 4321
EXPOSE 24678

CMD ["npm", "run", "dev"]
