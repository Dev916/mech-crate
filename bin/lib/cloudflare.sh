#!/bin/bash
#
# MechCrate Cloudflare Commands
# Cloudflare Workers & Containers management
#

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
            ./scripts/cf-setup.sh
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
            
            # Check if cf-setup has been run
            if [[ ! -f "infra/cloudflare/.env.cloudflare" ]]; then
                warn "Cloudflare not configured yet."
                if prompt_yn "Would you like to run setup now?"; then
                    ./scripts/cf-setup.sh || {
                        error "Setup failed. Please run 'mx cf setup' manually."
                    }
                    echo ""
                else
                    error "Run 'mx cf setup' first to configure your Cloudflare account."
                fi
            fi
            
            ./scripts/cf-init-app.sh "$app_name" "$@"
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
            echo -e "${BOLD}EXAMPLES:${NC}"
            echo "    mx cf setup"
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
