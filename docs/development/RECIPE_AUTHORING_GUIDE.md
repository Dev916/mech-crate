# Recipe Authoring Guide

A comprehensive guide for creating MechCrate recipes—reusable application templates with Docker orchestration, process management, and reverse proxying.

## Table of Contents

1. [Recipe Structure Overview](#recipe-structure-overview)
2. [The recipe.json Specification](#the-recipejson-specification)
3. [Docker Structure and Templating](#docker-structure-and-templating)
4. [System Files and Filesystem Mirroring](#system-files-and-filesystem-mirroring)
5. [Supervisor Configuration](#supervisor-configuration)
6. [Entrypoint Scripts](#entrypoint-scripts)
7. [Internal Reverse Proxy](#internal-reverse-proxy)
8. [Environment Variables](#environment-variables)
9. [Compose Files](#compose-files)
10. [Best Practices](#best-practices)

---

## Recipe Structure Overview

A recipe is a self-contained template that generates a complete service with application code, Docker infrastructure, and orchestration. Each recipe follows this structure:

```
templates/recipes/[recipe-name]/
├── app/                    # Application source code
│   ├── src/
│   ├── config/
│   └── ...
├── config/
│   └── env.service         # Environment template for Docker
├── docker/
│   ├── compose/
│   │   ├── service.yml     # Production compose
│   │   └── service.dev.yml # Development overrides
│   ├── dockerfiles/
│   │   ├── app.Dockerfile      # Multi-stage Dockerfile
│   │   └── app.prod.Dockerfile # Production-only build (optional)
│   └── system/
│       └── app/            # Filesystem mirror for container
│           ├── etc/
│           │   ├── supervisor/
│           │   └── nginx/ (or haproxy/)
│           └── usr/
│               └── local/
│                   └── bin/
│                       └── entrypoint
├── README.md               # Service documentation
└── recipe.json             # Recipe manifest
```

### Key Concepts

1. **Application Code** (`app/`): The actual service source code
2. **Docker Infrastructure** (`docker/`): All container-related files
3. **System Files** (`docker/system/app/`): Files mirrored into container filesystem
4. **Recipe Manifest** (`recipe.json`): Defines placeholders, templates, and install steps

---

## The recipe.json Specification

The `recipe.json` file is the heart of a recipe—it defines how templates are processed and what the recipe produces.

### Complete Schema

```json
{
  "name": "recipe-name",
  "title": "Human-Readable Title",
  "description": "Brief description of what this recipe provides",
  "version": "1.0",

  "features": [
    "Feature 1 (displayed to user)",
    "Feature 2",
    "Feature 3"
  ],

  "services": [
    { "name": "<name>", "description": "Main application" },
    { "name": "<name>-worker", "description": "Background worker" },
    { "name": "db", "description": "PostgreSQL (shared)" },
    { "name": "redis", "description": "Redis (shared)" }
  ],

  "options": {
    "domain": {
      "flag": "--domain",
      "default": "{{SERVICE_NAME}}.localhost",
      "description": "Custom domain for routing"
    },
    "port": {
      "flag": "--port",
      "default": "3000",
      "description": "Application port"
    }
  },

  "placeholders": {
    "SERVICE_NAME": { "source": "name" },
    "SERVICE_SLUG": { "source": "name", "transform": "slug" },
    "SERVICE_UPPER": { "source": "name", "transform": "upper" },
    "DOMAIN": { "source": "option:domain" },
    "PORT": { "source": "option:port" }
  },

  "directories": [
    "apps/{{SERVICE_NAME}}/src",
    "docker/compose",
    "docker/dockerfiles/{{SERVICE_NAME}}",
    "docker/config",
    "docker/system/{{SERVICE_NAME}}"
  ],

  "templates": [
    { "from": "app", "to": "apps/{{SERVICE_NAME}}" },
    { "from": "docker/compose/service.yml", "to": "docker/compose/{{SERVICE_NAME}}.yml" },
    { "from": "docker/compose/service.dev.yml", "to": "docker/compose/{{SERVICE_NAME}}.dev.yml" },
    { "from": "docker/dockerfiles/app.Dockerfile", "to": "docker/dockerfiles/{{SERVICE_NAME}}/app" },
    { "from": "docker/system/app", "to": "docker/system/{{SERVICE_NAME}}" },
    { "from": "config/env.service", "to": "docker/config/.env.{{SERVICE_NAME}}" }
  ],

  "post_install": {
    "renames": [
      { "from": "apps/{{SERVICE_NAME}}/gitignore.template", "to": "apps/{{SERVICE_NAME}}/.gitignore" }
    ],
    "chmod": [
      { "path": "docker/system/{{SERVICE_NAME}}/usr/local/bin/entrypoint", "mode": "+x" }
    ],
    "gitkeep": [
      "apps/{{SERVICE_NAME}}/storage/logs"
    ]
  },

  "next_steps": [
    "cd apps/{{SERVICE_NAME}}",
    "npm install",
    "make dev s={{SERVICE_NAME}}"
  ],

  "notes": [
    "App: http://{{DOMAIN}}",
    "Health: http://{{DOMAIN}}/api/health"
  ]
}
```

### Placeholder Transforms

| Transform | Description | Example |
|-----------|-------------|---------|
| `slug` | Lowercase with hyphens | `my-service` |
| `upper` | UPPERCASE with underscores | `MY_SERVICE` |
| `lower` | lowercase | `myservice` |
| `camel` | camelCase | `myService` |
| `pascal` | PascalCase | `MyService` |

### Important Rules

1. **Placeholders in file content only**: Use `{{PLACEHOLDER}}` in file contents
2. **Placeholders in paths are expanded**: Template paths like `docker/system/{{SERVICE_NAME}}` work
3. **No placeholders in actual directory names**: The `from` path in templates uses literal names (e.g., `docker/system/app`)

---

## Docker Structure and Templating

### Dockerfile Patterns

Use multi-stage builds to support both development and production:

```dockerfile
# ─────────────────────────────────────────────────────────────────────────────
# Base Stage
# ─────────────────────────────────────────────────────────────────────────────
FROM node:20-alpine AS base

WORKDIR /app

# Install system dependencies
RUN apk add --no-cache \
    curl \
    supervisor \
    nginx

# ─────────────────────────────────────────────────────────────────────────────
# Dependencies Stage
# ─────────────────────────────────────────────────────────────────────────────
FROM base AS deps

COPY package*.json ./
RUN npm ci

# ─────────────────────────────────────────────────────────────────────────────
# Development Stage
# ─────────────────────────────────────────────────────────────────────────────
FROM base AS development

# Copy dependencies
COPY --from=deps /app/node_modules ./node_modules

# Copy system files (filesystem mirroring pattern)
COPY docker/system/{{SERVICE_NAME}}/ /
RUN chmod +x /usr/local/bin/entrypoint

# Development uses bind mounts for source code
EXPOSE 80 3000 5173

ENTRYPOINT ["/usr/local/bin/entrypoint"]

# ─────────────────────────────────────────────────────────────────────────────
# Builder Stage
# ─────────────────────────────────────────────────────────────────────────────
FROM base AS builder

COPY --from=deps /app/node_modules ./node_modules
COPY apps/{{SERVICE_NAME}} ./

RUN npm run build

# ─────────────────────────────────────────────────────────────────────────────
# Production Stage
# ─────────────────────────────────────────────────────────────────────────────
FROM base AS production

# Copy system files (filesystem mirroring pattern)
COPY docker/system/{{SERVICE_NAME}}/ /
RUN chmod +x /usr/local/bin/entrypoint

# Copy built application
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./

EXPOSE 80

ENTRYPOINT ["/usr/local/bin/entrypoint"]
```

### Build Context

The Dockerfile is built from the **project root**, not the Docker folder:

```yaml
# docker/compose/service.yml
services:
  myapp:
    build:
      context: ../..              # Project root
      dockerfile: docker/dockerfiles/{{SERVICE_NAME}}/app
      target: production
```

This allows the Dockerfile to reference:
- `docker/system/{{SERVICE_NAME}}/` - System files
- `apps/{{SERVICE_NAME}}/` - Application source

---

## System Files and Filesystem Mirroring

The `docker/system/[service]/` directory mirrors the container filesystem. A single `COPY` command overlays all files:

```dockerfile
# Copy ALL system files in one command
COPY docker/system/{{SERVICE_NAME}}/ /
```

### Directory Structure

```
docker/system/app/
├── etc/
│   ├── supervisor/
│   │   ├── supervisord.conf        # Main supervisor config
│   │   └── conf.d/
│   │       ├── stack.dev.conf      # Development processes
│   │       ├── stack.prod.conf     # Production processes
│   │       └── queue.conf          # Queue worker processes
│   └── nginx/
│       ├── nginx.conf              # Main nginx config
│       └── http.d/
│           └── app.conf            # Application server block
└── usr/
    └── local/
        └── bin/
            ├── entrypoint          # Container entrypoint
            └── scheduler-runner    # Optional helper scripts
```

### How It Works

When the container starts:
- `docker/system/app/etc/supervisor/supervisord.conf` → `/etc/supervisor/supervisord.conf`
- `docker/system/app/etc/nginx/nginx.conf` → `/etc/nginx/nginx.conf`
- `docker/system/app/usr/local/bin/entrypoint` → `/usr/local/bin/entrypoint`

This pattern:
1. **Simplifies Dockerfile**: One `COPY` instead of many
2. **Makes structure obvious**: Container layout visible in source
3. **Enables version control**: All configs tracked in git

---

## Supervisor Configuration

Supervisor manages multiple processes within a single container. Use separate config files for different modes.

### Main Config (`supervisord.conf`)

```ini
[supervisord]
nodaemon=true
user=root
logfile=/var/log/supervisor/supervisord.log
pidfile=/var/run/supervisord.pid
childlogdir=/var/log/supervisor
logfile_maxbytes=50MB
logfile_backups=10
loglevel=info

[unix_http_server]
file=/var/run/supervisor.sock
chmod=0700

[rpcinterface:supervisor]
supervisor.rpcinterface_factory = supervisor.rpcinterface:make_main_rpcinterface

[supervisorctl]
serverurl=unix:///var/run/supervisor.sock

[include]
files = /etc/supervisor/conf.d/*.conf
```

### Development Stack (`stack.dev.conf`)

```ini
[program:app]
command=npm run dev -- --host 0.0.0.0 --port 3000
directory=/app
user=node
autostart=true
autorestart=true
startsecs=5
startretries=3
stdout_logfile=/dev/stdout
stdout_logfile_maxbytes=0
stderr_logfile=/dev/stderr
stderr_logfile_maxbytes=0
environment=HOME="/app",NODE_ENV="development"

[program:nginx]
command=nginx -g "daemon off;"
autostart=true
autorestart=true
startsecs=2
startretries=3
stdout_logfile=/dev/stdout
stdout_logfile_maxbytes=0
stderr_logfile=/dev/stderr
stderr_logfile_maxbytes=0
```

### Production Stack (`stack.prod.conf`)

```ini
[program:app]
command=node dist/server.js
directory=/app
user=node
autostart=true
autorestart=true
startsecs=5
startretries=3
stopwaitsecs=30
stdout_logfile=/dev/stdout
stdout_logfile_maxbytes=0
stderr_logfile=/dev/stderr
stderr_logfile_maxbytes=0
environment=HOME="/app",NODE_ENV="production"

[program:nginx]
command=nginx -g "daemon off;"
autostart=true
autorestart=true
startsecs=2
startretries=3
stdout_logfile=/dev/stdout
stdout_logfile_maxbytes=0
stderr_logfile=/dev/stderr
stderr_logfile_maxbytes=0
```

### Queue Workers (`queue.conf`)

```ini
[program:worker]
process_name=%(program_name)s_%(process_num)02d
command=node dist/worker.js
directory=/app
user=node
autostart=true
autorestart=true
startsecs=5
startretries=3
numprocs=%(ENV_WORKER_COUNT)s
stdout_logfile=/dev/stdout
stdout_logfile_maxbytes=0
stderr_logfile=/dev/stderr
stderr_logfile_maxbytes=0
environment=HOME="/app"

[program:scheduler]
command=/usr/local/bin/scheduler-runner
directory=/app
user=node
autostart=%(ENV_RUN_SCHEDULER)s
autorestart=true
startsecs=5
startretries=3
stdout_logfile=/dev/stdout
stdout_logfile_maxbytes=0
stderr_logfile=/dev/stderr
stderr_logfile_maxbytes=0
environment=HOME="/app"
```

### Key Supervisor Features

1. **Environment Variable Interpolation**: Use `%(ENV_VAR_NAME)s`
2. **Process Numbering**: Use `%(process_num)02d` for multiple workers
3. **Log Streaming**: Use `/dev/stdout` and `maxbytes=0` for Docker logs
4. **Conditional Autostart**: Use `%(ENV_ENABLED)s` with `true`/`false`

---

## Entrypoint Scripts

The entrypoint script handles container initialization, mode switching, and supervisor startup.

### Complete Entrypoint Template

```bash
#!/bin/sh
set -e

# ─────────────────────────────────────────────────────────────────────────────
# [Service Name] Entrypoint
# ─────────────────────────────────────────────────────────────────────────────

APP_MODE="${APP_MODE:-app}"
APP_ENV="${APP_ENV:-production}"

# ─────────────────────────────────────────────────────────────────────────────
# Environment Defaults
# ─────────────────────────────────────────────────────────────────────────────

export WORKER_COUNT="${WORKER_COUNT:-2}"
export RUN_SCHEDULER="${RUN_SCHEDULER:-false}"

# ─────────────────────────────────────────────────────────────────────────────
# Setup Directories and Permissions
# ─────────────────────────────────────────────────────────────────────────────

mkdir -p /var/log/supervisor
mkdir -p /var/log/nginx
mkdir -p /app/storage/logs

chown -R node:node /app/storage 2>/dev/null || true
chmod -R 775 /app/storage 2>/dev/null || true

# ─────────────────────────────────────────────────────────────────────────────
# Select Supervisor Config Based on Mode
# ─────────────────────────────────────────────────────────────────────────────

SUPERVISOR_CONF="/etc/supervisor/conf.d"

case "$APP_MODE" in
    app)
        # Main application mode
        if [ "$APP_ENV" = "local" ] || [ "$APP_ENV" = "development" ]; then
            echo "Starting in DEVELOPMENT mode..."
            rm -f "$SUPERVISOR_CONF/stack.prod.conf" "$SUPERVISOR_CONF/queue.conf" 2>/dev/null || true
        else
            echo "Starting in PRODUCTION mode..."
            rm -f "$SUPERVISOR_CONF/stack.dev.conf" "$SUPERVISOR_CONF/queue.conf" 2>/dev/null || true
        fi
        ;;
    worker|queue)
        # Background worker mode
        echo "Starting WORKER mode..."
        rm -f "$SUPERVISOR_CONF/stack.dev.conf" "$SUPERVISOR_CONF/stack.prod.conf" 2>/dev/null || true
        ;;
    scheduler)
        # Scheduler only mode
        echo "Starting SCHEDULER mode..."
        export RUN_SCHEDULER="true"
        rm -f "$SUPERVISOR_CONF/stack.dev.conf" "$SUPERVISOR_CONF/stack.prod.conf" 2>/dev/null || true
        # Disable workers in scheduler-only mode
        sed -i 's/autostart=true/autostart=false/' "$SUPERVISOR_CONF/queue.conf" 2>/dev/null || true
        ;;
    *)
        echo "Unknown APP_MODE: $APP_MODE"
        echo "Valid modes: app, worker, queue, scheduler"
        exit 1
        ;;
esac

# ─────────────────────────────────────────────────────────────────────────────
# Pre-flight Checks (app mode only)
# ─────────────────────────────────────────────────────────────────────────────

if [ "$APP_MODE" = "app" ]; then
    cd /app

    # Wait for database
    if [ -n "$DB_HOST" ]; then
        echo "Waiting for database at $DB_HOST:${DB_PORT:-5432}..."
        timeout=30
        while [ $timeout -gt 0 ]; do
            if nc -z "$DB_HOST" "${DB_PORT:-5432}" 2>/dev/null; then
                echo "Database is ready"
                break
            fi
            timeout=$((timeout - 1))
            sleep 1
        done
        if [ $timeout -eq 0 ]; then
            echo "Warning: Database connection timeout"
        fi
    fi

    # Wait for Redis
    if [ -n "$REDIS_HOST" ]; then
        echo "Waiting for Redis at $REDIS_HOST:${REDIS_PORT:-6379}..."
        timeout=30
        while [ $timeout -gt 0 ]; do
            if nc -z "$REDIS_HOST" "${REDIS_PORT:-6379}" 2>/dev/null; then
                echo "Redis is ready"
                break
            fi
            timeout=$((timeout - 1))
            sleep 1
        done
    fi

    # Run migrations (optional)
    if [ "$RUN_MIGRATIONS" = "true" ]; then
        echo "Running migrations..."
        npm run migrate 2>/dev/null || true
    fi
fi

# ─────────────────────────────────────────────────────────────────────────────
# Start Supervisor
# ─────────────────────────────────────────────────────────────────────────────

echo "Starting supervisord..."
exec /usr/bin/supervisord -c /etc/supervisor/supervisord.conf
```

### Entrypoint Pattern Summary

1. **Mode Selection**: Use `APP_MODE` env var to select configs
2. **Environment Switching**: Use `APP_ENV` for dev/prod behavior
3. **Config Pruning**: Remove unused config files at startup
4. **Dependency Waiting**: Wait for databases/caches before starting
5. **Pre-flight Tasks**: Run migrations, cache warming in app mode
6. **Exec Supervisor**: Always `exec` to replace shell with supervisor

---

## Internal Reverse Proxy

### Nginx (For PHP/Static Applications)

Use Nginx when proxying to a single upstream (PHP-FPM, Octane, etc.):

**`nginx.conf`**:
```nginx
user nginx;
worker_processes auto;
error_log /var/log/nginx/error.log notice;
pid /var/run/nginx.pid;

events {
    worker_connections 1024;
    multi_accept on;
    use epoll;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for"';

    access_log /var/log/nginx/access.log main;

    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;

    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types text/plain text/css text/xml application/json application/javascript 
               application/rss+xml application/atom+xml image/svg+xml;

    include /etc/nginx/http.d/*.conf;
}
```

**`http.d/app.conf`**:
```nginx
map $http_upgrade $connection_upgrade {
    default upgrade;
    '' close;
}

server {
    listen 80;
    server_name _;
    root /app/public;
    index index.php index.html;

    charset utf-8;

    # Static assets
    location ~* \.(ico|css|js|gif|jpe?g|png|svg|woff2?|ttf|eot|webp|avif)$ {
        expires 1y;
        access_log off;
        add_header Cache-Control "public, immutable";
        try_files $uri =404;
    }

    # Dev server assets (HMR)
    location /build/ {
        try_files $uri @vite;
    }

    location @vite {
        proxy_pass http://127.0.0.1:5173;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;
        proxy_set_header Host $host;
    }

    # Application
    location / {
        try_files $uri $uri/ @app;
    }

    location @app {
        proxy_pass http://127.0.0.1:8000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Health check
    location /up {
        access_log off;
        return 200 'OK';
        add_header Content-Type text/plain;
    }
}
```

### HAProxy (For Multi-Process/Load Balancing)

Use HAProxy when load balancing across multiple backend processes:

**`haproxy.cfg`**:
```haproxy
global
    log stdout format raw local0 info
    chroot /var/lib/haproxy
    pidfile /var/run/haproxy.pid
    maxconn 4000
    user haproxy
    group haproxy
    daemon

defaults
    mode http
    log global
    option httplog
    option dontlognull
    option http-server-close
    option redispatch
    retries 3
    timeout http-request 10s
    timeout queue 1m
    timeout connect 10s
    timeout client 1m
    timeout server 1m
    timeout http-keep-alive 10s
    timeout check 10s
    maxconn 3000

frontend main
    bind *:80
    option http-server-close

    # Forward headers
    http-request set-header X-Forwarded-Proto https if { ssl_fc }
    http-request set-header X-Forwarded-Proto http if !{ ssl_fc }

    # Health check endpoint
    acl is_health path /up
    use_backend health if is_health

    default_backend app

backend app
    balance roundrobin
    option httpchk GET /up
    http-check expect status 200

    # Dynamic servers added by entrypoint
    # server app-00 127.0.0.1:9000 check
    # server app-01 127.0.0.1:9001 check
    # ...

backend health
    http-request return status 200 content-type text/plain string "OK"
```

**Dynamic HAProxy Configuration**:

In the entrypoint, dynamically generate backend servers:

```bash
_writeHAProxyServers() {
    local num_procs="$1"
    local port_base="$2"
    local config_file="/etc/haproxy/conf.d/servers.cfg"

    echo "" > "$config_file"
    for i in $(seq 0 $((num_procs - 1))); do
        echo "    server app-0$i 127.0.0.1:$((port_base + i)) check" >> "$config_file"
    done
}

# In main():
export APP_NUM_PROCS="${APP_NUM_PROCS:-4}"
_writeHAProxyServers $APP_NUM_PROCS 9000
cat /etc/haproxy/cfg/*.cfg > /etc/haproxy/haproxy.cfg
```

### When to Use Which

| Scenario | Proxy |
|----------|-------|
| Single app server (Node, Octane, etc.) | Nginx |
| PHP-FPM with static files | Nginx |
| Multiple worker processes | HAProxy |
| WebSocket connections | Both work |
| Need load balancing metrics | HAProxy |
| Simple static + proxy | Nginx |

---

## Environment Variables

### Docker Config File (`config/env.service`)

This file becomes `.env.{{SERVICE_NAME}}` and is loaded by Docker Compose:

```bash
# ─────────────────────────────────────────────────────────────────────────────
# {{SERVICE_NAME}} - Container Environment
# ─────────────────────────────────────────────────────────────────────────────

# Application
APP_NAME={{SERVICE_NAME}}
APP_ENV=production
APP_DEBUG=false
APP_URL=https://{{DOMAIN}}
APP_VERSION=1.0.0

# Database
DB_CONNECTION=pgsql
DB_HOST=db
DB_PORT=5432
DB_DATABASE={{SERVICE_NAME}}
DB_USERNAME={{SERVICE_NAME}}
DB_PASSWORD=

# Redis
REDIS_HOST=redis
REDIS_PORT=6379
REDIS_PASSWORD=

# Cache/Queue
CACHE_DRIVER=redis
QUEUE_CONNECTION=redis
SESSION_DRIVER=redis

# Workers
WORKER_COUNT=2
RUN_SCHEDULER=false
```

### Environment Variable Categories

1. **Application Config**: `APP_*` - Runtime behavior
2. **Database**: `DB_*` - Connection settings
3. **Cache/Queue**: `REDIS_*`, `CACHE_*`, `QUEUE_*`
4. **Process Control**: `WORKER_COUNT`, `RUN_SCHEDULER`
5. **Mode Selection**: `APP_MODE`, `APP_ENV`

### Supervisor Environment Interpolation

Supervisor reads environment variables with `%(ENV_NAME)s`:

```ini
[program:worker]
numprocs=%(ENV_WORKER_COUNT)s
autostart=%(ENV_RUN_SCHEDULER)s
```

---

## Compose Files

### Production (`service.yml`)

```yaml
# {{SERVICE_NAME}} - Production Stack

services:
  {{SERVICE_NAME}}:
    build:
      context: ../..
      dockerfile: docker/dockerfiles/{{SERVICE_NAME}}/app
      target: production
    container_name: {{SERVICE_NAME}}
    env_file:
      - ../config/.env.{{SERVICE_NAME}}
    environment:
      - APP_MODE=app
      - APP_ENV=production
    volumes:
      - ../../apps/{{SERVICE_NAME}}/storage:/app/storage
    networks:
      - {{SERVICE_NAME}}-internal
      - traefik
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.{{SERVICE_NAME}}.rule=Host(`{{DOMAIN}}`)"
      - "traefik.http.services.{{SERVICE_NAME}}.loadbalancer.server.port=80"
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "-q", "--spider", "http://localhost/up"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 30s

  {{SERVICE_NAME}}-worker:
    build:
      context: ../..
      dockerfile: docker/dockerfiles/{{SERVICE_NAME}}/app
      target: production
    container_name: {{SERVICE_NAME}}-worker
    env_file:
      - ../config/.env.{{SERVICE_NAME}}
    environment:
      - APP_MODE=worker
      - APP_ENV=production
      - WORKER_COUNT=2
    volumes:
      - ../../apps/{{SERVICE_NAME}}/storage:/app/storage
    networks:
      - {{SERVICE_NAME}}-internal
    depends_on:
      {{SERVICE_NAME}}:
        condition: service_healthy
    restart: unless-stopped

networks:
  {{SERVICE_NAME}}-internal:
    driver: bridge
  traefik:
    external: true
```

### Development Override (`service.dev.yml`)

```yaml
# {{SERVICE_NAME}} - Development Overrides

services:
  {{SERVICE_NAME}}:
    build:
      target: development
    volumes:
      - ../../apps/{{SERVICE_NAME}}:/app
      - /app/node_modules
    environment:
      - APP_MODE=app
      - APP_ENV=local
      - APP_DEBUG=true
    ports:
      - "${{{SERVICE_UPPER}}_PORT:-80}:80"
      - "5173:5173"
    healthcheck:
      test: ["CMD", "wget", "-q", "--spider", "http://localhost/up"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 60s

  {{SERVICE_NAME}}-worker:
    build:
      target: development
    volumes:
      - ../../apps/{{SERVICE_NAME}}:/app
      - /app/node_modules
    environment:
      - APP_MODE=worker
      - APP_ENV=local
```

### Usage

```bash
# Development
docker compose -f docker/compose/myapp.yml -f docker/compose/myapp.dev.yml up

# Production
docker compose -f docker/compose/myapp.yml up -d
```

---

## Best Practices

### 1. Dockerfile Organization

- Use multi-stage builds (base → deps → dev/build → prod)
- Install supervisor and proxy in base stage
- Copy system files with single `COPY` command
- Set `ENTRYPOINT` to your entrypoint script

### 2. System Files

- Mirror exact container filesystem structure
- Use separate supervisor configs per mode
- Include all needed scripts in `usr/local/bin/`
- Set executable permissions in Dockerfile

### 3. Entrypoint Design

- Always use `set -e` at the top
- Export environment defaults early
- Create directories and set permissions
- Remove unused supervisor configs based on mode
- Wait for dependencies before starting
- Use `exec` for final command

### 4. Supervisor

- Stream logs to stdout/stderr for Docker
- Use environment interpolation for dynamic config
- Set appropriate `startsecs` and `startretries`
- Use `stopwaitsecs` for graceful shutdown

### 5. Reverse Proxy

- Nginx for simple proxy + static serving
- HAProxy for load balancing multiple processes
- Always include health check endpoints
- Forward appropriate headers (X-Forwarded-*)

### 6. Environment Variables

- Use consistent naming (`DB_*`, `REDIS_*`, etc.)
- Provide sensible defaults in entrypoint
- Keep secrets out of Dockerfiles
- Use `.env` files for compose

### 7. Compose Files

- Use build targets to switch dev/prod
- Volume mount source code in dev only
- Include healthchecks for orchestration
- Use service dependencies appropriately

---

## Checklist for New Recipes

- [ ] `recipe.json` with all placeholders
- [ ] `app/` with complete application code
- [ ] `docker/dockerfiles/app.Dockerfile` with multi-stage build
- [ ] `docker/system/app/` with filesystem mirror:
  - [ ] `etc/supervisor/supervisord.conf`
  - [ ] `etc/supervisor/conf.d/stack.dev.conf`
  - [ ] `etc/supervisor/conf.d/stack.prod.conf`
  - [ ] `etc/nginx/` or `etc/haproxy/` configs
  - [ ] `usr/local/bin/entrypoint`
- [ ] `docker/compose/service.yml` for production
- [ ] `docker/compose/service.dev.yml` for development
- [ ] `config/env.service` environment template
- [ ] `README.md` with setup instructions
- [ ] Health check endpoint in application
- [ ] Test with `mx add <name> --recipe=<recipe>`

---

*End of Recipe Authoring Guide*
