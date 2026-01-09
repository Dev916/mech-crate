# MechCrate Recipes

Data-driven recipe system for scaffolding application stacks.

## Architecture

Each recipe is defined by **data (JSON)** + **templates (files)**:

```
recipes/
├── bin/lib/recipe.sh          # Unified engine (in bin/lib/)
├── laravel/
│   ├── recipe.json            # Metadata, options, steps
│   ├── app/                   # App template files
│   ├── docker/                # Docker templates
│   └── config/                # Config templates
├── nuxt/
│   ├── recipe.json
│   ├── app/
│   ├── docker/
│   └── config/
└── rust-api/
    ├── recipe.json
    ├── app/
    ├── docker/
    └── config/
```

## recipe.json Schema

```json
{
  "name": "recipe-name",
  "title": "Human Title",
  "description": "Short description",
  "version": "1.0",
  
  "features": ["Feature 1", "Feature 2"],
  
  "services": [
    { "name": "<name>", "description": "Main service" }
  ],
  
  "options": {
    "domain": {
      "flag": "--domain",
      "default": "{{SERVICE_NAME}}.localhost",
      "description": "Custom domain"
    }
  },
  
  "placeholders": {
    "SERVICE_NAME": { "source": "name" },
    "SERVICE_SLUG": { "source": "name", "transform": "slug" },
    "DOMAIN": { "source": "option:domain" }
  },
  
  "directories": [
    "apps/{{SERVICE_NAME}}/src"
  ],
  
  "templates": [
    { "from": "app", "to": "apps/{{SERVICE_NAME}}" },
    { "from": "docker/compose/service.yml", "to": "docker/compose/{{SERVICE_NAME}}.yml" }
  ],
  
  "post_install": {
    "renames": [{ "from": "...", "to": "..." }],
    "chmod": [{ "path": "...", "mode": "+x" }],
    "gitkeep": ["apps/{{SERVICE_NAME}}/storage"]
  },
  
  "next_steps": [
    "cd apps/{{SERVICE_NAME}} && npm install"
  ],
  
  "notes": ["Optional notes shown after install"]
}
```

## Placeholders

Use `{{PLACEHOLDER}}` in any template file:

| Placeholder | Source | Transform | Example |
|------------|--------|-----------|---------|
| `{{SERVICE_NAME}}` | name | - | `myapp` |
| `{{SERVICE_SLUG}}` | name | slug | `myapp` |
| `{{SERVICE_UPPER}}` | name | upper | `MYAPP` |
| `{{DOMAIN}}` | option:domain | - | `myapp.localhost` |

### Transforms

- `slug` - lowercase, alphanumeric + hyphens
- `upper` - uppercase, underscores for non-alphanumeric
- `rust_crate` - lowercase, underscores (valid Rust crate name)
- `ssr_bool` - "spa" → "false", else "true"

## Available Recipes

| Recipe | Description |
|--------|-------------|
| `astro` | Astro 4 + Vue 3 SSR + shadcn-vue + PrimeVue + Pinia + PostgreSQL + Redis |
| `laravel` | Laravel 12 + Octane (Swoole) + Filament + Inertia |
| `nuxt` | Nuxt 3 SSR/SSG + Tailwind |
| `rust-api` | Actix-web + SQLx (API only) |
| `rust-leptos` | Leptos SSR + Actix + shadcn-ui + PostgreSQL + Redis |
| `rust-worker` | Job worker with Redis pub/sub, PostgreSQL, LLM support |

## Usage

```bash
# List recipes
mx recipes

# Get recipe info
mx recipes info laravel

# Install a recipe
mx add myapp --recipe=laravel
mx add myapp --recipe=laravel --domain=myapp.com
mx add myapi --recipe=rust-api --port=8080
```

## Creating a New Recipe

1. Create recipe directory:
   ```bash
   mkdir -p templates/recipes/myrecipe/{app,docker,config}
   ```

2. Create `recipe.json` with metadata

3. Add template files with `{{PLACEHOLDER}}` syntax

4. Test:
   ```bash
   mx add test-service --recipe=myrecipe
   ```
