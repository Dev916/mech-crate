<p align="center">
  <img src="assets/mechcrate-logo.png" alt="MechCrate Logo" width="200">
</p>

# MechCrate (mx)

🦝 **Crate Raccoon is the mascot.** MechCrate is the project scaffolding kit for Docker-based development.

## Quick Start

```bash
# Install mx (add to your PATH)
export PATH="$PATH:/path/to/mech-crate/bin"

# Create a new project
mx new my-app

# Enter the project and start developing
cd my-app
make doctor    # Check dependencies
make init      # Initialize environment
make dev       # Start development
```

## What MechCrate Is

MechCrate is a reusable set of Docker and Docker Compose conventions plus Makefile modules plus scripts that let you drop an app into a known structure and start building immediately.

It standardizes:
* **Local development** - Docker Compose with dev overrides
* **Environment config** - `.env` files split per service
* **Filesystem layout** - 1:1 host-to-container mapping
* **Make-based CLI** - Consistent commands across all projects

## The mx CLI Tool

```bash
mx new <name>        # Create a new MechCrate project
mx add <service>     # Add a new service to existing project
mx doctor            # Check project health and dependencies
mx help              # Show all commands

# Project commands (run from project root)
mx dev [service]     # Start services in development mode
mx up [service]      # Start services in production mode
mx down [service]    # Stop services
mx logs [service]    # Tail service logs
mx build <service>   # Build a service image
mx restart <service> # Restart a service
mx sh <service>      # Shell into a service
mx ps                # List running services
```

## Non-Negotiable Folder Contract

If a repo uses MechCrate, the structure must exist exactly like this:

```
project-root/
├── Makefile                          # Root makefile
├── make/                             # Make modules
│   ├── common.mk                     # Shared helpers
│   ├── dev.mk                        # Development commands
│   ├── up.mk                         # Service management
│   ├── down.mk
│   ├── build.mk
│   ├── logs.mk
│   ├── restart.mk
│   ├── sh.mk
│   ├── run.mk
│   ├── start.mk
│   └── stop.mk
├── scripts/                          # Shell scripts
│   ├── .bashrc                       # Helper functions
│   ├── dev.sh
│   ├── up.sh
│   ├── down.sh
│   ├── build.sh
│   ├── logs.sh
│   ├── restart.sh
│   ├── sh.sh
│   ├── run.sh
│   ├── exec.sh
│   ├── start.sh
│   ├── stop.sh
│   ├── ps.sh
│   ├── init.sh
│   ├── test.sh
│   ├── doctor.sh
│   └── help.sh
└── docker/
    ├── .config/                      # Environment files
    │   ├── .env.shared               # Shared across all services
    │   ├── .env.secrets              # Credentials (gitignored)
    │   ├── .env.secrets.template     # Template for secrets
    │   ├── .env.app                  # Per-service config
    │   ├── .env.db
    │   └── .env.redis
    ├── compose/                      # Compose files (atomic)
    │   ├── app.yml                   # Baseline (production)
    │   ├── app.dev.yml               # Development overrides
    │   ├── db.yml
    │   ├── db.dev.yml
    │   ├── redis.yml
    │   └── redis.dev.yml
    ├── system/                       # 1:1 filesystem mounts
    │   ├── app/
    │   │   ├── app/                  # → /app
    │   │   ├── etc/app/              # → /etc/app
    │   │   └── var/log/app/          # → /var/log/app
    │   └── postgres/
    │       └── docker-entrypoint-initdb.d/
    └── dockerfiles/                  # Dockerfiles
        └── app/
            └── app                   # Multi-stage Dockerfile
```

## Environment Config Rules

MechCrate uses centralized env files loaded in a consistent order:

1. `docker/.config/.env.shared` - Shared across all services
2. `docker/.config/.env.secrets` - Credentials (gitignored)
3. `docker/.config/.env.<service>` - Service-specific config

## Compose Rules

### Atomic Service Files
Each service is defined in its own compose file inside `docker/compose/`. This lets you compose any stack you want by passing multiple `-f` files.

### Baseline + Dev Override Pattern
- **Production**: `app.yml` only (baseline)
- **Development**: `app.yml` + `app.dev.yml` (baseline + overrides)

Development overrides add:
- Volume mounts for hot-reload
- Debug ports
- Development-only environment variables
- Disabled health checks

## 1:1 Filesystem Mirroring Rule

Host directory `docker/system/<service>/path/to/file` maps to container path `/path/to/file`.

This makes Dockerfiles clean:
```dockerfile
COPY docker/system/app/ /
COPY docker/system/nginx/ /
COPY docker/system/postgres/ /
```

## Make Commands

| Command | Description |
|---------|-------------|
| `make dev` | Start all services in dev mode |
| `make dev s=app` | Start specific service in dev mode |
| `make up` | Start services (production mode) |
| `make up s=app` | Start specific service |
| `make down` | Stop all services |
| `make down s=app` | Stop specific service |
| `make logs` | Tail all logs |
| `make logs s=app` | Tail specific service logs |
| `make sh s=app` | Shell into service |
| `make build s=app` | Build service image |
| `make build s=app t=v1.0` | Build with specific tag |
| `make restart s=app` | Restart service |
| `make ps` | List running services |
| `make doctor` | Check project health |
| `make init` | Initialize environment |
| `make help` | Show all commands |

## Adding a New Service

Using mx:
```bash
mx add api
```

Or manually:
1. Add `docker/compose/<service>.yml`
2. Add `docker/compose/<service>.dev.yml` (dev overrides)
3. Add `docker/.config/.env.<service>` (config)
4. Add `docker/system/<service>/...` (filesystem content)
5. Add `docker/dockerfiles/<service>/app` (Dockerfile)

## Project Structure Reference

See `reference/mono/` for a complete working example with multiple services.

## Development

The MechCrate tool lives in:
- `bin/mx` - CLI tool
- `templates/` - Project templates

---

🦝 **Crate Raccoon says: Happy building!**
