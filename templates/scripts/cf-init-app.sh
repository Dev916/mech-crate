#!/bin/bash
#
# Initialize a new Cloudflare Worker
# Supports: Regular Workers, Cron Workers, Container Workers
#
# Usage: 
#   ./cf-init-app.sh <app-name>              # Interactive mode
#   ./cf-init-app.sh <app-name> --type=worker|cron|container
#

set -e

# ═══════════════════════════════════════════════════════════════════════════════
# Colors and Styling
# ═══════════════════════════════════════════════════════════════════════════════
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'
BOLD='\033[1m'
DIM='\033[2m'

# ═══════════════════════════════════════════════════════════════════════════════
# Paths
# ═══════════════════════════════════════════════════════════════════════════════
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
CF_DIR="$ROOT_DIR/infra/cloudflare"
CF_ENV_FILE="$CF_DIR/.env.cloudflare"
CF_APPS_DIR="$CF_DIR/apps"

# ═══════════════════════════════════════════════════════════════════════════════
# Logging Functions
# ═══════════════════════════════════════════════════════════════════════════════
info() { echo -e "${BLUE}ℹ${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; exit 1; }
step() { echo -e "${MAGENTA}→${NC} $1"; }

# ═══════════════════════════════════════════════════════════════════════════════
# Parse Arguments
# ═══════════════════════════════════════════════════════════════════════════════
APP_NAME=""
WORKER_TYPE=""
SKIP_PROMPTS=false

for arg in "$@"; do
    case $arg in
        --type=*)
            WORKER_TYPE="${arg#*=}"
            ;;
        --yes|-y)
            SKIP_PROMPTS=true
            ;;
        --help|-h)
            echo "Usage: $0 <app-name> [options]"
            echo ""
            echo "Options:"
            echo "  --type=TYPE    Worker type: worker, cron, container"
            echo "  --yes, -y      Skip confirmation prompts"
            echo "  --help, -h     Show this help"
            echo ""
            echo "Worker Types:"
            echo "  worker     - Standard edge worker (fetch handler)"
            echo "  cron       - Scheduled worker (cron triggers)"
            echo "  container  - Container-backed worker (Durable Objects)"
            exit 0
            ;;
        -*)
            warn "Unknown option: $arg"
            ;;
        *)
            if [[ -z "$APP_NAME" ]]; then
                APP_NAME="$arg"
            fi
            ;;
    esac
done

# ═══════════════════════════════════════════════════════════════════════════════
# Validation
# ═══════════════════════════════════════════════════════════════════════════════
if [[ -z "$APP_NAME" ]]; then
    error "Usage: $0 <app-name> [--type=worker|cron|container]"
fi

# Validate app name
if [[ ! "$APP_NAME" =~ ^[a-zA-Z0-9]([a-zA-Z0-9._-]*[a-zA-Z0-9])?$ ]]; then
    warn "App name '$APP_NAME' contains unusual characters. Proceeding anyway..."
fi

# Load Cloudflare configuration
if [[ ! -f "$CF_ENV_FILE" ]]; then
    error "Cloudflare not configured. Run 'make cf-setup' first."
fi
source "$CF_ENV_FILE"

if [[ -z "$CF_ACCOUNT_ID" ]]; then
    error "CF_ACCOUNT_ID not set. Run 'make cf-setup' first."
fi

APP_DIR="$CF_APPS_DIR/$APP_NAME"

# Check if already exists
if [[ -d "$APP_DIR" ]]; then
    warn "App '$APP_NAME' already exists at $APP_DIR"
    if [[ "$SKIP_PROMPTS" != true ]]; then
        read -r -p "Overwrite? [y/N]: " overwrite
        if [[ ! "$overwrite" =~ ^[Yy] ]]; then
            echo "Aborted."
            exit 0
        fi
    fi
    rm -rf "$APP_DIR"
fi

