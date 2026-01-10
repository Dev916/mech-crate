# ─────────────────────────────────────────────────────────────────────────────
# {{SERVICE_NAME}} - Zola Static Site
# Multi-stage build for development and production
# ─────────────────────────────────────────────────────────────────────────────

ARG ZOLA_VERSION={{ZOLA_VERSION}}

# ─────────────────────────────────────────────────────────────────────────────
# Builder Stage - Download Zola binary and build the site
# ─────────────────────────────────────────────────────────────────────────────
FROM alpine:3.19 AS builder

ARG ZOLA_VERSION

WORKDIR /site

# Install dependencies for downloading and extracting
RUN apk add --no-cache \
    curl \
    tar \
    gzip \
    ca-certificates

# Download and install Zola binary
RUN ARCH=$(uname -m) && \
    case "$ARCH" in \
        x86_64) ZOLA_ARCH="x86_64-unknown-linux-gnu" ;; \
        aarch64) ZOLA_ARCH="aarch64-unknown-linux-gnu" ;; \
        *) echo "Unsupported architecture: $ARCH" && exit 1 ;; \
    esac && \
    curl -sL "https://github.com/getzola/zola/releases/download/v${ZOLA_VERSION}/zola-v${ZOLA_VERSION}-${ZOLA_ARCH}.tar.gz" | \
    tar xz -C /usr/local/bin

# Copy source files
COPY . .

# Build the static site
RUN zola build

# ─────────────────────────────────────────────────────────────────────────────
# Development Stage - Live reload with zola serve
# ─────────────────────────────────────────────────────────────────────────────
FROM alpine:3.19 AS development

ARG ZOLA_VERSION

WORKDIR /site

# Install dependencies
RUN apk add --no-cache \
    curl \
    tar \
    gzip \
    ca-certificates \
    inotify-tools

# Download and install Zola binary
RUN ARCH=$(uname -m) && \
    case "$ARCH" in \
        x86_64) ZOLA_ARCH="x86_64-unknown-linux-gnu" ;; \
        aarch64) ZOLA_ARCH="aarch64-unknown-linux-gnu" ;; \
        *) echo "Unsupported architecture: $ARCH" && exit 1 ;; \
    esac && \
    curl -sL "https://github.com/getzola/zola/releases/download/v${ZOLA_VERSION}/zola-v${ZOLA_VERSION}-${ZOLA_ARCH}.tar.gz" | \
    tar xz -C /usr/local/bin

# Create non-root user
RUN addgroup -S zola && adduser -S zola -G zola
RUN chown -R zola:zola /site

USER zola

EXPOSE 1111
EXPOSE 1024

# Start Zola development server with live reload
# - 0.0.0.0 binds to all interfaces (needed for Docker)
# - Port 1111 for HTTP, 1024 for live reload WebSocket
CMD ["zola", "serve", "--interface", "0.0.0.0", "--port", "1111", "--base-url", "localhost"]

# ─────────────────────────────────────────────────────────────────────────────
# Production Stage - Nginx serving static files
# ─────────────────────────────────────────────────────────────────────────────
FROM nginx:1.25-alpine AS production

# Remove default nginx configuration
RUN rm -rf /etc/nginx/conf.d/*

# Copy custom nginx configuration
COPY docker/system/{{SERVICE_NAME}}/etc/nginx/http.d/app.conf /etc/nginx/conf.d/default.conf

# Copy built static files from builder
COPY --from=builder /site/public /usr/share/nginx/html

# Create cache directories
RUN mkdir -p /var/cache/nginx && \
    chown -R nginx:nginx /var/cache/nginx

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD wget -q --spider http://localhost/up || exit 1

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
