# Cloudflare Infrastructure

Multi-app deployment infrastructure for Cloudflare Workers.

## Supported Worker Types

| Type | Description | Best For |
|------|-------------|----------|
| **worker** | Standard edge worker | APIs, proxies, static sites |
| **cron** | Scheduled worker | Background jobs, data sync |
| **container** | Docker container backend | SSR apps, stateful services |

## Directory Structure

```
infra/cloudflare/
├── .env.cloudflare          # Credentials (gitignored)
├── apps/
│   ├── api.example.com/     # Regular Worker
│   ├── sync-job/            # Cron Worker  
│   └── app.example.com/     # Container Worker
└── README.md
```

## Quick Start

### 1. Setup Credentials

```bash
make cf-setup
```

This interactive wizard will:
- Authenticate with Cloudflare (if not already)
- Save your Account ID
- Optionally configure an API token for CI/CD

### 2. Initialize an App

```bash
# Interactive mode (choose type)
make cf-init a=myapp

# Or specify type directly
make cf-init a=myapp type=worker      # Regular worker
make cf-init a=myapp type=cron        # Cron worker  
make cf-init a=myapp type=container   # Container worker
```

This creates:
- `infra/cloudflare/apps/<app>/` - Worker code
- `docker/dockerfiles/<app>/app` - Dockerfile (container only)
- `docker/.config/.env.<app>` - Environment config

### 3. Deploy

```bash
# Deploy single app
make cf-deploy a=myapp

# Deploy all apps
make cf-deploy-all
```

## Commands Reference

### Setup & Status

| Command | Description |
|---------|-------------|
| `make cf-setup` | Interactive setup wizard |
| `make cf-vars` | Show the resolved credentials and which scope supplied them |
| `make cf-check-credentials` | Fail early, naming both config paths, when no account id resolves |
| `make cf-login` | Authenticate with Cloudflare |
| `make cf-whoami` | Show current authentication |
| `make cf-status` | Show all apps status |
| `make cf-list` | List all configured apps |

### App Management

| Command | Description |
|---------|-------------|
| `make cf-init a=<app>` | Initialize a new app |
| `make cf-install a=<app>` | Install worker dependencies |
| `make cf-dev a=<app>` | Run worker locally |

### Build & Deploy

| Command | Description |
|---------|-------------|
| `make cf-build a=<app>` | Build container image |
| `make cf-push a=<app>` | Push to Cloudflare registry |
| `make cf-publish a=<app>` | Build and push |
| `make cf-deploy a=<app>` | Full deploy to production |
| `make cf-deploy-preview a=<app>` | Deploy to preview |
| `make cf-deploy-all` | Deploy all apps |

### Monitoring

| Command | Description |
|---------|-------------|
| `make cf-logs a=<app>` | Tail production logs |
| `make cf-logs-preview a=<app>` | Tail preview logs |
| `make cf-images a=<app>` | List registry images |

## Worker Endpoints

Each deployed app exposes these endpoints:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/_health` | GET | Health check |
| `/_healthz` | GET | Kubernetes-style health |
| `/_readyz` | GET | Readiness check |
| `/_container/status` | GET | Container state & metadata |
| `/_container/start` | POST | Start container |
| `/_container/stop` | POST | Graceful stop |
| `/_container/restart` | POST | Restart container |

### Example

```bash
# Check health
curl https://pricelove.co/_health

# Get container status
curl https://pricelove.co/_container/status

# Restart container
curl -X POST https://pricelove.co/_container/restart
```

## Configuration

### Environment Variables

Credentials use the two names wrangler itself reads, `CLOUDFLARE_ACCOUNT_ID` and
`CLOUDFLARE_API_TOKEN`, so the value in a file reaches the deploy untranslated.
They resolve from two scopes, highest precedence first: the environment (or the
`make` command line), then the project file, then the global file.

| Scope | File | Written by |
|-------|------|------------|
| Project | `infra/cloudflare/.env.cloudflare` | `make cf-setup` |
| Global | `~/.mech-crate/config/infra/cloudflare.env` | `mx infra setup cloudflare` |

The `.env.cloudflare` file (created by `cf-setup`, gitignored):

```bash
# Required
CLOUDFLARE_ACCOUNT_ID=your_account_id

# Optional (for CI/CD)
CLOUDFLARE_API_TOKEN=your_api_token

# Build options
CF_DOCKER_PLATFORM=linux/amd64
```

A global config alone is a complete setup: `make cf-init` and the deploy targets
resolve from it, and a project file is only needed when this project deploys to a
different account than the rest of the workstation.

`CF_ACCOUNT_ID` and `CF_API_TOKEN` are **deprecated aliases**. They are still
read, at their own scope's precedence, so a file written by an older `cf-setup`
keeps working; nothing writes them any more, and the toolchain names the file
that still uses them.

Two targets make the resolution visible instead of guessable:

```bash
make cf-vars              # resolved account id, which scope supplied it, both file paths
make cf-check-credentials # fails loudly, naming both paths and both fixes, when nothing resolves
```

### Per-App Configuration

Each app has its own `wrangler.toml` with:

- **Preview environment**: Deployed to `*.workers.dev`
- **Production environment**: Deployed to your domain

Edit `infra/cloudflare/apps/<app>/wrangler.toml` to customize:

```toml
# Adjust container limits
[[env.production.containers]]
max_instances = 10
min_instances = 1

# Add custom routes
[env.production]
routes = [
  { pattern = "pricelove.co/*", zone_name = "pricelove.co" },
  { pattern = "www.pricelove.co/*", zone_name = "pricelove.co" }
]
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Deploy to Cloudflare
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          
      - name: Install dependencies
        run: npm ci
        
      - name: Deploy
        env:
          CLOUDFLARE_API_TOKEN: ${{ secrets.CLOUDFLARE_API_TOKEN }}
          CLOUDFLARE_ACCOUNT_ID: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
        run: make cf-deploy-all
```

CI needs no credentials file: the environment is the highest-precedence scope, so
those two variables are enough on their own.

### Required Secrets

Create in your repo settings:

| Secret | Description |
|--------|-------------|
| `CLOUDFLARE_API_TOKEN` | API token with Workers & Registry permissions |
| `CLOUDFLARE_ACCOUNT_ID` | Your Cloudflare account ID |

## Adding Multiple Apps

```bash
# Initialize apps
make cf-init a=pricelove.co
make cf-init a=theblock.co
make cf-init a=myapp.co

# Deploy all
make cf-deploy-all

# Or deploy individually
make cf-deploy a=pricelove.co
make cf-deploy a=theblock.co
```

## Versioning

Container images are tagged from each app's `package.json`:

```json
{
  "name": "pricelove.co-worker",
  "version": "1.2.3"
}
```

Results in: `registry.cloudflare.com/<account>/pricelove.co:v1.2.3`

To update version and deploy:

```bash
cd infra/cloudflare/apps/pricelove.co
npm version patch  # or minor, major
cd -
make cf-deploy a=pricelove.co
```

## Troubleshooting

### Container won't start

```bash
# Check status
curl https://pricelove.co/_container/status

# View logs
make cf-logs a=pricelove.co

# Force restart
curl -X POST https://pricelove.co/_container/restart
```

### Image not found

```bash
# List available images
make cf-images a=pricelove.co

# Rebuild and push
make cf-publish a=pricelove.co

# Sync wrangler.toml with new tag
make cf-sync-image a=pricelove.co
```

### Authentication issues

```bash
# Re-login
make cf-login

# Verify identity
make cf-whoami

# Re-run setup
make cf-setup
```

---
🦝 Built with MechCrate
