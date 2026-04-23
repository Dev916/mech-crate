# MechCrate Router

The MechCrate Router is a workstation-wide Traefik reverse proxy that enables running multiple projects simultaneously with hostname-based routing. Instead of juggling ports (`localhost:3000`, `localhost:8080`), access all your services via clean hostnames like `api.localhost`, `admin.localhost`, and `docs.localhost`.

## Table of Contents

1. [Quick Start](#quick-start)
2. [How It Works](#how-it-works)
3. [Commands](#commands)
4. [Configuration](#configuration)
5. [Connecting Services](#connecting-services)
6. [Dashboard](#dashboard)
7. [HTTPS Support](#https-support)
8. [Dynamic Configuration](#dynamic-configuration)
9. [Troubleshooting](#troubleshooting)
10. [Environment Variables](#environment-variables)

---

## Quick Start

```bash
# Install the router (first-time only)
mx router install

# Start the router
mx router up

# Check the dashboard URL
mx router inspect
```

Once running, any MechCrate service with proper labels will automatically be accessible via its hostname.

---

## How It Works

```
┌─────────────────────────────────────────────────────────────────────-┐
│                         Your Browser                                 │
│                                                                      │
│   http://api.localhost    http://admin.localhost    http://...       │
└─────────────────────────────┬────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────────-─┐
│                     MechCrate Router (Traefik)                       │
│                                                                      │
│   Listens on:  80 (HTTP), 443 (HTTPS), 7680+ (Dashboard)             │
│   Network:     devmesh-traefik                                       │
│                                                                      │
│   Routes requests based on Host header to matching containers        │
└─────────────────────────────┬────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│   Project A   │     │   Project B   │     │   Project C   │
│               │     │               │     │               │
│  api.localhost│     │admin.localhost│     │ docs.localhost│
│     :80       │     │     :80       │     │     :3000     │
└───────────────┘     └───────────────┘     └───────────────┘
```

### Key Concepts

1. **Single Entry Point**: The router binds to ports 80/443 on your machine. All HTTP(S) traffic flows through it.

2. **Docker Label Discovery**: Traefik watches the Docker socket and automatically discovers containers with `traefik.enable=true` labels.

3. **Shared Network**: All services that want routing must join the `devmesh-traefik` network. This is the only way Traefik can reach them.

4. **No Port Conflicts**: Since Traefik handles all routing, your services don't need to expose ports directly. Multiple services can all listen on port 80 internally.

---

## Commands

### `mx router install`

First-time setup. Copies the router template to `~/.mech-crate/router` and creates the shared Docker network.

```bash
mx router install
```

**What it does:**
- Creates `~/.mech-crate/router/` directory
- Copies Traefik configuration files
- Creates `devmesh-traefik` Docker network
- Sets proper permissions on certificate storage

### `mx router up`

Start or update the router. If already running, this will apply any configuration changes.

```bash
mx router up
```

**Output:**
```
[router] Dashboard: http://localhost:7680
```

### `mx router down`

Stop the router. Running services will become inaccessible via hostname until the router is restarted.

```bash
mx router down
```

### `mx router restart`

Stop and start the router. Useful after major configuration changes.

```bash
mx router restart
```

### `mx router status`

Show the router container status.

```bash
mx router status
```

**Output:**
```
NAME         IMAGE          STATUS         PORTS
mx-router    traefik:v3.6.1   Up 2 hours     0.0.0.0:80->80/tcp, ...
```

### `mx router logs`

Tail the router logs. Press `Ctrl+C` to stop.

```bash
mx router logs
```

Useful for debugging routing issues—you'll see every request that hits the router.

### `mx router reload`

Hot-reload configuration without restarting the container. Use this after editing files in `~/.mech-crate/router/config/dynamic/`.

```bash
mx router reload
```

### `mx router inspect`

Show router details including the dashboard URL and connected services.

```bash
mx router inspect
```

**Output:**
```
MechCrate Router

  State Dir  : /Users/you/.mech-crate/router
  Compose    : /Users/you/.mech-crate/router/docker-compose.yml
  Network    : devmesh-traefik
  Project    : mx-router
  Port Range : 7680-7799
  Dashboard  : http://localhost:7680

Connected Services:
  - mx-router
  - api
  - admin
  - redis
  - db
```

### `mx router network`

Ensure the shared network exists and print its name. Useful in scripts.

```bash
mx router network
# Output: devmesh-traefik
```

### `mx router uninstall`

Remove the router installation completely.

```bash
mx router uninstall
```

You'll be prompted whether to also remove the Docker network.

---

## Configuration

### File Locations

| Path | Purpose |
|------|---------|
| `~/.mech-crate/router/` | State directory (all router files) |
| `~/.mech-crate/router/docker-compose.yml` | Router container definition |
| `~/.mech-crate/router/config/traefik.yml` | Main Traefik configuration |
| `~/.mech-crate/router/config/dynamic/` | Dynamic configuration (hot-reloadable) |
| `~/.mech-crate/router/letsencrypt/` | HTTPS certificate storage |

### Main Configuration (`traefik.yml`)

```yaml
entryPoints:
  web:
    address: ":80"
  websecure:
    address: ":443"

providers:
  docker:
    endpoint: unix:///var/run/docker.sock
    watch: true
    exposedByDefault: false        # Only route containers with traefik.enable=true
    network: devmesh-traefik       # Only look at containers on this network
  file:
    directory: /etc/traefik/dynamic
    watch: true                    # Hot-reload dynamic config

api:
  dashboard: true
  insecure: true                   # Dashboard accessible without auth (dev only)

ping:
  entryPoint: web                  # Health check endpoint

log:
  level: INFO

accessLog: {}                      # Log all requests
```

---

## Connecting Services

### Required Labels

Every service that needs routing must have these Docker labels:

```yaml
services:
  myapp:
    # ... build, volumes, etc ...
    networks:
      - default              # For internal communication (db, redis)
      - devmesh-traefik      # For Traefik routing
    labels:
      - traefik.enable=true
      - traefik.http.routers.myapp.rule=Host(`myapp.localhost`)
      - traefik.http.routers.myapp.entrypoints=web
      - traefik.http.services.myapp.loadbalancer.server.port=80
      - traefik.docker.network=devmesh-traefik

networks:
  devmesh-traefik:
    external: true
```

### Label Reference

| Label | Required | Description |
|-------|----------|-------------|
| `traefik.enable=true` | Yes | Enable routing for this container |
| `traefik.http.routers.<name>.rule` | Yes | Routing rule (usually `Host(...)`) |
| `traefik.http.routers.<name>.entrypoints` | Yes | Which ports to listen on |
| `traefik.http.services.<name>.loadbalancer.server.port` | Yes | Container's internal port |
| `traefik.docker.network` | Yes | Which network Traefik should use |

### Routing Rules

```yaml
# Simple hostname
- traefik.http.routers.myapp.rule=Host(`myapp.localhost`)

# Multiple hostnames
- traefik.http.routers.myapp.rule=Host(`myapp.localhost`) || Host(`app.localhost`)

# Hostname with path prefix
- traefik.http.routers.myapp.rule=Host(`api.localhost`) && PathPrefix(`/v1`)

# Regex hostname (all subdomains)
- traefik.http.routers.myapp.rule=HostRegexp(`{subdomain:.+}.myapp.localhost`)
```

### Network Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    devmesh-traefik network                       │
│  (external, shared by all projects)                              │
│                                                                  │
│    ┌──────────┐     ┌──────────┐     ┌──────────┐               │
│    │ mx-router│     │   api    │     │  admin   │               │
│    │ (traefik)│     │  :80     │     │  :80     │               │
│    └──────────┘     └────┬─────┘     └────┬─────┘               │
└──────────────────────────┼────────────────┼─────────────────────┘
                           │                │
┌──────────────────────────┼────────────────┼─────────────────────┐
│                          │   default network                     │
│  (implicit, per-project) │                │                      │
│                          │                │                      │
│                     ┌────▼─────┐     ┌────▼─────┐               │
│                     │    db    │     │  redis   │               │
│                     │  :5432   │     │  :6379   │               │
│                     └──────────┘     └──────────┘               │
└─────────────────────────────────────────────────────────────────┘
```

**Key points:**
- Only the app service needs `devmesh-traefik` (it receives external traffic)
- Database, Redis, workers stay on the default network only
- All services in the same compose file can talk via the default network

---

## Dashboard

The Traefik dashboard provides a visual overview of all routers, services, and middlewares.

### Access

```bash
mx router inspect
# Look for "Dashboard : http://localhost:7680"
```

Or find the port in `~/.mech-crate/router/.dashboard-port`.

### Dashboard Features

| Section | Shows |
|---------|-------|
| **HTTP Routers** | All routing rules and their status |
| **HTTP Services** | Backend services and their health |
| **HTTP Middlewares** | Active middleware chains |
| **TCP/UDP** | Non-HTTP routing (if configured) |

### Screenshots

The dashboard shows:
- Green checkmarks for healthy routes
- Red indicators for misconfigured routes
- Request statistics and latency
- Configuration details for each router

---

## HTTPS Support

### Local Development (mkcert)

For local HTTPS, use [mkcert](https://github.com/FiloSottile/mkcert) to generate trusted certificates:

```bash
# Install mkcert (macOS)
brew install mkcert
mkcert -install

# Generate certificates for your domains
cd ~/.mech-crate/router/letsencrypt
mkcert "*.localhost" localhost 127.0.0.1

# This creates:
#   _wildcard.localhost+2.pem (certificate)
#   _wildcard.localhost+2-key.pem (key)
```

Then add a dynamic config file:

```yaml
# ~/.mech-crate/router/config/dynamic/certs.yml
tls:
  certificates:
    - certFile: /etc/traefik/acme/_wildcard.localhost+2.pem
      keyFile: /etc/traefik/acme/_wildcard.localhost+2-key.pem
```

Update your service labels:

```yaml
labels:
  - traefik.http.routers.myapp.entrypoints=websecure
  - traefik.http.routers.myapp.tls=true
```

### Production (Let's Encrypt)

For production with real certificates, add to `traefik.yml`:

```yaml
certificatesResolvers:
  letsencrypt:
    acme:
      email: you@example.com
      storage: /etc/traefik/acme/acme.json
      httpChallenge:
        entryPoint: web
```

Then use in your labels:

```yaml
labels:
  - traefik.http.routers.myapp.tls.certresolver=letsencrypt
```

---

## Dynamic Configuration

Files in `~/.mech-crate/router/config/dynamic/` are hot-reloaded. No restart needed.

### Default Middlewares

```yaml
# ~/.mech-crate/router/config/dynamic/middlewares.yml
http:
  middlewares:
    # Security headers
    default-headers:
      headers:
        frameDeny: true
        contentTypeNosniff: true
        browserXssFilter: true
        referrerPolicy: "strict-origin-when-cross-origin"
    
    # Response compression
    compress-responses:
      compress: {}
    
    # Rate limiting
    rate-limit:
      rateLimit:
        average: 100
        burst: 50
```

### Using Middlewares

Reference middlewares in your service labels:

```yaml
labels:
  - traefik.http.routers.myapp.middlewares=default-headers@file,compress-responses@file
```

### Custom Routes (File Provider)

You can define routes without Docker labels:

```yaml
# ~/.mech-crate/router/config/dynamic/custom-routes.yml
http:
  routers:
    external-api:
      rule: Host(`external.localhost`)
      service: external-api
      entryPoints:
        - web

  services:
    external-api:
      loadBalancer:
        servers:
          - url: http://host.docker.internal:9000
```

This routes `external.localhost` to a service running directly on your host machine.

---

## Troubleshooting

### Service Not Reachable

1. **Is the router running?**
   ```bash
   mx router status
   ```

2. **Is the service on the correct network?**
   ```bash
   docker network inspect devmesh-traefik
   ```
   Look for your container in the output.

3. **Are the labels correct?**
   ```bash
   docker inspect <container> --format '{{json .Config.Labels}}' | jq
   ```

4. **Check router logs:**
   ```bash
   mx router logs
   ```

### 404 Not Found

- Missing `traefik.docker.network=devmesh-traefik` label
- Container not on `devmesh-traefik` network
- Typo in hostname rule

### 502 Bad Gateway

- Container crashed or unhealthy
- Wrong port in `loadbalancer.server.port`
- Service not ready yet (check healthcheck)

### Port Already in Use

```
Error: bind: address already in use
```

Something else is using port 80/443. Find it:
```bash
lsof -i :80
lsof -i :443
```

Common culprits: Apache, Nginx, another Traefik instance.

### Dashboard Not Accessible

The dashboard port is auto-allocated from 7680-7799. Check which port:
```bash
mx router inspect
# or
cat ~/.mech-crate/router/.dashboard-port
```

### Container Not Showing in Dashboard

- `traefik.enable=true` label missing
- Container not on `devmesh-traefik` network
- Container not running

---

## Environment Variables

Customize router behavior with these environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `MX_ROUTER_HOME` | `~/.mech-crate/router` | State directory location |
| `MX_ROUTER_NETWORK` | `devmesh-traefik` | Docker network name |
| `MX_ROUTER_DASHBOARD_PORT` | (auto) | Force specific dashboard port |
| `MX_ROUTER_DASHBOARD_RANGE` | `7680-7799` | Port range for auto-allocation |

### Examples

```bash
# Use a different state directory
MX_ROUTER_HOME=/opt/router mx router install

# Force dashboard to port 9000
MX_ROUTER_DASHBOARD_PORT=9000 mx router up

# Use a different port range
MX_ROUTER_DASHBOARD_RANGE=8800-8899 mx router up
```

### Persistent Configuration

Add to your shell profile (`~/.zshrc`, `~/.bashrc`):

```bash
export MX_ROUTER_DASHBOARD_PORT=7777
```

---

## Advanced Topics

### Running Multiple Routers

Not recommended, but possible with different networks:

```bash
MX_ROUTER_HOME=~/.mech-crate/router-staging \
MX_ROUTER_NETWORK=staging-network \
MX_ROUTER_DASHBOARD_PORT=7700 \
mx router install
```

### Integrating Non-Docker Services

For services running directly on your host:

```yaml
# ~/.mech-crate/router/config/dynamic/host-services.yml
http:
  routers:
    local-app:
      rule: Host(`localapp.localhost`)
      service: local-app
      entryPoints:
        - web
  services:
    local-app:
      loadBalancer:
        servers:
          - url: http://host.docker.internal:3000
```

### Load Balancing

Multiple instances of the same service:

```yaml
services:
  api-1:
    labels:
      - traefik.http.services.api.loadbalancer.server.port=80
  api-2:
    labels:
      - traefik.http.services.api.loadbalancer.server.port=80
```

Both containers register to the same service—Traefik load balances between them.

---

*Part of the MechCrate toolset. Run `mx router help` for command reference.*
