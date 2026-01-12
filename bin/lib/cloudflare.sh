#!/bin/bash
#
# MechCrate Cloudflare Commands
# Cloudflare Workers & Containers management
#

# ─────────────────────────────────────────────────────────────────────────────
# Cloudflare Config Resolution
# ─────────────────────────────────────────────────────────────────────────────

# Load Cloudflare config using hierarchical resolution
# Returns 0 if config loaded, 1 if no config found
_cf_load_config() {
    # Try hierarchical resolution first (project → global)
    if infra_load_config "cloudflare" 2>/dev/null; then
        return 0
    fi
    
    # Fall back to legacy project-local path for backwards compatibility
    local legacy_config="./infra/cloudflare/.env.cloudflare"
    if [[ -f "$legacy_config" ]]; then
        # shellcheck source=/dev/null
        source "$legacy_config"
        return 0
    fi
    
    return 1
}

# Check if Cloudflare is configured (either globally or project-local)
_cf_is_configured() {
    # Check via hierarchical resolution
    if infra_resolve_config "cloudflare" &>/dev/null; then
        return 0
    fi
    
    # Check legacy path
    [[ -f "./infra/cloudflare/.env.cloudflare" ]]
}

# Get the config source description for user messages
_cf_config_source() {
    local resolved
    resolved="$(infra_resolve_config "cloudflare" 2>/dev/null || echo "")"
    
    if [[ -z "$resolved" ]]; then
        echo "not configured"
    elif [[ "$resolved" == *"/.mech-crate/config/"* ]]; then
        echo "global (~/.mech-crate/config/infra/cloudflare.env)"
    else
        echo "project-local ($resolved)"
    fi
}

# Setup command - handles both project-local and global config awareness
_cf_setup_cmd() {
    echo -e "${CYAN}"
    cat << 'EOF'
    ╭──────────────────────────────────────────────────────────╮
    │  ☁️  Cloudflare Setup                                     │
    ╰──────────────────────────────────────────────────────────╯
EOF
    echo -e "${NC}"
    
    # Check current config status
    local current_source
    current_source="$(_cf_config_source)"
    
    if [[ "$current_source" != "not configured" ]]; then
        echo -e "Current configuration: ${BOLD}${current_source}${NC}"
        echo ""
    fi
    
    # Check if global config exists
    if _infra_has_global_config "cloudflare"; then
        echo -e "${BOLD}Options:${NC}"
        echo "  1) Link to global config (recommended for shared credentials)"
        echo "  2) Set up project-local credentials (for different account/API key)"
        echo ""
        read -r -p "Choose [1/2]: " choice
        echo ""
        
        case "$choice" in
            1)
                _infra_link "cloudflare"
                return 0
                ;;
            2)
                # Continue with project-local setup
                ;;
            *)
                warn "Invalid choice. Defaulting to project-local setup."
                ;;
        esac
    fi
    
    # Run the project-local setup script
    ./scripts/cf-setup.sh
}

# Show config status
_cf_config_cmd() {
    echo -e "${BOLD}Cloudflare Configuration Status${NC}"
    echo ""
    
    local source
    source="$(_cf_config_source)"
    echo -e "  Active config: ${CYAN}${source}${NC}"
    echo ""
    
    if _cf_is_configured; then
        _cf_load_config
        echo -e "  ${GREEN}●${NC} Account ID: ${CF_ACCOUNT_ID:-not set}"
        echo -e "  ${GREEN}●${NC} Platform: ${CF_DOCKER_PLATFORM:-not set}"
        if [[ -n "${CLOUDFLARE_API_TOKEN:-}" ]]; then
            echo -e "  ${GREEN}●${NC} API Token: ***configured***"
        else
            echo -e "  ${YELLOW}○${NC} API Token: not set (using wrangler login)"
        fi
    else
        echo -e "  ${RED}○${NC} Not configured"
        echo ""
        echo "  Run one of:"
        echo "    mx infra setup cloudflare     # Set up global config"
        echo "    mx cf setup                   # Set up project-local config"
        echo "    mx infra link cloudflare      # Link to existing global config"
    fi
    echo ""
}