# ═══════════════════════════════════════════════════════════════════════════════
# Interactive Worker Type Selection
# ═══════════════════════════════════════════════════════════════════════════════
select_worker_type() {
    echo ""
    echo -e "${CYAN}╭────────────────────────────────────────────────────────────╮${NC}"
    echo -e "${CYAN}│${NC}  ${BOLD}🌐 Cloudflare Worker Setup${NC}                               ${CYAN}│${NC}"
    echo -e "${CYAN}│${NC}  App: ${BOLD}$APP_NAME${NC}                                          ${CYAN}│${NC}"
    echo -e "${CYAN}╰────────────────────────────────────────────────────────────╯${NC}"
    echo ""
    echo -e "${BOLD}Select worker type:${NC}"
    echo ""
    echo -e "  ${GREEN}1)${NC} ${BOLD}Regular Worker${NC}"
    echo -e "     ${DIM}Standard edge worker with fetch handler${NC}"
    echo -e "     ${DIM}Best for: APIs, proxies, transformations, static sites${NC}"
    echo ""
    echo -e "  ${GREEN}2)${NC} ${BOLD}Cron Worker${NC}"
    echo -e "     ${DIM}Scheduled worker with cron triggers${NC}"
    echo -e "     ${DIM}Best for: Background jobs, data sync, cleanup tasks${NC}"
    echo ""
    echo -e "  ${GREEN}3)${NC} ${BOLD}Container Worker${NC}"
    echo -e "     ${DIM}Worker backed by Docker container (Durable Objects)${NC}"
    echo -e "     ${DIM}Best for: SSR apps, complex backends, stateful services${NC}"
    echo ""
    
    while true; do
        read -r -p "Enter choice [1-3]: " choice
        case $choice in
            1) WORKER_TYPE="worker"; break ;;
            2) WORKER_TYPE="cron"; break ;;
            3) WORKER_TYPE="container"; break ;;
            *) echo -e "${RED}Invalid choice. Please enter 1, 2, or 3.${NC}" ;;
        esac
    done
}

# ═══════════════════════════════════════════════════════════════════════════════
# Collect Configuration Based on Worker Type
# ═══════════════════════════════════════════════════════════════════════════════
collect_worker_config() {
    echo ""
    echo -e "${CYAN}── Regular Worker Configuration ──${NC}"
    echo ""
    
    # Route configuration
    read -r -p "Configure custom domain route? [y/N]: " use_route
    if [[ "$use_route" =~ ^[Yy] ]]; then
        read -r -p "Domain (e.g., api.example.com): " CUSTOM_DOMAIN
        read -r -p "Zone name (e.g., example.com): " ZONE_NAME
    fi
    
    # KV Namespace
    read -r -p "Add KV namespace binding? [y/N]: " use_kv
    if [[ "$use_kv" =~ ^[Yy] ]]; then
        read -r -p "KV binding name (e.g., CACHE): " KV_BINDING
        KV_BINDING=${KV_BINDING:-CACHE}
    fi
    
    # Environment variables
    read -r -p "Add environment variables? [y/N]: " use_vars
    if [[ "$use_vars" =~ ^[Yy] ]]; then
        echo -e "${DIM}Enter variables (KEY=value), empty line to finish:${NC}"
        ENV_VARS=()
        while true; do
            read -r var
            [[ -z "$var" ]] && break
            ENV_VARS+=("$var")
        done
    fi
}

collect_cron_config() {
    echo ""
    echo -e "${CYAN}── Cron Worker Configuration ──${NC}"
    echo ""
    
    # Cron schedule
    echo -e "${DIM}Common schedules:${NC}"
    echo -e "  ${DIM}*/5 * * * *    - Every 5 minutes${NC}"
    echo -e "  ${DIM}0 * * * *      - Every hour${NC}"
    echo -e "  ${DIM}0 0 * * *      - Daily at midnight${NC}"
    echo -e "  ${DIM}0 0 * * 0      - Weekly on Sunday${NC}"
    echo ""
    read -r -p "Cron schedule [*/5 * * * *]: " CRON_SCHEDULE
    CRON_SCHEDULE=${CRON_SCHEDULE:-"*/5 * * * *"}
    
    # Multiple schedules
    read -r -p "Add additional cron schedules? [y/N]: " add_more
    CRON_SCHEDULES=("$CRON_SCHEDULE")
    while [[ "$add_more" =~ ^[Yy] ]]; do
        read -r -p "Additional schedule: " extra_schedule
        [[ -n "$extra_schedule" ]] && CRON_SCHEDULES+=("$extra_schedule")
        read -r -p "Add another? [y/N]: " add_more
    done
    
    # KV for state persistence
    read -r -p "Add KV namespace for state persistence? [Y/n]: " use_kv
    if [[ ! "$use_kv" =~ ^[Nn] ]]; then
        KV_BINDING="CRON_STATE"
    fi
    
    # Webhook notification
    read -r -p "Configure webhook notifications? [y/N]: " use_webhook
    if [[ "$use_webhook" =~ ^[Yy] ]]; then
        read -r -p "Webhook URL (leave empty to configure later): " WEBHOOK_URL
    fi
}

