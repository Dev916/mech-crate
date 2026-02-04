<p align="center">
  <img src="assets/mechcrate-logo.png" alt="MechCrate Logo" width="200">
</p>

# MechCrate (mx)

🦝 **Crate Raccoon is the mascot.** MechCrate is the project scaffolding kit for Docker-based development.

## Installation

### Quick Install

```bash
# Clone the repo
git clone https://github.com/your-org/mech-crate.git
cd mech-crate

# Build and install globally
make install-local    # Installs to ~/.local/bin (no sudo)
# or
make install          # Installs to /usr/local/bin (may need sudo)
```

### Manual Build

```bash
# Build release binaries
cargo build --release

# Initialize MechCrate templates
MECH_CRATE_ROOT=$(pwd) ./target/release/mx init

# Add to PATH or copy to a directory in your PATH
cp target/release/mx ~/.local/bin/
```

### Verify Installation

```bash
mx --version
mx doctor
```

## Quick Start

```bash
# Start the global router (first-time only)
mx router install
mx router up

# Create a new project
mx new my-app

# Enter the project and start developing
cd my-app
make doctor    # Check dependencies
make init      # Initialize environment
make dev       # Start development (access via hostname!)
```

## What MechCrate Is

MechCrate is a reusable set of Docker and Docker Compose conventions plus Makefile modules plus scripts that let you drop an app into a known structure and start building immediately.

It standardizes:
* **Local development** - Docker Compose with dev overrides
* **Environment config** - `.env` files split per service
* **Filesystem layout** - 1:1 host-to-container mapping
* **Make-based CLI** - Consistent commands across all projects

## The mx CLI Tool

The `mx` CLI is written in Rust for performance and reliability.

### Global Commands

```bash
mx init              # Initialize MechCrate (~/.mech-crate)
mx doctor            # Check system health
mx recipes list      # List available recipes
mx recipes info <name>  # Show recipe details

mx router install    # First-time setup
mx router up         # Start the global router
mx router down       # Stop the router
mx router status     # Check router status
mx router inspect    # Show dashboard URL and connected services

mx mcp build         # Build MCP server for AI integration
mx mcp start         # Start Weaviate RAG backend
mx mcp status        # Check MCP status

mx infra list        # List infrastructure providers
mx infra setup <provider>  # Configure provider credentials
```

### Project Commands

```bash
mx new <name>        # Create a new MechCrate project
mx new <name> --with api  # With a specific service
mx add <service>     # Add a new service to existing project
mx add api --recipe rust-api  # Add with a specific recipe
mx upgrade           # Update project with latest scaffolding
mx upgrade --diff    # Show diffs before updating
```

### Make Commands (in project)

```bash
make dev             # Start services in development mode
make up              # Start services (production mode)
make down            # Stop all services
make logs            # Tail all logs
make logs s=app      # Tail specific service logs
make sh s=app        # Shell into service
make build s=app     # Build service image
make restart s=app   # Restart service
make ps              # List running services
make doctor          # Check project health
make help            # Show all commands
```

## Available Recipes

| Recipe | Description |
|--------|-------------|
| `astro` | Full-stack Astro 5 with Vue 3 islands, SSR |
| `laravel` | Laravel 12 + Octane (Swoole) with Filament admin |
| `nuxt` | Nuxt 3 SSR/SSG application with Nitro server |
| `rust-api` | Rust API service with Actix-web, SQLx |
| `rust-leptos` | Leptos SSR + Actix-web with shadcn-ui |
| `rust-worker` | High-performance job worker with Redis |
| `zola` | Zola static site generator |

## Non-Negotiable Folder Contract

If a repo uses MechCrate, the structure must exist exactly like this:

```
project-root/
├── Makefile                          # Root makefile
├── make/                             # Make modules
│   ├── common.mk
│   ├── dev.mk
│   └── ...
├── scripts/                          # Shell scripts
│   └── ...
├── apps/                             # Application source code
│   └── <service>/
└── docker/
    ├── .config/                      # Environment files
    │   ├── .env.shared
    │   ├── .env.secrets
    │   └── .env.<service>
    ├── compose/                      # Compose files
    │   ├── <service>.yml
    │   └── <service>.dev.yml
    ├── system/                       # 1:1 filesystem mounts
    │   └── <service>/
    └── dockerfiles/                  # Dockerfiles
        └── <service>/
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
- **Production**: `service.yml` only (baseline)
- **Development**: `service.yml` + `service.dev.yml` (baseline + overrides)

Development overrides add:
- Volume mounts for hot-reload
- Debug ports
- Development-only environment variables
- Disabled health checks

## Documentation

| Guide | Description |
|-------|-------------|
| [Router Guide](docs/router.md) | Global Traefik reverse proxy setup |
| [Recipe Authoring](docs/development/RECIPE_AUTHORING_GUIDE.md) | Create custom service recipes |
| [Rust CLI Development](docs/development/RUST_CLI_DEVELOPMENT.md) | Develop the mx CLI |
| [Quick Reference](docs/development/QUICK_REFERENCE.md) | Common commands cheatsheet |

## Development

### Project Structure

```
mech-crate/
├── Cargo.toml              # Workspace manifest
├── crates/
│   ├── mx-lib/             # Shared library (core logic)
│   ├── mx-cli/             # CLI binary (the `mx` command)
│   └── mx-mcp-server/      # MCP server for AI agents
├── templates/              # Recipe templates
│   ├── project/            # Base project structure
│   ├── recipes/            # Service recipes
│   └── router/             # Global router
└── docs/                   # Documentation
```

### Building

```bash
# Build debug
make build

# Build release
make build-release

# Run tests
make test

# Run linter
make lint

# Install locally
make install-local
```

---

🦝 **Crate Raccoon says: Happy building!**
