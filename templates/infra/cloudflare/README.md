# Cloudflare Infrastructure

Multi-app deployment infrastructure for Cloudflare Workers + Containers.

## Directory Structure

```
infra/cloudflare/
├── .env.cloudflare          # Credentials (gitignored)
├── apps/
│   ├── pricelove.co/        # App: pricelove.co
│   │   ├── src/index.ts
│   │   ├── wrangler.toml
│   │   ├── package.json
│   │   └── tsconfig.json
│   └── theblock.co/         # App: theblock.co
│       └── ...
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
make cf-init a=pricelove.co
```

This creates:
- `infra/cloudflare/apps/pricelove.co/` - Worker code
- `docker/dockerfiles/pricelove.co/app` - Dockerfile
- `docker/.config/.env.pricelove_co` - Environment config

### 3. Deploy

```bash
# Deploy single app
make cf-deploy a=pricelove.co

# Deploy all apps
make cf-deploy-all
```

## Commands Reference

### Setup & Status

| Command | Description |
|---------|-------------|
| `make cf-setup` | Interactive setup wizard |
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

The `.env.cloudflare` file (created by `cf-setup`):

```bash
# Required
CF_ACCOUNT_ID=your_account_id

# Optional (for CI/CD)
CLOUDFLARE_API_TOKEN=your_api_token

# Build options
CF_DOCKER_PLATFORM=linux/amd64
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
          CF_ACCOUNT_ID: ${{ secrets.CF_ACCOUNT_ID }}
        run: make cf-deploy-all
```

### Required Secrets

Create in your repo settings:

| Secret | Description |
|--------|-------------|
| `CLOUDFLARE_API_TOKEN` | API token with Workers & Registry permissions |
| `CF_ACCOUNT_ID` | Your Cloudflare account ID |

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