collect_container_config() {
    echo ""
    echo -e "${CYAN}── Container Worker Configuration ──${NC}"
    echo ""
    
    # Container port
    read -r -p "Container port [8080]: " CONTAINER_PORT
    CONTAINER_PORT=${CONTAINER_PORT:-8080}
    
    # Sleep timeout
    echo -e "${DIM}Sleep timeout (container shuts down after inactivity):${NC}"
    read -r -p "Sleep after [10m]: " SLEEP_AFTER
    SLEEP_AFTER=${SLEEP_AFTER:-"10m"}
    
    # Max instances
    read -r -p "Max container instances [5]: " MAX_INSTANCES
    MAX_INSTANCES=${MAX_INSTANCES:-5}
    
    # Production max instances
    read -r -p "Production max instances [10]: " PROD_MAX_INSTANCES
    PROD_MAX_INSTANCES=${PROD_MAX_INSTANCES:-10}
    
    # Custom domain
    read -r -p "Configure custom domain? [y/N]: " use_domain
    if [[ "$use_domain" =~ ^[Yy] ]]; then
        read -r -p "Domain (e.g., app.example.com): " CUSTOM_DOMAIN
        read -r -p "Zone name (e.g., example.com): " ZONE_NAME
    fi
    
    # Create Dockerfile
    read -r -p "Create Dockerfile template? [Y/n]: " create_dockerfile
    CREATE_DOCKERFILE=true
    [[ "$create_dockerfile" =~ ^[Nn] ]] && CREATE_DOCKERFILE=false
}

# ═══════════════════════════════════════════════════════════════════════════════
# Generate Regular Worker
# ═══════════════════════════════════════════════════════════════════════════════
generate_regular_worker() {
    step "Creating regular worker source..."
    
    cat > "$APP_DIR/src/index.ts" << 'EOF'
/**
 * Regular Cloudflare Worker
 * 
 * Handles HTTP requests at the edge with minimal latency.
 * Supports KV, R2, D1, and other Cloudflare bindings.
 */

export interface Env {
  // KV Namespace binding (uncomment if needed)
  // CACHE: KVNamespace;
  
  // Environment variables
  ENVIRONMENT: string;
}

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);
    
    // Health check endpoint
    if (url.pathname === '/_health' || url.pathname === '/_healthz') {
      return Response.json({
        status: 'healthy',
        timestamp: new Date().toISOString(),
        environment: env.ENVIRONMENT || 'development'
      });
    }
    
    // API routes
    if (url.pathname.startsWith('/api/')) {
      return handleApiRequest(request, env, ctx);
    }
    
    // Default response
    return new Response(JSON.stringify({
      message: 'Hello from __APP_NAME__!',
      path: url.pathname,
      method: request.method
    }), {
      headers: { 'Content-Type': 'application/json' }
    });
  }
};

async function handleApiRequest(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
  const url = new URL(request.url);
  
  // Example: GET /api/hello
  if (url.pathname === '/api/hello' && request.method === 'GET') {
    return Response.json({ message: 'Hello, World!' });
  }
  
  // Example: POST /api/echo
  if (url.pathname === '/api/echo' && request.method === 'POST') {
    const body = await request.json();
    return Response.json({ echo: body });
  }
  
  return Response.json({ error: 'Not Found' }, { status: 404 });
}
EOF

    # Replace placeholder
    sed -i '' "s/__APP_NAME__/$APP_NAME/g" "$APP_DIR/src/index.ts" 2>/dev/null || \
    sed -i "s/__APP_NAME__/$APP_NAME/g" "$APP_DIR/src/index.ts" 2>/dev/null || true
    
    step "Creating wrangler.toml..."
    
    cat > "$APP_DIR/wrangler.toml" << EOF
# Cloudflare Worker Configuration
# Type: Regular Worker
# App: ${APP_NAME}

name = "${APP_NAME}"
main = "src/index.ts"
compatibility_date = "2024-12-01"
account_id = "${CF_ACCOUNT_ID}"

# Development settings
workers_dev = true

[vars]
ENVIRONMENT = "development"

EOF

    # Add KV binding if configured
    if [[ -n "$KV_BINDING" ]]; then
        cat >> "$APP_DIR/wrangler.toml" << EOF
# KV Namespace (create with: wrangler kv:namespace create "${KV_BINDING}")
# [[kv_namespaces]]
# binding = "${KV_BINDING}"
# id = "your-kv-namespace-id"

EOF
    fi

    # Add routes if configured
    if [[ -n "$CUSTOM_DOMAIN" ]]; then
        cat >> "$APP_DIR/wrangler.toml" << EOF
# ─────────────────────────────────────────────────────────────────────────────
# Production Environment
# ─────────────────────────────────────────────────────────────────────────────
[env.production]
workers_dev = false
routes = [
  { pattern = "${CUSTOM_DOMAIN}/*", zone_name = "${ZONE_NAME:-$CUSTOM_DOMAIN}" }
]

[env.production.vars]
ENVIRONMENT = "production"
EOF
    else
        cat >> "$APP_DIR/wrangler.toml" << EOF
# ─────────────────────────────────────────────────────────────────────────────
# Production Environment
# ─────────────────────────────────────────────────────────────────────────────
[env.production]
workers_dev = false
# Uncomment and configure your domain:
# routes = [
#   { pattern = "your-domain.com/*", zone_name = "your-domain.com" }
# ]

[env.production.vars]
ENVIRONMENT = "production"
EOF
    fi
}

