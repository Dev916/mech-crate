---
title: "Infrastructure Configuration Guide"
category: infra
languages: []
complexity: intermediate
use_cases:
  - "setting up global infrastructure credentials"
  - "linking projects to shared credentials"
  - "overriding configuration per project"
  - "configuring Cloudflare, AWS, or Hetzner providers"
summary: "How MechCrate resolves hierarchical infrastructure credentials for Cloudflare, DigitalOcean, AWS, and Hetzner across global and project scopes."
---

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

`mx cf <cmd>` is a thin wrapper over the project's `make cf-<cmd>` targets
(`make/cloudflare.mk`, present when the project was created with
`--infra cloudflare`): `mx cf deploy myapp` runs exactly
`make cf-deploy a=myapp`. `mx cf --help` lists every wrapped target.

## Hierarchical Config Resolution

Every credential resolves on its own, highest precedence first:

1. **The environment** (or the `make` command line)
2. **Project-local config** (`./infra/<provider>/.env.<provider>`), unless the
   project is linked to global
3. **Global config** (`~/.mech-crate/config/infra/<provider>.env`), shared by
   every project on the workstation

Within a scope the canonical name is read before a deprecated alias
(`CLOUDFLARE_ACCOUNT_ID` before `CF_ACCOUNT_ID`), but precedence is by scope,
never by spelling: a project file carrying only the alias still beats a global
file carrying the canonical name. This is the order `make/cloudflare.mk`
implements, and `mx infra inspect <provider>` prints the resolved set with the
scope that supplied each key, so the two cannot disagree.

### Linking

A project is **linked** when its `.env.<provider>` contains the exact line
`MX_INFRA_USE_GLOBAL=true`. A linked project's file is not read for values, so
its credentials resolve from the environment, then the global config. That one
line is the whole mechanism, and every reader honors it: `mx infra inspect`,
`make cf-vars` / `mx cf vars` (which print `CF_PROJECT_LINKED=true`), every
`make cf-*` target, and `scripts/cf-init-app.sh`.

- `mx infra link <provider>` adds the line (creating the file if needed) and
  keeps everything else in the file. It needs a project (run it from the root
  or below) and an existing global config, and is idempotent.
- `mx infra unlink <provider>` removes the line, so the project's own values
  apply again. A file that held nothing but the link is deleted. Unlinking an
  unlinked project is a no-op.

Linking is only needed to make a project with its own `.env.<provider>` defer
to the global config without deleting that file: a project with no project
file already falls through to global.

The older `.env.linked` marker file is **deprecated and ignored**. It was only
ever read by the Rust resolver and written by nothing. `mx infra inspect` warns
about a stray one, and `mx infra link` / `unlink` delete it.

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

# Show the resolved config for a provider: each key with the scope that
# supplied it (environment / project / global, deprecated aliases tagged),
# the link state, and both file paths. Secrets are masked.
mx infra inspect cloudflare

# Remove a global config
mx infra remove cloudflare
```

### Project Linking

From within a MechCrate project:

```bash
# Link project to global config
mx infra link cloudflare

# Remove link (the project's own values apply again)
mx infra unlink cloudflare

# Check current config status (alias of mx cf vars)
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

A linked project's config file contains the line `mx infra link` writes:

```bash
# Linked to the global config by `mx infra link`; `mx infra unlink` removes this.
MX_INFRA_USE_GLOBAL=true
```

Any other lines in the file are kept, and ignored while the link is in place.

## Provider-Specific Configuration

### Cloudflare

```bash
# Global setup
mx infra setup cloudflare

# Project-local setup (if needed)
mx cf setup

# Show the resolved credentials and which scope supplied them (make cf-vars)
mx cf vars

# Fail early if no account id resolves (make cf-check-credentials)
mx cf check-credentials

# Initialize and deploy an app (make cf-init / make cf-deploy)
mx cf init api.example.com --type=container
mx cf deploy api.example.com
```

**Config Variables:**
- `CLOUDFLARE_ACCOUNT_ID` - Your Cloudflare account ID
- `CF_DOCKER_PLATFORM` - Docker platform (linux/amd64 or linux/arm64)
- `CLOUDFLARE_API_TOKEN` - API token for CI/CD (optional)

The two credential names are wrangler's own, so the value in a config file
reaches the deploy untranslated. `CF_ACCOUNT_ID` / `CF_API_TOKEN` are deprecated
aliases: still read at their own scope's precedence so an older config keeps
working, never written. For Cloudflare specifically, both scopes are read with no
linking required, the project file winning over the global one; `make cf-vars`
(`mx cf vars`) prints which scope supplied the account id and
`make cf-check-credentials` (`mx cf check-credentials`) fails loudly when none
did. See `mx-cloudflare-deploy.md` §5 for the as-implemented detail.

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
CLOUDFLARE_ACCOUNT_ID=$CLOUDFLARE_ACCOUNT_ID
CF_DOCKER_PLATFORM=linux/amd64
CLOUDFLARE_API_TOKEN=$CLOUDFLARE_API_TOKEN
EOF
```

Or use environment variables directly. For Cloudflare the environment is the
highest-precedence scope, above both config files:

```bash
export CLOUDFLARE_ACCOUNT_ID=...
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

# Deploy to production (export the prod config; the environment wins)
set -a; source infra/cloudflare/.env.cloudflare.prod; set +a
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
mx cf vars

# If linked but want local
mx infra unlink cloudflare

# If local but want global
mx infra link cloudflare
```

### Reset configuration

```bash
# Remove global config
mx infra remove cloudflare

# Remove project link (or delete the project file to drop its values too)
mx infra unlink cloudflare

# Start fresh
mx infra setup cloudflare
```

---

*Part of the MechCrate documentation*
