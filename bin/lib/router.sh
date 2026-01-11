#!/bin/bash
#
# MechCrate Router Management
# Global Traefik reverse proxy for all MechCrate projects
#

# ─────────────────────────────────────────────────────────────────────────────
# Configuration
# ─────────────────────────────────────────────────────────────────────────────

MX_ROUTER_HOME="${MX_ROUTER_HOME:-${HOME}/.mech-crate/router}"
MX_ROUTER_NETWORK="${MX_ROUTER_NETWORK:-devmesh-traefik}"
MX_ROUTER_PROJECT="mx-router"
MX_ROUTER_DASHBOARD_RANGE="${MX_ROUTER_DASHBOARD_RANGE:-7680-7799}"
MX_ROUTER_DASHBOARD_PORT_FILE="${MX_ROUTER_HOME}/.dashboard-port"
MX_ROUTER_DASHBOARD_PORT=""

# ─────────────────────────────────────────────────────────────────────────────
# Internal Helpers
# ─────────────────────────────────────────────────────────────────────────────

_router_log() {
    echo -e "${CYAN}[router]${NC} $*"
}

_router_die() {
    echo -e "${RED}[router] error:${NC} $*" >&2
    exit 1
}

_router_need_command() {
    command -v "$1" >/dev/null 2>&1 || _router_die "'$1' command is required"
}

_router_ensure_installed() {
    [[ -f "${MX_ROUTER_HOME}/docker-compose.yml" ]] || _router_die "Router not installed. Run: mx router install"
}

_router_ensure_network() {
    if ! docker network inspect "${MX_ROUTER_NETWORK}" >/dev/null 2>&1; then
        _router_log "Creating docker network ${MX_ROUTER_NETWORK}"
        docker network create "${MX_ROUTER_NETWORK}" >/dev/null
    fi
}

_router_range_bounds() {
    local range="$1"
    if [[ "${range}" =~ ^([0-9]+)-([0-9]+)$ ]]; then
        printf '%s %s\n' "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}"
    elif [[ "${range}" =~ ^[0-9]+$ ]]; then
        printf '%s %s\n' "${range}" "${range}"
    else
        _router_die "Invalid MX_ROUTER_DASHBOARD_RANGE: ${range}"
    fi
}

_router_allocate_dashboard_port() {
    _router_need_command python3
    local bounds start end selection
    bounds="$(_router_range_bounds "${MX_ROUTER_DASHBOARD_RANGE}")"
    read -r start end <<<"${bounds}"
    selection="$(python3 - "$start" "$end" <<'PY' || true
import socket
import sys

start = int(sys.argv[1])
end = int(sys.argv[2])

for port in range(start, end + 1):
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        try:
            sock.bind(('127.0.0.1', port))
        except OSError:
            continue
        print(port)
        sys.exit(0)
sys.exit(1)
PY
)"
    [[ -n "${selection}" ]] || _router_die "Unable to find free port in range ${start}-${end}"
    printf '%s' "${selection}"
}

_router_ensure_dashboard_port() {
    [[ -n "${MX_ROUTER_DASHBOARD_PORT}" ]] && {
        export MX_ROUTER_DASHBOARD_PORT
        return
    }

    local candidate=""

    if [[ -n "${MX_ROUTER_DASHBOARD_PORT_ENV:-}" ]]; then
        candidate="${MX_ROUTER_DASHBOARD_PORT_ENV}"
    elif [[ -s "${MX_ROUTER_DASHBOARD_PORT_FILE}" ]]; then
        candidate="$(tr -d '[:space:]' <"${MX_ROUTER_DASHBOARD_PORT_FILE}")"
    else
        candidate="$(_router_allocate_dashboard_port)"
        printf '%s\n' "${candidate}" >"${MX_ROUTER_DASHBOARD_PORT_FILE}"
        _router_log "Selected dashboard port ${candidate} (range ${MX_ROUTER_DASHBOARD_RANGE})"
    fi

    [[ "${candidate}" =~ ^[0-9]+$ ]] || _router_die "Invalid dashboard port '${candidate}'"
    MX_ROUTER_DASHBOARD_PORT="${candidate}"
    export MX_ROUTER_DASHBOARD_PORT
}

