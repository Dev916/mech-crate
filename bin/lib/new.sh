#!/bin/bash
#
# MechCrate New Project Command
# Creates a new MechCrate project
#

# Create new project
create_project() {
    local project_name="$1"
    shift
    
    # Parse additional options
    local with_services=()
    local with_infra=()
    local no_prompt=false
    
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --with)
                shift
                while [[ $# -gt 0 && ! "$1" =~ ^-- ]]; do
                    with_services+=("$1")
                    shift
                done
                ;;
            --infra)
                shift
                while [[ $# -gt 0 && ! "$1" =~ ^-- ]]; do
                    with_infra+=("$1")
                    shift
                done
                ;;
            --no-prompt)
                no_prompt=true
                shift
                ;;
            *)
                shift
                ;;
        esac
    done
    
    if [[ -z "$project_name" ]]; then
        error "Project name is required. Usage: mx new <project-name>"
    fi
    
    if [[ -d "$project_name" ]]; then
        error "Directory '$project_name' already exists."
    fi
    
    raccoon
    info "Creating new MechCrate project: ${BOLD}$project_name${NC}"
    
    # Interactive infrastructure selection (unless --no-prompt or already specified)
    if [[ "$no_prompt" == "false" && ${#with_infra[@]} -eq 0 ]]; then
        echo ""
        echo -e "${BOLD}Infrastructure Options:${NC}"
        echo ""
        
        if prompt_yn "  Include ${CYAN}Cloudflare${NC} Workers + Containers?"; then
            with_infra+=("cloudflare")
        fi
        
        # Future: Add more infrastructure options here
        # if prompt_yn "  Include ${CYAN}AWS${NC} via Terraform?"; then
        #     with_infra+=("aws")
        # fi
        # if prompt_yn "  Include ${CYAN}DigitalOcean${NC} via Terraform?"; then
        #     with_infra+=("digitalocean")
        # fi
        
        echo ""
    fi
    
    # Create project directory
    mkdir -p "$project_name"
    cd "$project_name"
    
    # Create directory structure
    info "Creating directory structure..."
    mkdir -p apps
    mkdir -p make
    mkdir -p scripts
    mkdir -p docker/.config
    mkdir -p docker/compose
    mkdir -p docker/system
    mkdir -p docker/dockerfiles
    mkdir -p tmp/up
    
    # Copy templates
    info "Copying templates..."
    
    # Makefile
    cp "$TEMPLATES_DIR/Makefile.template" "Makefile"
    
    # Make modules
    for mk in "$TEMPLATES_DIR/make/"*.mk; do
        cp "$mk" "make/$(basename "$mk")"
    done
    
    # Scripts (including hidden files like .bashrc)
    shopt -s dotglob
    for script in "$TEMPLATES_DIR/scripts/"*; do
        if [[ -f "$script" ]]; then
            cp "$script" "scripts/$(basename "$script")"
        fi
    done
    shopt -u dotglob
    chmod +x scripts/*.sh 2>/dev/null || true
    
    # Docker config templates (shared only).
    # Intentionally DO NOT scaffold any service-specific docker compose / dockerfiles / system files here.
    # Those should be created by: mx add <service> (or mx add <service> --recipe=<recipe>)
    info "Copying shared Docker config..."
    for config in "$TEMPLATES_DIR/docker/config/"*; do
        [[ -f "$config" ]] || continue
        local filename
        filename=$(basename "$config")
        local target_name=""
        case "$filename" in
            env.shared)
                target_name=".env.shared"
                ;;
            env.secrets.template)
                target_name=".env.secrets.template"
                ;;
            *)
                # Skip service-specific examples like env.app/env.db/env.redis
                continue
                ;;
        esac
        cp "$config" "docker/.config/$target_name"
    done
    
    # ─────────────────────────────────────────────────────────────────────────
    # Infrastructure Setup
    # ─────────────────────────────────────────────────────────────────────────
    
    local has_cloudflare=false
    
    for infra in "${with_infra[@]}"; do
        case "$infra" in
            cloudflare)
                has_cloudflare=true
                info "Setting up Cloudflare infrastructure..."
                mkdir -p infra/cloudflare
                
                # Copy cloudflare templates
                if [[ -d "$TEMPLATES_DIR/infra/cloudflare" ]]; then
                    cp -r "$TEMPLATES_DIR/infra/cloudflare/"* "infra/cloudflare/"
                    
                    # Replace placeholders with project name
                    local project_slug=$(echo "$project_name" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | tr -cd '[:alnum:]-')
                    
                    # Update all template files with project name
                    find "infra/cloudflare" -type f \( -name "*.ts" -o -name "*.toml" -o -name "*.json" -o -name "*.md" \) \
                        -exec sed -i '' "s/{{PROJECT_NAME}}/$project_slug/g" {} \; 2>/dev/null || \
                    find "infra/cloudflare" -type f \( -name "*.ts" -o -name "*.toml" -o -name "*.json" -o -name "*.md" \) \
                        -exec sed -i "s/{{PROJECT_NAME}}/$project_slug/g" {} \; 2>/dev/null || true
                fi
                
                success "Cloudflare infrastructure added"
                ;;
            aws)
                warn "AWS infrastructure not yet implemented - coming soon!"
                ;;
            digitalocean)
                warn "DigitalOcean infrastructure not yet implemented - coming soon!"
                ;;
            *)
                warn "Unknown infrastructure: $infra (skipping)"
                ;;
        esac
    done
    
    # Remove cloudflare.mk if Cloudflare not selected
    if [[ "$has_cloudflare" == "false" && -f "make/cloudflare.mk" ]]; then
        rm "make/cloudflare.mk"
    fi
    
    # Create .gitignore
    cat > .gitignore << 'EOF'
# MechCrate
tmp/
docker/.compose/
docker/.config/.env.secrets
data/

# Dependencies
node_modules/
vendor/
target/

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log
logs/

# Build artifacts
dist/
build/

# Infrastructure secrets
infra/**/.env
infra/**/*.tfvars
infra/**/*.tfstate
infra/**/*.tfstate.*
infra/**/.terraform/
EOF

    # Create README
    cat > README.md << EOF
# $project_name

A MechCrate project.

## Quick Start

\`\`\`bash
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
\`\`\`

## Project Structure

\`\`\`
$project_name/
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
\`\`\`

## Commands

| Command | Description |
|---------|-------------|
| \`make dev\` | Start all services in dev mode |
| \`make dev s=app\` | Start specific service in dev mode |
| \`make up\` | Start services (production mode) |
| \`make down\` | Stop all services |
| \`make logs\` | Tail all logs |
| \`make logs s=app\` | Tail specific service logs |
| \`make sh s=app\` | Shell into service |
| \`make build s=app\` | Build service image |
| \`make restart s=app\` | Restart service |
| \`make ps\` | List running services |

---
🦝 Built with MechCrate
EOF

    # Add services if requested
    for svc in "${with_services[@]}"; do
        add_service_internal "$svc"
    done
    
    # Add Cloudflare section to README if enabled
    if [[ "$has_cloudflare" == "true" ]]; then
        cat >> README.md << 'EOF'

## Cloudflare Deployment

```bash
# Initial setup (run once)
make cf-setup                  # Configure credentials

# Initialize an app
make cf-init a=myapp           # Interactive - choose worker type
make cf-init a=myapp type=worker     # Regular edge worker
make cf-init a=myapp type=cron       # Scheduled worker
make cf-init a=myapp type=container  # Container-backed worker

# Deploy
make cf-deploy a=myapp         # Deploy single app
make cf-deploy-all             # Deploy all apps
```

| Command | Description |
|---------|-------------|
| `make cf-setup` | Interactive setup wizard |
| `make cf-init a=<app>` | Initialize a new app (interactive) |
| `make cf-status` | Show all apps status |
| `make cf-deploy a=<app>` | Deploy to production |
| `make cf-deploy-all` | Deploy all apps |

### Worker Types

| Type | Description |
|------|-------------|
| `worker` | Standard edge worker (APIs, proxies, static sites) |
| `cron` | Scheduled worker (background jobs, data sync) |
| `container` | Docker container backend (SSR apps, stateful services) |

See `infra/cloudflare/README.md` for detailed documentation.
EOF
    fi
    
    success "Project '$project_name' created successfully!"
    echo ""
    
    # ─────────────────────────────────────────────────────────────────────────
    # Interactive Cloudflare App Setup
    # ─────────────────────────────────────────────────────────────────────────
    if [[ "$has_cloudflare" == "true" && "$no_prompt" == "false" ]]; then
        echo ""
        echo -e "${CYAN}╭────────────────────────────────────────────────────────────╮${NC}"
        echo -e "${CYAN}│${NC}  ${BOLD}🌐 Cloudflare Setup${NC}                                      ${CYAN}│${NC}"
        echo -e "${CYAN}╰────────────────────────────────────────────────────────────╯${NC}"
        echo ""
        
        if prompt_yn "Would you like to initialize a Cloudflare Worker app now?"; then
            echo ""
            
            # Run cf-setup first if needed
            if [[ ! -f "infra/cloudflare/.env.cloudflare" ]]; then
                info "First, let's configure your Cloudflare credentials..."
                echo ""
                ./scripts/cf-setup.sh || {
                    warn "Cloudflare setup incomplete. You can run 'make cf-setup' later."
                }
            fi
            
            # Only continue if setup was successful
            if [[ -f "infra/cloudflare/.env.cloudflare" ]]; then
                echo ""
                read -r -p "Enter your app name (e.g., api.example.com, my-worker): " cf_app_name
                
                if [[ -n "$cf_app_name" ]]; then
                    echo ""
                    ./scripts/cf-init-app.sh "$cf_app_name"
                else
                    warn "No app name provided. You can initialize apps later with 'make cf-init a=<app>'"
                fi
            fi
        else
            echo ""
            info "You can initialize Cloudflare apps later:"
            echo "    make cf-setup              # Configure credentials"
            echo "    make cf-init a=myapp       # Initialize an app"
        fi
        
        echo ""
    fi
    
    # ─────────────────────────────────────────────────────────────────────────
    # Final Summary
    # ─────────────────────────────────────────────────────────────────────────
    
    echo -e "${CYAN}╭────────────────────────────────────────────────────────────╮${NC}"
    echo -e "${CYAN}│${NC}  ${BOLD}📦 Project Ready!${NC}                                        ${CYAN}│${NC}"
    echo -e "${CYAN}╰────────────────────────────────────────────────────────────╯${NC}"
    echo ""
    info "Next steps:"
    echo "    cd $project_name"
    echo "    make doctor       # Check dependencies"
    echo "    make dev          # Start development"
    
    if [[ "$has_cloudflare" == "true" ]]; then
        echo ""
        info "Cloudflare commands:"
        echo "    make cf-setup              # Run setup wizard"
        echo "    make cf-init a=myapp       # Initialize an app"
        echo "    make cf-deploy a=myapp     # Deploy to Cloudflare"
    fi
    
    echo ""
    echo -e "${CYAN}🦝 Crate Raccoon says: Your stack is ready!${NC}"
}
