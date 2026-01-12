# Infrastructure Configuration Guide

MechCrate supports hierarchical configuration for infrastructure providers like Cloudflare, DigitalOcean, AWS, and Hetzner. This allows you to set up credentials once globally and reuse them across all projects, while still allowing project-specific overrides when needed.

## Quick Start

```bash
# Set up global credentials (once per workstation)
mx infra setup cloudflare

# Create a new project with Cloudflare infrastructure
mx new myproject --infra cloudflare

# Link the project to your global credentials
cd myproject
mx infra link cloudflare

# Or use project-specific credentials instead
mx cf setup
```

## Hierarchical Config Resolution

When MechCrate needs infrastructure credentials, it looks in this order:

1. **Project-local config** (`./infra/<provider>/.env.<provider>`)
   - Used if it exists AND is not linked to global
   - Takes precedence over global config

2. **Global config** (`~/.mech-crate/config/infra/<provider>.env`)
   - Used if project has no local config, or if project is linked to global
   - Shared across all projects on the workstation

```
┌─────────────────────────────────────────────────────────────────┐
│                    Config Resolution Flow                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Project needs credentials                                       │
│           │                                                      │
│           ▼                                                      │
│  ┌─────────────────────┐                                        │
│  │ Project config      │                                        │
│  │ exists?             │──── No ────┐                           │
│  └─────────────────────┘            │                           │
│           │ Yes                     │                           │
│           ▼                         ▼                           │
│  ┌─────────────────────┐   ┌─────────────────────┐             │
│  │ Is it linked to     │   │ Use global config   │             │
│  │ global?             │   │ if available        │             │
│  └─────────────────────┘   └─────────────────────┘             │
│           │                         │                           │
│    Yes    │    No                   │                           │
│           │                         │                           │
│           ▼                         │                           │
│  ┌─────────────────────┐           │                           │
│  │ Use global config   │           │                           │
│  └─────────────────────┘           │                           │
│                                     │                           │
│           ▼                         │                           │
│  ┌─────────────────────┐           │                           │
│  │ Use project-local   │           │                           │
│  │ config              │           │                           │
│  └─────────────────────┘           │                           │
│                                     ▼                           │
│                          ┌─────────────────────┐               │
│                          │ Error: Not          │               │
│                          │ configured          │               │
│                          └─────────────────────┘               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Supported Providers

| Provider | Description | Config File |
|----------|-------------|-------------|
| `cloudflare` | Cloudflare Workers & Containers | `cloudflare.env` |
| `digitalocean` | DigitalOcean Droplets & App Platform | `digitalocean.env` |
| `aws` | Amazon Web Services | `aws.env` |
| `hetzner` | Hetzner Cloud | `hetzner.env` |

## Commands

### Global Configuration

```bash
# Interactive provider selection
mx infra setup

# Configure a specific provider
mx infra setup cloudflare
mx infra setup digitalocean
mx infra setup aws
mx infra setup hetzner

# List all configured providers
mx infra list

# Show detailed config for a provider
mx infra inspect cloudflare

# Remove a global config
mx infra remove cloudflare
```

### Project Linking

From within a MechCrate project:

```bash
# Link project to global config
mx infra link cloudflare

# Remove link (stop using global config)
mx infra unlink cloudflare

# Check current config status
mx cf config
```

## Configuration Files

### Global Config Location

```
~/.mech-crate/config/infra/
├── cloudflare.env
├── digitalocean.env
├── aws.env
└── hetzner.env
```

### Project Config Location

```
./infra/<provider>/
└── .env.<provider>
```

### Link Marker

When a project is linked to global config, the project config file contains:

```bash
MX_INFRA_USE_GLOBAL=true
MX_INFRA_PROVIDER=cloudflare
MX_INFRA_LINKED_AT=2026-01-11T12:00:00-08:00
```

## Provider-Specific Configuration

### Cloudflare

```bash
# Global setup
mx infra setup cloudflare

# Project-local setup (if needed)
mx cf setup

# Check current config source
mx cf config
```

**Config Variables:**
- `CF_ACCOUNT_ID` - Your Cloudflare account ID
- `CF_DOCKER_PLATFORM` - Docker platform (linux/amd64 or linux/arm64)
- `CLOUDFLARE_API_TOKEN` - API token for CI/CD (optional)

### DigitalOcean

```bash
mx infra setup digitalocean
```

**Config Variables:**
- `DO_API_TOKEN` - Personal Access Token
- `DO_DEFAULT_REGION` - Default region (e.g., nyc3)
- `DO_SPACES_ACCESS_KEY` - Spaces access key (optional)
- `DO_SPACES_SECRET_KEY` - Spaces secret key (optional)
- `DO_SPACES_REGION` - Spaces region (optional)

### AWS

```bash
mx infra setup aws
```

**Config Variables:**
- `AWS_ACCESS_KEY_ID` - AWS access key ID
- `AWS_SECRET_ACCESS_KEY` - AWS secret access key
- `AWS_DEFAULT_REGION` - Default region (e.g., us-east-1)

### Hetzner

```bash
mx infra setup hetzner
```

**Config Variables:**
- `HETZNER_API_TOKEN` - Hetzner Cloud API token
- `HETZNER_DEFAULT_LOCATION` - Default location (e.g., fsn1)

## Workflows

### Shared Team Setup

When multiple team members work on the same project but have different accounts:

```bash
# Each developer sets up their own global config
mx infra setup cloudflare

# Project is configured to use global config by default
cd myproject
mx infra link cloudflare

# Developer B with different account uses project-local instead
mx infra unlink cloudflare
mx cf setup  # Enter different credentials
```

### CI/CD Setup

For CI/CD pipelines, you typically want to use project-local config with secrets from the pipeline:

```bash
# In CI/CD, create the config file directly
cat > infra/cloudflare/.env.cloudflare << EOF
CF_ACCOUNT_ID=$CF_ACCOUNT_ID
CF_DOCKER_PLATFORM=linux/amd64
CLOUDFLARE_API_TOKEN=$CLOUDFLARE_API_TOKEN
EOF
```

Or use environment variables directly (MechCrate will fall back to environment):

```bash
export CF_ACCOUNT_ID=...
export CLOUDFLARE_API_TOKEN=...
mx cf deploy myapp
```

### Multi-Account Setup

For projects that deploy to different accounts (staging vs production):

```bash
# Project structure
myproject/
├── infra/
│   └── cloudflare/
│       ├── .env.cloudflare          # Staging credentials
│       └── .env.cloudflare.prod     # Production credentials

# Deploy to staging (uses default .env.cloudflare)
mx cf deploy myapp

# Deploy to production (override with prod config)
source infra/cloudflare/.env.cloudflare.prod
mx cf deploy myapp
```

## Security Notes

1. **Global configs have 600 permissions** - Only readable by the owner
2. **Config files are gitignored** - Never commit credentials
3. **Secrets are masked** in `mx infra inspect` output
4. **API tokens are optional** - Use `wrangler login` for interactive auth

## Troubleshooting

### Config not found

```bash
# Check what configs exist
mx infra list

# See detailed status for a provider
mx infra inspect cloudflare

# For Cloudflare specifically
mx cf config
```

### Wrong config being used

```bash
# Check which config is active
mx cf config

# If linked but want local
mx infra unlink cloudflare

# If local but want global
mx infra link cloudflare
```

### Reset configuration

```bash
# Remove global config
mx infra remove cloudflare

# Remove project link
rm ./infra/cloudflare/.env.cloudflare

# Start fresh
mx infra setup cloudflare
```

---

*Part of the MechCrate documentation*