_router_current_dashboard_port() {
    if [[ -n "${MX_ROUTER_DASHBOARD_PORT}" ]]; then
        printf '%s' "${MX_ROUTER_DASHBOARD_PORT}"
    elif [[ -s "${MX_ROUTER_DASHBOARD_PORT_FILE}" ]]; then
        tr -d '[:space:]' <"${MX_ROUTER_DASHBOARD_PORT_FILE}"
    fi
}

_router_compose() {
    _router_ensure_installed
    _router_ensure_dashboard_port
    _router_need_command docker
    (
        cd "${MX_ROUTER_HOME}"
        MX_ROUTER_DASHBOARD_PORT="${MX_ROUTER_DASHBOARD_PORT}" docker compose -p "${MX_ROUTER_PROJECT}" "$@"
    )
}

# ─────────────────────────────────────────────────────────────────────────────
# Router Commands
# ─────────────────────────────────────────────────────────────────────────────

_router_install() {
    _router_need_command docker
    
    local template_dir="${MECH_CRATE_ROOT}/templates/router"
    [[ -d "${template_dir}" ]] || _router_die "Router template not found at ${template_dir}"
    
    mkdir -p "${MX_ROUTER_HOME}"
    _router_log "Installing router to ${MX_ROUTER_HOME}"
    
    # Copy template files
    cp -r "${template_dir}/"* "${MX_ROUTER_HOME}/"
    
    # Set proper permissions on acme.json
    chmod 600 "${MX_ROUTER_HOME}/letsencrypt/acme.json" 2>/dev/null || true
    
    # Create the shared network
    _router_ensure_network
    
    success "Router installed. Run 'mx router up' to start."
}

_router_up() {
    _router_ensure_installed
    _router_ensure_network
    _router_compose up -d
    _router_log "Dashboard: http://localhost:${MX_ROUTER_DASHBOARD_PORT}"
}

_router_down() {
    _router_ensure_installed
    _router_compose down
}

_router_restart() {
    _router_down
    _router_up
}

_router_status() {
    _router_ensure_installed
    _router_compose ps
}

_router_logs() {
    _router_ensure_installed
    _router_compose logs -f
}

_router_reload() {
    _router_ensure_installed
    _router_compose kill -s HUP traefik
    _router_log "Configuration reloaded"
}

_router_inspect() {
    local port
    port="$(_router_current_dashboard_port || true)"
    port="${port:-<unassigned>}"
    
    echo -e "${BOLD}MechCrate Router${NC}"
    echo ""
    echo -e "  ${CYAN}State Dir${NC}  : ${MX_ROUTER_HOME}"
    echo -e "  ${CYAN}Compose${NC}    : ${MX_ROUTER_HOME}/docker-compose.yml"
    echo -e "  ${CYAN}Network${NC}    : ${MX_ROUTER_NETWORK}"
    echo -e "  ${CYAN}Project${NC}    : ${MX_ROUTER_PROJECT}"
    echo -e "  ${CYAN}Port Range${NC} : ${MX_ROUTER_DASHBOARD_RANGE}"
    echo -e "  ${CYAN}Dashboard${NC}  : http://localhost:${port}"
    echo ""
    
    # Show connected services if router is running
    if docker ps --format '{{.Names}}' 2>/dev/null | grep -q "^mx-router$"; then
        echo -e "${BOLD}Connected Services:${NC}"
        docker network inspect "${MX_ROUTER_NETWORK}" --format '{{range .Containers}}  - {{.Name}}{{"\n"}}{{end}}' 2>/dev/null || true
    fi
}

_router_network() {
    _router_ensure_network
    echo "${MX_ROUTER_NETWORK}"
}