# ═══════════════════════════════════════════════════════════════════════════════
# Generate Cron Worker
# ═══════════════════════════════════════════════════════════════════════════════
generate_cron_worker() {
    step "Creating cron worker source..."
    
    cat > "$APP_DIR/src/index.ts" << 'EOF'
/**
 * Cron Worker - Scheduled Tasks
 * 
 * Executes on a schedule defined in wrangler.toml.
 * Use for background jobs, data sync, cleanup, and more.
 */

export interface Env {
  // KV for persisting state between runs
  CRON_STATE: KVNamespace;
  
  // Optional: Webhook URL for notifications
  WEBHOOK_URL?: string;
  
  ENVIRONMENT: string;
}

interface CronState {
  lastRun: string;
  runCount: number;
  lastStatus: 'success' | 'error';
  lastError?: string;
}

export default {
  // Scheduled handler - runs on cron trigger
  async scheduled(event: ScheduledEvent, env: Env, ctx: ExecutionContext): Promise<void> {
    console.log(`[__APP_NAME__] Cron triggered at ${new Date(event.scheduledTime).toISOString()}`);
    console.log(`[__APP_NAME__] Cron pattern: ${event.cron}`);
    
    const startTime = Date.now();
    let status: 'success' | 'error' = 'success';
    let error: string | undefined;
    
    try {
      // ═══════════════════════════════════════════════════════════════════════
      // YOUR CRON JOB LOGIC HERE
      // ═══════════════════════════════════════════════════════════════════════
      
      await runCronJob(event, env, ctx);
      
      // ═══════════════════════════════════════════════════════════════════════
      
    } catch (e) {
      status = 'error';
      error = e instanceof Error ? e.message : String(e);
      console.error(`[__APP_NAME__] Cron error:`, e);
      
      // Send error notification if webhook configured
      if (env.WEBHOOK_URL) {
        ctx.waitUntil(sendWebhook(env.WEBHOOK_URL, {
          app: '__APP_NAME__',
          event: 'cron_error',
          error,
          cron: event.cron,
          timestamp: new Date().toISOString()
        }));
      }
    }
    
    // Update state
    const state = await getState(env);
    state.lastRun = new Date().toISOString();
    state.runCount += 1;
    state.lastStatus = status;
    state.lastError = error;
    
    ctx.waitUntil(saveState(env, state));
    
    const duration = Date.now() - startTime;
    console.log(`[__APP_NAME__] Cron completed in ${duration}ms with status: ${status}`);
  },
  
  // Optional: HTTP handler for status checks
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);
    
    // Health/status endpoint
    if (url.pathname === '/_health' || url.pathname === '/status') {
      const state = await getState(env);
      return Response.json({
        app: '__APP_NAME__',
        type: 'cron',
        state,
        environment: env.ENVIRONMENT
      });
    }
    
    // Manual trigger (useful for testing)
    if (url.pathname === '/trigger' && request.method === 'POST') {
      const event: ScheduledEvent = {
        scheduledTime: Date.now(),
        cron: 'manual',
        noRetry: () => {}
      };
      
      ctx.waitUntil((async () => {
        await this.scheduled(event, env, ctx);
      })());
      
      return Response.json({ message: 'Cron job triggered', timestamp: new Date().toISOString() });
    }
    
    return Response.json({
      app: '__APP_NAME__',
      type: 'cron',
      endpoints: {
        '/_health': 'GET - Health check and last run status',
        '/trigger': 'POST - Manually trigger cron job'
      }
    });
  }
};

// ═══════════════════════════════════════════════════════════════════════════════
// Your Cron Job Implementation
// ═══════════════════════════════════════════════════════════════════════════════

async function runCronJob(event: ScheduledEvent, env: Env, ctx: ExecutionContext): Promise<void> {
  // Example: Fetch data from an API
  // const response = await fetch('https://api.example.com/data');
  // const data = await response.json();
  
  // Example: Process and store data
  // await env.CRON_STATE.put('latest_data', JSON.stringify(data));
  
  // Example: Clean up old data
  // const keys = await env.CRON_STATE.list({ prefix: 'temp_' });
  // for (const key of keys.keys) {
  //   await env.CRON_STATE.delete(key.name);
  // }
  
  console.log('[__APP_NAME__] Cron job executed successfully');
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

async function getState(env: Env): Promise<CronState> {
  const data = await env.CRON_STATE.get('_state', 'json');
  return (data as CronState) || {
    lastRun: 'never',
    runCount: 0,
    lastStatus: 'success'
  };
}

async function saveState(env: Env, state: CronState): Promise<void> {
  await env.CRON_STATE.put('_state', JSON.stringify(state));
}

async function sendWebhook(url: string, payload: object): Promise<void> {
  try {
    await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
  } catch (e) {
    console.error('Webhook failed:', e);
  }
}
EOF

    # Replace placeholder
    sed -i '' "s/__APP_NAME__/$APP_NAME/g" "$APP_DIR/src/index.ts" 2>/dev/null || \
    sed -i "s/__APP_NAME__/$APP_NAME/g" "$APP_DIR/src/index.ts" 2>/dev/null || true
    
    step "Creating wrangler.toml..."
    
    # Build triggers section
    TRIGGERS=""
    for schedule in "${CRON_SCHEDULES[@]}"; do
        TRIGGERS="${TRIGGERS}  \"${schedule}\","$'\n'
    done
    # Remove trailing comma and newline
    TRIGGERS="${TRIGGERS%,
}"
    
    cat > "$APP_DIR/wrangler.toml" << EOF