# Cloudflare commands
cloudflare_cmd() {
    local subcmd="${1:-help}"
    shift 2>/dev/null || true
    
    if ! is_mech_crate_project; then
        error "Not in a MechCrate project. Run 'mx new <name> --infra cloudflare' first."
    fi
    
    if [[ ! -d "infra/cloudflare" ]]; then
        error "Cloudflare infrastructure not set up. Create a new project with --infra cloudflare"
    fi
    
    case "$subcmd" in
        setup)
            _cf_setup_cmd
            ;;
        init)
            local app_name="$1"
            shift 2>/dev/null || true
            
            if [[ -z "$app_name" ]]; then
                echo -e "${BOLD}Usage:${NC} mx cf init <app-name> [--type=worker|cron|container]"
                echo ""
                echo "Initializes a new Cloudflare Worker with interactive setup."
                echo ""
                echo -e "${BOLD}Worker Types:${NC}"
                echo "    worker      Standard edge worker (APIs, proxies, static sites)"
                echo "    cron        Scheduled worker (background jobs, data sync)"
                echo "    container   Docker container backend (SSR apps, stateful services)"
                echo ""
                echo -e "${BOLD}Examples:${NC}"
                echo "    mx cf init api.example.com              # Interactive mode"
                echo "    mx cf init api.example.com --type=worker"
                echo "    mx cf init sync-job --type=cron"
                echo "    mx cf init app.example.com --type=container"
                exit 1
            fi
            
            # Check if Cloudflare is configured (global or project-local)
            if ! _cf_is_configured; then
                warn "Cloudflare not configured yet."
                echo ""
                echo "Options:"
                echo "  1) Use global config (if available)"
                echo "  2) Set up project-local credentials"
                echo ""
                
                # Check if global config exists
                if _infra_has_global_config "cloudflare"; then
                    info "Global Cloudflare config found."
                    if prompt_yn "Link this project to global config?"; then
                        _infra_link "cloudflare"
                        echo ""
                    else
                        if prompt_yn "Set up project-local credentials instead?"; then
                            _cf_setup_cmd || {
                                error "Setup failed. Please run 'mx cf setup' manually."
                            }
                            echo ""
                        else
                            error "Run 'mx cf setup' or 'mx infra link cloudflare' first."
                        fi
                    fi
                else
                    info "No global Cloudflare config found."
                    echo "You can set up global config with: mx infra setup cloudflare"
                    echo ""
                    if prompt_yn "Set up project-local credentials now?"; then
                        _cf_setup_cmd || {
                            error "Setup failed. Please run 'mx cf setup' manually."
                        }
                        echo ""
                    else
                        error "Run 'mx cf setup' first to configure your Cloudflare account."
                    fi
                fi
            fi
            
            # Load config and run init
            _cf_load_config
            ./scripts/cf-init-app.sh "$app_name" "$@"
            ;;
        config)
            _cf_config_cmd
            ;;
        status)
            make cf-status
            ;;
        deploy)
            local app_name="$1"
            if [[ -z "$app_name" ]]; then
                error "Usage: mx cf deploy <app-name>"
            fi
            make cf-deploy a="$app_name"
            ;;
        deploy-all)
            make cf-deploy-all
            ;;
        list)
            make cf-list
            ;;
        dev)
            local app_name="$1"
            if [[ -z "$app_name" ]]; then
                error "Usage: mx cf dev <app-name>"
            fi
            make cf-dev a="$app_name"
            ;;
        logs)
            local app_name="$1"
            if [[ -z "$app_name" ]]; then
                error "Usage: mx cf logs <app-name>"
            fi
            make cf-logs a="$app_name"
            ;;
        help|--help|-h)
            echo -e "${BOLD}mx cf${NC} - Cloudflare Management Commands"
            echo ""
            echo -e "${BOLD}USAGE:${NC}"
            echo "    mx cf <command> [options]"
            echo ""
            echo -e "${BOLD}COMMANDS:${NC}"
            echo "    setup                 Run Cloudflare setup wizard"
            echo "    config                Show current config status and source"
            echo "    init <app> [opts]     Initialize a new worker (interactive)"
            echo "    status                Show all Cloudflare apps status"
            echo "    list                  List configured apps"
            echo "    deploy <app>          Deploy app to production"
            echo "    deploy-all            Deploy all apps"
            echo "    dev <app>             Run worker locally"
            echo "    logs <app>            Tail production logs"
            echo ""
            echo -e "${BOLD}INIT OPTIONS:${NC}"
            echo "    --type=worker         Standard edge worker"
            echo "    --type=cron           Scheduled worker"
            echo "    --type=container      Docker container backend"
            echo ""
            echo -e "${BOLD}CONFIGURATION:${NC}"
            echo "    Cloudflare credentials can be configured at two levels:"
            echo ""
            echo -e "    ${CYAN}Global${NC} (~/.mech-crate/config/infra/cloudflare.env)"
            echo "      Set once, shared across all projects."
            echo "      Configure with: mx infra setup cloudflare"
            echo ""
            echo -e "    ${CYAN}Project-local${NC} (./infra/cloudflare/.env.cloudflare)"
            echo "      Per-project credentials (e.g., different account)."
            echo "      Configure with: mx cf setup"
            echo ""
            echo "    Projects can link to global config with: mx infra link cloudflare"
            echo ""
            echo -e "${BOLD}EXAMPLES:${NC}"
            echo "    mx cf config                              # Show config status"
            echo "    mx cf setup                               # Configure credentials"
            echo "    mx cf init api.example.com"
            echo "    mx cf init my-job --type=cron"
            echo "    mx cf init app.example.com --type=container"
            echo "    mx cf deploy api.example.com"
            echo "    mx cf deploy-all"
            ;;
        *)
            # Try to proxy to make cf-* command
            if make -n "cf-$subcmd" &>/dev/null 2>&1; then
                make "cf-$subcmd" "$@"
            else
                error "Unknown Cloudflare command: $subcmd. Run 'mx cf help' for usage."
            fi
            ;;
    esac
}
