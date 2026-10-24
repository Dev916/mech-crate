# ─────────────────────────────────────────────────────────────────────────────
# {{SERVICE_NAME}} - Zola Static Site
# Multi-stage build for development and production
# ─────────────────────────────────────────────────────────────────────────────

ARG ZOLA_VERSION={{ZOLA_VERSION}}

# ─────────────────────────────────────────────────────────────────────────────
# Zola Stage - glibc base carrying the zola binary for the build platform
# ─────────────────────────────────────────────────────────────────────────────
# Zola publishes its Linux binaries as glibc (`*-unknown-linux-gnu`) builds:
# there is no aarch64 musl tarball, and no aarch64 Linux build at all before
# v0.20.0. A glibc binary does not run on Alpine (musl), so this base is Debian,
# and the triple follows the build platform so Apple Silicon and x86_64 hosts
# both fetch a tarball that exists (bd:mech-crate-b15).
FROM debian:bookworm-slim AS zola

ARG ZOLA_VERSION

RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Download and install Zola binary. `curl -f` turns a missing asset into a
# failed step, and `zola --version` proves the binary runs on this base.
RUN ARCH=$(uname -m) && \
    case "$ARCH" in \
        x86_64|amd64) ZOLA_ARCH="x86_64-unknown-linux-gnu" ;; \
        aarch64|arm64) ZOLA_ARCH="aarch64-unknown-linux-gnu" ;; \
        *) echo "Unsupported architecture: $ARCH" && exit 1 ;; \
    esac && \
    curl -fsSL "https://github.com/getzola/zola/releases/download/v${ZOLA_VERSION}/zola-v${ZOLA_VERSION}-${ZOLA_ARCH}.tar.gz" | \
    tar xz -C /usr/local/bin && \
    zola --version

# ─────────────────────────────────────────────────────────────────────────────
# Builder Stage - Build the site
# ─────────────────────────────────────────────────────────────────────────────
FROM zola AS builder

WORKDIR /site

# Copy the site source. The build context is the project root, so the site is
# apps/{{SERVICE_NAME}}, not the context itself.
COPY apps/{{SERVICE_NAME}}/ .

# Build the static site
RUN zola build

# ─────────────────────────────────────────────────────────────────────────────
# Development Stage - Live reload with zola serve
# ─────────────────────────────────────────────────────────────────────────────
FROM zola AS development

WORKDIR /site

# wget backs the compose healthcheck; inotify-tools is for file watching.
RUN apt-get update && \
    apt-get install -y --no-install-recommends wget inotify-tools && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd --system zola && useradd --system --gid zola zola
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