# Cloudflare Worker Configuration
# Type: Cron Worker (Scheduled)
# App: ${APP_NAME}

name = "${APP_NAME}"
main = "src/index.ts"
compatibility_date = "2024-12-01"
account_id = "${CF_ACCOUNT_ID}"

# Development settings
workers_dev = true

[vars]
ENVIRONMENT = "development"
EOF

    # Add webhook if configured
    if [[ -n "$WEBHOOK_URL" ]]; then
        echo "WEBHOOK_URL = \"${WEBHOOK_URL}\"" >> "$APP_DIR/wrangler.toml"
    fi

    cat >> "$APP_DIR/wrangler.toml" << EOF

# KV Namespace for state persistence
# Create with: wrangler kv:namespace create "CRON_STATE"
# Then add the ID below:
[[kv_namespaces]]
binding = "CRON_STATE"
id = "TODO_CREATE_KV_NAMESPACE"
preview_id = "TODO_CREATE_PREVIEW_KV_NAMESPACE"

# ─────────────────────────────────────────────────────────────────────────────
# Cron Triggers
# ─────────────────────────────────────────────────────────────────────────────
[triggers]
crons = [
${TRIGGERS}
]

# ─────────────────────────────────────────────────────────────────────────────
# Production Environment
# ─────────────────────────────────────────────────────────────────────────────
[env.production]
workers_dev = false

[env.production.vars]
ENVIRONMENT = "production"
EOF

    if [[ -n "$WEBHOOK_URL" ]]; then
        echo "WEBHOOK_URL = \"${WEBHOOK_URL}\"" >> "$APP_DIR/wrangler.toml"
    fi

    cat >> "$APP_DIR/wrangler.toml" << EOF

[[env.production.kv_namespaces]]
binding = "CRON_STATE"
id = "TODO_CREATE_PROD_KV_NAMESPACE"

[env.production.triggers]
crons = [
${TRIGGERS}
]
EOF
}