_router_uninstall() {
    if [[ -d "${MX_ROUTER_HOME}" ]]; then
        # Stop if running
        if docker ps --format '{{.Names}}' 2>/dev/null | grep -q "^mx-router$"; then
            _router_log "Stopping router..."
            _router_down 2>/dev/null || true
        fi
        
        _router_log "Removing ${MX_ROUTER_HOME}"
        rm -rf "${MX_ROUTER_HOME}"
        
        # Optionally remove network
        if prompt_yn "Remove the ${MX_ROUTER_NETWORK} network?" "n"; then
            docker network rm "${MX_ROUTER_NETWORK}" 2>/dev/null || true
            _router_log "Network removed"
        fi
        
        success "Router uninstalled"
    else
        warn "Router not installed"
    fi
}

_router_help() {
    echo -e "${BOLD}mx router${NC} - MechCrate Global Router Management"
    echo ""
    echo -e "${BOLD}USAGE:${NC}"
    echo "    mx router <command>"
    echo ""
    echo -e "${BOLD}COMMANDS:${NC}"
    echo "    install     Install the global Traefik router"
    echo "    up          Start or update the router"
    echo "    down        Stop the router"
    echo "    restart     Stop and start the router"
    echo "    status      Show router container status"
    echo "    logs        Tail router logs (Ctrl+C to stop)"
    echo "    reload      Hot-reload configuration without restart"
    echo "    inspect     Show router details and connected services"
    echo "    network     Ensure network exists and print name"
    echo "    uninstall   Remove router installation"
    echo ""
    echo -e "${BOLD}CONFIGURATION:${NC}"
    echo "    Dynamic config: ~/.mech-crate/router/config/dynamic/"
    echo "    Add .yml files there for custom middlewares, routes, etc."
    echo ""
    echo -e "${BOLD}ENVIRONMENT VARIABLES:${NC}"
    echo "    MX_ROUTER_HOME              State directory (default: ~/.mech-crate/router)"
    echo "    MX_ROUTER_NETWORK           Docker network name (default: devmesh-traefik)"
    echo "    MX_ROUTER_DASHBOARD_PORT    Force specific dashboard port"
    echo "    MX_ROUTER_DASHBOARD_RANGE   Port scan range (default: 7680-7799)"
    echo ""
    echo -e "${BOLD}EXAMPLES:${NC}"
    echo "    mx router install           # First-time setup"
    echo "    mx router up                # Start the router"
    echo "    mx router inspect           # See dashboard URL and connected services"
    echo "    mx router logs              # Debug issues"
    echo ""
    echo -e "${BOLD}HOW IT WORKS:${NC}"
    echo "    The router runs a global Traefik instance on ports 80/443."
    echo "    MechCrate services connect to the 'devmesh-traefik' network"
    echo "    and use Docker labels to register their hostname routes."
    echo ""
    echo "    Example service labels:"
    echo "      - traefik.enable=true"
    echo "      - traefik.http.routers.myapp.rule=Host(\`myapp.localhost\`)"
    echo "      - traefik.http.services.myapp.loadbalancer.server.port=80"
    echo "      - traefik.docker.network=devmesh-traefik"
    echo ""
}

# ─────────────────────────────────────────────────────────────────────────────
# Main Router Command Handler
# ─────────────────────────────────────────────────────────────────────────────

router_cmd() {
    local subcommand="${1:-help}"
    shift 2>/dev/null || true
    
    case "$subcommand" in
        install)
            _router_install "$@"
            ;;
        up|start)
            _router_up "$@"
            ;;
        down|stop)
            _router_down "$@"
            ;;
        restart)
            _router_restart "$@"
            ;;
        status|ps)
            _router_status "$@"
            ;;
        logs)
            _router_logs "$@"
            ;;
        reload)
            _router_reload "$@"
            ;;
        inspect|info)
            _router_inspect "$@"
            ;;
        network)
            _router_network "$@"
            ;;
        uninstall|remove)
            _router_uninstall "$@"
            ;;
        help|--help|-h)
            _router_help
            ;;
        *)
            error "Unknown router command: $subcommand. Run 'mx router help' for usage."
            ;;
    esac
}
