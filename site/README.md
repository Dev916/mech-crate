# site

A MechCrate project.

## Quick Start

```bash
# Check dependencies
make doctor

# Add a service (pick a recipe, or use the default template)
mx add api --recipe=nuxt

# Start development
make dev

# View logs
make logs

# Stop services
make down
```

## Project Structure

```
site/
├── Makefile              # Root makefile
├── apps/                 # Application source code
│   └── <service>/        # Each service's source
│       ├── src/          # Source code
│       ├── package.json  # Dependencies
│       └── ...
├── make/                 # Make modules
│   ├── common.mk         # Shared helpers
│   ├── dev.mk            # Development commands
│   ├── up.mk             # Service management
│   └── ...
├── scripts/              # Shell scripts
│   ├── .bashrc           # Helper functions
│   ├── dev.sh            # Development script
│   └── ...
└── docker/
    ├── .config/          # Environment files
    │   ├── .env.shared   # Shared config
    │   ├── .env.secrets.template  # Secrets template (gitignored secrets created on init)
    │   └── .env.<svc>             # Per-service config (created by mx add)
    ├── compose/          # Compose files
    │   └── <service>.yml / <service>.dev.yml  # Created by mx add / recipes
    ├── system/           # System-level files (configs, etc.)
    │   └── <service>/    # Maps to container /
    │       ├── etc/      # Config files
    │       └── var/      # Log directories
    └── dockerfiles/      # Dockerfiles
        └── <service>/
            └── app       # Dockerfile
```

## Commands

| Command | Description |
|---------|-------------|
| `make dev` | Start all services in dev mode |
| `make dev s=app` | Start specific service in dev mode |
| `make up` | Start services (production mode) |
| `make down` | Stop all services |
| `make logs` | Tail all logs |
| `make logs s=app` | Tail specific service logs |
| `make sh s=app` | Shell into service |
| `make build s=app` | Build service image |
| `make restart s=app` | Restart service |
| `make ps` | List running services |

---
🦝 Built with MechCrate