# ═══════════════════════════════════════════════════════════════════════════════
# Generate Container Worker
# ═══════════════════════════════════════════════════════════════════════════════
generate_container_worker() {
    step "Creating container worker source..."
    
    CONTAINER_PORT=${CONTAINER_PORT:-8080}
    SLEEP_AFTER=${SLEEP_AFTER:-"10m"}
    MAX_INSTANCES=${MAX_INSTANCES:-5}
    PROD_MAX_INSTANCES=${PROD_MAX_INSTANCES:-10}
    
    cat > "$APP_DIR/src/index.ts" << EOF
/**
 * Container Worker - Docker Container Backend
 * 
 * Routes requests to a Docker container running via Durable Objects.
 * Supports auto-scaling, health checks, and container lifecycle management.
 */

import { Container, getContainer } from '@cloudflare/containers';

const HEALTH_PATHS = new Set(['/_health', '/_healthz', '/_readyz']);
const DEFAULT_INSTANCE = '${APP_NAME}-ssr';
const CONTAINER_PORT = ${CONTAINER_PORT};

export class ContainerDO extends Container {
  defaultPort = CONTAINER_PORT;
  sleepAfter = '${SLEEP_AFTER}';
  envVars = {
    NODE_ENV: 'production',
    HOST: '0.0.0.0',
    PORT: \`\${CONTAINER_PORT}\`
  };
}

interface Env {
  APP_CONTAINER: DurableObjectNamespace<ContainerDO>;
  CONTAINER_INSTANCE_ID?: string;
}

const worker = {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const instanceId =
      url.searchParams.get('instance') ?? env.CONTAINER_INSTANCE_ID ?? DEFAULT_INSTANCE;
    const container = getContainer(env.APP_CONTAINER, instanceId);

    // Health endpoints
    if (HEALTH_PATHS.has(url.pathname)) {
      const state = await container.getState();
      return Response.json(
        {
          app: '${APP_NAME}',
          status: state.status,
          lastChange: new Date(state.lastChange).toISOString(),
          exitCode: 'exitCode' in state ? state.exitCode : undefined
        },
        { headers: { 'cache-control': 'no-store' } }
      );
    }

    // Container management endpoints
    if (url.pathname === '/_container/restart') {
      if (request.method !== 'POST') return new Response('Method Not Allowed', { status: 405 });
      await container.destroy();
      return new Response('Restart requested', { status: 202 });
    }

    if (url.pathname === '/_container/start') {
      if (request.method !== 'POST') return new Response('Method Not Allowed', { status: 405 });
      await container.startAndWaitForPorts({ ports: [CONTAINER_PORT] });
      const state = await container.getState();
      return Response.json({ message: 'Container started', state, instanceId });
    }

    if (url.pathname === '/_container/status') {
      const state = await container.getState();
      return Response.json({ app: '${APP_NAME}', state, instanceId });
    }

    if (url.pathname === '/_container/stop') {
      if (request.method !== 'POST') return new Response('Method Not Allowed', { status: 405 });
      await container.stop();
      return new Response('Stop signal sent', { status: 202 });
    }

    return container.fetch(request);
  }
};

export default worker;
EOF

    step "Creating wrangler.toml..."
    
    cat > "$APP_DIR/wrangler.toml" << EOF
# Cloudflare Worker Configuration
# Type: Container Worker (Durable Objects)
# App: ${APP_NAME}

name = "${APP_NAME}-worker"
main = "src/index.ts"
compatibility_date = "2024-12-01"
account_id = "${CF_ACCOUNT_ID}"
workers_dev = true

[vars]
CONTAINER_INSTANCE_ID = "${APP_NAME}-dev"

[[containers]]
name = "${APP_NAME}"
class_name = "ContainerDO"
image = "registry.cloudflare.com/${CF_ACCOUNT_ID}/${APP_NAME}:v0.0.1"
max_instances = ${MAX_INSTANCES}

[[durable_objects.bindings]]
name = "APP_CONTAINER"
class_name = "ContainerDO"

[[migrations]]
tag = "v1"
new_sqlite_classes = ["ContainerDO"]

# ─────────────────────────────────────────────────────────────────────────────
# Preview Environment
# ─────────────────────────────────────────────────────────────────────────────
[env.preview]
workers_dev = true

[[env.preview.containers]]
name = "${APP_NAME}"
class_name = "ContainerDO"
image = "registry.cloudflare.com/${CF_ACCOUNT_ID}/${APP_NAME}:v0.0.1"
max_instances = 3

[[env.preview.durable_objects.bindings]]
name = "APP_CONTAINER"
class_name = "ContainerDO"

# ─────────────────────────────────────────────────────────────────────────────
# Production Environment
# ─────────────────────────────────────────────────────────────────────────────
[env.production]
workers_dev = false
EOF

    # Add routes if configured
    if [[ -n "$CUSTOM_DOMAIN" ]]; then
        cat >> "$APP_DIR/wrangler.toml" << EOF
routes = [
  { pattern = "${CUSTOM_DOMAIN}/*", zone_name = "${ZONE_NAME:-$CUSTOM_DOMAIN}" }
]
EOF
    else
        cat >> "$APP_DIR/wrangler.toml" << EOF
routes = [
  { pattern = "${APP_NAME}/*", zone_name = "${APP_NAME}" }
]
EOF
    fi

    cat >> "$APP_DIR/wrangler.toml" << EOF

[[env.production.containers]]
name = "${APP_NAME}"
class_name = "ContainerDO"
image = "registry.cloudflare.com/${CF_ACCOUNT_ID}/${APP_NAME}:v0.0.1"
max_instances = ${PROD_MAX_INSTANCES}

[[env.production.durable_objects.bindings]]
name = "APP_CONTAINER"
class_name = "ContainerDO"

[env.production.vars]
CONTAINER_INSTANCE_ID = "${APP_NAME}-prod"
EOF

    # Create Dockerfile if requested
    if [[ "$CREATE_DOCKERFILE" == true ]]; then
        DOCKERFILE_DIR="$ROOT_DIR/docker/dockerfiles/$APP_NAME"
        mkdir -p "$DOCKERFILE_DIR"
        
        step "Creating Dockerfile template..."
        cat > "$DOCKERFILE_DIR/app" << EOF
# Production Dockerfile for Cloudflare Containers
# App: ${APP_NAME}
# Build: docker build -f docker/dockerfiles/${APP_NAME}/app -t ${APP_NAME}:v0.0.1 .

# ─────────────────────────────────────────────────────────────────────────────
# Build Stage
# ─────────────────────────────────────────────────────────────────────────────
FROM node:20-alpine AS builder
WORKDIR /app

ARG APP_VERSION=0.0.1
ENV APP_VERSION=\$APP_VERSION

COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build

# ─────────────────────────────────────────────────────────────────────────────
# Production Stage
# ─────────────────────────────────────────────────────────────────────────────
FROM node:20-alpine AS production
WORKDIR /app

RUN addgroup -g 1001 -S nodejs && \\
    adduser -S nodejs -u 1001

COPY --from=builder --chown=nodejs:nodejs /app/dist ./dist
COPY --from=builder --chown=nodejs:nodejs /app/node_modules ./node_modules
COPY --from=builder --chown=nodejs:nodejs /app/package.json ./

USER nodejs

# Cloudflare Containers configuration
ENV PORT=${CONTAINER_PORT}
ENV HOST=0.0.0.0
ENV NODE_ENV=production

EXPOSE ${CONTAINER_PORT}

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s \\
    CMD node -e "require('http').get('http://localhost:${CONTAINER_PORT}/health', (r) => process.exit(r.statusCode === 200 ? 0 : 1))"

CMD ["node", "dist/index.js"]
EOF
    fi
}

# ═══════════════════════════════════════════════════════════════════════════════
# Generate package.json (common for all types)
# ═══════════════════════════════════════════════════════════════════════════════
generate_package_json() {
    step "Creating package.json..."
    
    local deps=""
    local dev_deps='"@cloudflare/workers-types": "^4.20241205.0",
    "typescript": "^5.7.2",
    "wrangler": "^3.99.0"'
    
    # Add container dependencies
    if [[ "$WORKER_TYPE" == "container" ]]; then
        deps='"@cloudflare/containers": "^0.0.30"'
    fi
    
    cat > "$APP_DIR/package.json" << EOF
{
  "name": "${APP_NAME}-worker",
  "private": true,
  "type": "module",
  "version": "0.0.1",
  "scripts": {
    "dev": "wrangler dev",
    "deploy": "wrangler deploy",
    "deploy:preview": "wrangler deploy --env preview",
    "deploy:production": "wrangler deploy --env production",
    "typecheck": "tsc --noEmit",
    "tail": "wrangler tail"
  },
EOF

    if [[ -n "$deps" ]]; then
        cat >> "$APP_DIR/package.json" << EOF
  "dependencies": {
    ${deps}
  },
EOF
    fi

    cat >> "$APP_DIR/package.json" << EOF
  "devDependencies": {
    ${dev_deps}
  }
}
EOF
}

# ═══════════════════════════════════════════════════════════════════════════════
# Generate tsconfig.json (common for all types)
# ═══════════════════════════════════════════════════════════════════════════════
generate_tsconfig() {
    step "Creating tsconfig.json..."
    
    cat > "$APP_DIR/tsconfig.json" << 'EOF'
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ES2022",
    "moduleResolution": "bundler",
    "lib": ["ES2022"],
    "types": ["@cloudflare/workers-types/2023-07-01"],
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true,
    "esModuleInterop": true,
    "allowSyntheticDefaultImports": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules"]
}
EOF
}

# ═══════════════════════════════════════════════════════════════════════════════
# Generate environment config
# ═══════════════════════════════════════════════════════════════════════════════
generate_env_config() {
    local APP_NAME_SAFE=$(echo "$APP_NAME" | tr '.-' '_' | tr '[:upper:]' '[:lower:]')
    local APP_NAME_UPPER=$(echo "$APP_NAME_SAFE" | tr '[:lower:]' '[:upper:]')
    
    ENV_DIR="$ROOT_DIR/docker/.config"
    if [[ ! -f "$ENV_DIR/.env.$APP_NAME_SAFE" ]]; then
        step "Creating environment config..."
        mkdir -p "$ENV_DIR"
        
        if [[ "$WORKER_TYPE" == "container" ]]; then
            cat > "$ENV_DIR/.env.$APP_NAME_SAFE" << EOF
# ${APP_NAME} configuration
${APP_NAME_UPPER}_PORT=${CONTAINER_PORT:-8080}
${APP_NAME_UPPER}_LOG_LEVEL=info
NODE_ENV=production
EOF
        else
            cat > "$ENV_DIR/.env.$APP_NAME_SAFE" << EOF
# ${APP_NAME} configuration
${APP_NAME_UPPER}_LOG_LEVEL=info
NODE_ENV=production
EOF
        fi
    fi
}

# ═══════════════════════════════════════════════════════════════════════════════
# Print Summary
# ═══════════════════════════════════════════════════════════════════════════════
print_summary() {
    local TYPE_LABEL=""
    case $WORKER_TYPE in
        worker) TYPE_LABEL="Regular Worker" ;;
        cron) TYPE_LABEL="Cron Worker" ;;
        container) TYPE_LABEL="Container Worker" ;;
    esac
    
    echo ""
    success "App '${APP_NAME}' initialized as ${TYPE_LABEL}!"
    echo ""
    
    echo -e "${CYAN}╭────────────────────────────────────────────────────────────╮${NC}"
    echo -e "${CYAN}│${NC}  ${BOLD}📁 Created Files${NC}                                         ${CYAN}│${NC}"
    echo -e "${CYAN}├────────────────────────────────────────────────────────────┤${NC}"
    echo -e "${CYAN}│${NC}  ${DIM}$APP_DIR/${NC}"
    echo -e "${CYAN}│${NC}    ├── src/index.ts"
    echo -e "${CYAN}│${NC}    ├── wrangler.toml"
    echo -e "${CYAN}│${NC}    ├── package.json"
    echo -e "${CYAN}│${NC}    └── tsconfig.json"
    
    if [[ "$WORKER_TYPE" == "container" && "$CREATE_DOCKERFILE" == true ]]; then
        echo -e "${CYAN}│${NC}"
        echo -e "${CYAN}│${NC}  ${DIM}docker/dockerfiles/$APP_NAME/${NC}"
        echo -e "${CYAN}│${NC}    └── app"
    fi
    
    echo -e "${CYAN}╰────────────────────────────────────────────────────────────╯${NC}"
    
    echo ""
    echo -e "${BOLD}Next Steps:${NC}"
    echo ""
    
    case $WORKER_TYPE in
        worker)
            echo "  1. Install dependencies:"
            echo "     ${BOLD}make cf-install a=$APP_NAME${NC}"
            echo ""
            echo "  2. Run locally:"
            echo "     ${BOLD}make cf-dev a=$APP_NAME${NC}"
            echo ""
            echo "  3. Deploy to production:"
            echo "     ${BOLD}make cf-deploy a=$APP_NAME${NC}"
            ;;
        cron)
            echo "  1. Create KV namespace for state:"
            echo "     ${BOLD}cd infra/cloudflare/apps/$APP_NAME${NC}"
            echo "     ${BOLD}wrangler kv:namespace create CRON_STATE${NC}"
            echo "     ${BOLD}wrangler kv:namespace create CRON_STATE --preview${NC}"
            echo ""
            echo "  2. Update wrangler.toml with KV namespace IDs"
            echo ""
            echo "  3. Install dependencies:"
            echo "     ${BOLD}make cf-install a=$APP_NAME${NC}"
            echo ""
            echo "  4. Test locally (manual trigger):"
            echo "     ${BOLD}make cf-dev a=$APP_NAME${NC}"
            echo "     ${DIM}curl -X POST http://localhost:8787/trigger${NC}"
            echo ""
            echo "  5. Deploy to production:"
            echo "     ${BOLD}make cf-deploy a=$APP_NAME${NC}"
            ;;
        container)
            echo "  1. Configure your domain in Cloudflare DNS"
            if [[ -n "$CUSTOM_DOMAIN" ]]; then
                echo "     Point ${BOLD}${CUSTOM_DOMAIN}${NC} to Cloudflare (proxied)"
            else
                echo "     Point ${BOLD}${APP_NAME}${NC} to Cloudflare (proxied)"
            fi
            echo ""
            echo "  2. Update the Dockerfile for your app:"
            echo "     ${BOLD}docker/dockerfiles/$APP_NAME/app${NC}"
            echo ""
            echo "  3. Install dependencies:"
            echo "     ${BOLD}make cf-install a=$APP_NAME${NC}"
            echo ""
            echo "  4. Build and push container:"
            echo "     ${BOLD}make cf-publish a=$APP_NAME${NC}"
            echo ""
            echo "  5. Deploy to production:"
            echo "     ${BOLD}make cf-deploy a=$APP_NAME${NC}"
            ;;
    esac
    
    echo ""
}

# ═══════════════════════════════════════════════════════════════════════════════
# Main Execution
# ═══════════════════════════════════════════════════════════════════════════════

# Select worker type if not provided
if [[ -z "$WORKER_TYPE" ]]; then
    select_worker_type
fi

# Validate worker type
case $WORKER_TYPE in
    worker|cron|container) ;;
    *)
        error "Invalid worker type '$WORKER_TYPE'. Use: worker, cron, or container"
        ;;
esac

# Collect configuration
case $WORKER_TYPE in
    worker) collect_worker_config ;;
    cron) collect_cron_config ;;
    container) collect_container_config ;;
esac

echo ""
echo -e "${CYAN}╭────────────────────────────────────────────────────────────╮${NC}"
echo -e "${CYAN}│${NC}  ${BOLD}🚀 Creating: $APP_NAME${NC}"
echo -e "${CYAN}│${NC}  ${DIM}Type: $WORKER_TYPE${NC}"
echo -e "${CYAN}╰────────────────────────────────────────────────────────────╯${NC}"
echo ""

# Create app directory
mkdir -p "$APP_DIR/src"

# Generate files based on worker type
case $WORKER_TYPE in
    worker) generate_regular_worker ;;
    cron) generate_cron_worker ;;
    container) generate_container_worker ;;
esac

# Generate common files
generate_package_json
generate_tsconfig
generate_env_config

# Print summary
print_summary
