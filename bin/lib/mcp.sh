#!/bin/bash
#
# MechCrate MCP Server Management
# Commands for managing the MCP server and Weaviate RAG backend
#
# Port allocation strategy:
#   - Weaviate HTTP: 8080-8179
#   - Weaviate gRPC: 50051-50150
#   - State stored in ~/.mech-crate/mcp/

# ─────────────────────────────────────────────────────────────────────────────
# Configuration
# ─────────────────────────────────────────────────────────────────────────────

MX_MCP_DIR="${MECH_CRATE_ROOT}/mcp-server"
MX_MCP_STATE_DIR="${HOME}/.mech-crate/mcp"
MX_MCP_BIN="${MX_MCP_DIR}/target/release/mx-mcp"
MX_INGEST_BIN="${MX_MCP_DIR}/target/release/mx-ingest"
MX_MCP_PROJECT="mx-mcp-rag"

# Port ranges
MX_MCP_HTTP_PORT_RANGE="${MX_MCP_HTTP_PORT_RANGE:-8080-8179}"
MX_MCP_GRPC_PORT_RANGE="${MX_MCP_GRPC_PORT_RANGE:-50051-50150}"

# State files
MX_MCP_HTTP_PORT_FILE="${MX_MCP_STATE_DIR}/.weaviate-http-port"
MX_MCP_GRPC_PORT_FILE="${MX_MCP_STATE_DIR}/.weaviate-grpc-port"
MX_MCP_PID_FILE="${MX_MCP_STATE_DIR}/.weaviate-pid"

# ─────────────────────────────────────────────────────────────────────────────
# Internal Helpers
# ─────────────────────────────────────────────────────────────────────────────

_mcp_log() {
    echo -e "${CYAN}[mcp]${NC} $*" >&2
}

_mcp_die() {
    echo -e "${RED}[mcp] error:${NC} $*" >&2
    exit 1
}

_mcp_ensure_state_dir() {
    mkdir -p "${MX_MCP_STATE_DIR}"
}

_mcp_need_build() {
    if [[ ! -f "$MX_MCP_BIN" ]]; then
        return 0
    fi
    
    # Check if source is newer than binary
    local newest_src
    newest_src=$(find "$MX_MCP_DIR/src" -name "*.rs" -newer "$MX_MCP_BIN" 2>/dev/null | head -1)
    [[ -n "$newest_src" ]]
}

_mcp_ensure_binary() {
    if _mcp_need_build; then
        _mcp_log "Building MCP server..."
        (cd "$MX_MCP_DIR" && cargo build --release) || _mcp_die "Build failed"
        success "MCP server built"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Port Allocation
# ─────────────────────────────────────────────────────────────────────────────

_mcp_parse_range() {
    local range="$1"
    if [[ "${range}" =~ ^([0-9]+)-([0-9]+)$ ]]; then
        printf '%s %s\n' "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}"
    elif [[ "${range}" =~ ^[0-9]+$ ]]; then
        printf '%s %s\n' "${range}" "${range}"
    else
        _mcp_die "Invalid port range: ${range}"
    fi
}

_mcp_is_port_free() {
    local port="$1"
    ! (echo >/dev/tcp/127.0.0.1/"$port") 2>/dev/null
}

_mcp_find_free_port() {
    local range="$1"
    local start end
    read -r start end < <(_mcp_parse_range "$range")
    
    for ((port = start; port <= end; port++)); do
        if _mcp_is_port_free "$port"; then
            echo "$port"
            return 0
        fi
    done
    
    return 1
}

_mcp_allocate_ports() {
    _mcp_ensure_state_dir
    
    local http_port grpc_port
    
    # Check if we have stored ports that are still valid
    if [[ -f "$MX_MCP_HTTP_PORT_FILE" && -f "$MX_MCP_GRPC_PORT_FILE" ]]; then
        http_port=$(cat "$MX_MCP_HTTP_PORT_FILE")
        grpc_port=$(cat "$MX_MCP_GRPC_PORT_FILE")
        
        # Check if Weaviate is already running on these ports
        if curl -sf "http://localhost:${http_port}/v1/.well-known/ready" >/dev/null 2>&1; then
            _mcp_log "Weaviate already running on port ${http_port}"
            echo "${http_port} ${grpc_port}"
            return 0
        fi
        
        # Check if ports are still free
        if _mcp_is_port_free "$http_port" && _mcp_is_port_free "$grpc_port"; then
            echo "${http_port} ${grpc_port}"
            return 0
        fi
        
        _mcp_log "Previously allocated ports in use, finding new ones..."
    fi
    
    # Allocate new ports
    http_port=$(_mcp_find_free_port "$MX_MCP_HTTP_PORT_RANGE") || \
        _mcp_die "No free ports in HTTP range ${MX_MCP_HTTP_PORT_RANGE}"
    
    grpc_port=$(_mcp_find_free_port "$MX_MCP_GRPC_PORT_RANGE") || \
        _mcp_die "No free ports in gRPC range ${MX_MCP_GRPC_PORT_RANGE}"
    
    # Store the ports
    echo "$http_port" > "$MX_MCP_HTTP_PORT_FILE"
    echo "$grpc_port" > "$MX_MCP_GRPC_PORT_FILE"
    
    _mcp_log "Allocated ports: HTTP=${http_port}, gRPC=${grpc_port}"
    echo "${http_port} ${grpc_port}"
}

_mcp_get_weaviate_url() {
    if [[ -f "$MX_MCP_HTTP_PORT_FILE" ]]; then
        local port
        port=$(cat "$MX_MCP_HTTP_PORT_FILE")
        echo "http://localhost:${port}"
    else
        echo "http://localhost:8080"
    fi
}

_mcp_get_http_port() {
    if [[ -f "$MX_MCP_HTTP_PORT_FILE" ]]; then
        cat "$MX_MCP_HTTP_PORT_FILE"
    else
        echo "8080"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Weaviate Management
# ─────────────────────────────────────────────────────────────────────────────

_mcp_is_weaviate_running() {
    local url
    url=$(_mcp_get_weaviate_url)
    curl -sf "${url}/v1/.well-known/ready" >/dev/null 2>&1
}

_mcp_weaviate_compose() {
    local http_port grpc_port
    read -r http_port grpc_port < <(_mcp_allocate_ports)
    
    (
        cd "$MX_MCP_DIR"
        MX_MCP_WEAVIATE_PORT="$http_port" \
        MX_MCP_GRPC_PORT="$grpc_port" \
        docker compose -p "$MX_MCP_PROJECT" "$@"
    )
}

_mcp_start_weaviate() {
    local http_port grpc_port
    read -r http_port grpc_port < <(_mcp_allocate_ports)
    
    # Check if already running
    if curl -sf "http://localhost:${http_port}/v1/.well-known/ready" >/dev/null 2>&1; then
        _mcp_log "Weaviate already running on port ${http_port}"
        return 0
    fi
    
    _mcp_log "Starting Weaviate on port ${http_port}..."
    
    (
        cd "$MX_MCP_DIR"
        MX_MCP_WEAVIATE_PORT="$http_port" \
        MX_MCP_GRPC_PORT="$grpc_port" \
        docker compose -p "$MX_MCP_PROJECT" up -d
    )
    
    # Wait for ready
    _mcp_log "Waiting for Weaviate to be ready..."
    local timeout=120
    while [[ $timeout -gt 0 ]]; do
        if curl -sf "http://localhost:${http_port}/v1/.well-known/ready" >/dev/null 2>&1; then
            success "Weaviate is ready at http://localhost:${http_port}"
            return 0
        fi
        sleep 2
        timeout=$((timeout - 2))
        echo -n "."
    done
    echo ""
    
    warn "Weaviate startup timed out. Check: docker logs mx-mcp-weaviate"
    return 1
}

_mcp_stop_weaviate() {
    _mcp_log "Stopping Weaviate..."
    _mcp_weaviate_compose down 2>/dev/null || true
    success "Weaviate stopped"
}

# ─────────────────────────────────────────────────────────────────────────────
# MCP Commands
# ─────────────────────────────────────────────────────────────────────────────

_mcp_build() {
    _mcp_log "Building MCP server..."
    (cd "$MX_MCP_DIR" && cargo build --release) || _mcp_die "Build failed"
    success "MCP server built successfully"
    echo ""
    echo "Binaries:"
    echo "  $MX_MCP_BIN"
    echo "  $MX_INGEST_BIN"
}

_mcp_start_rag() {
    _mcp_start_weaviate
}

_mcp_stop_rag() {
    _mcp_stop_weaviate
}

_mcp_status_rag() {
    local http_port
    http_port=$(_mcp_get_http_port)
    
    echo -e "${BOLD}Weaviate RAG Backend Status${NC}"
    echo ""
    echo -e "  ${CYAN}HTTP Port${NC}  : ${http_port}"
    echo -e "  ${CYAN}URL${NC}        : http://localhost:${http_port}"
    echo -e "  ${CYAN}State Dir${NC}  : ${MX_MCP_STATE_DIR}"
    echo ""
    
    if _mcp_is_weaviate_running; then
        echo -e "  ${GREEN}●${NC} Running"
    else
        echo -e "  ${RED}●${NC} Not running"
    fi
    echo ""
    
    docker ps --filter "name=mx-mcp" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" 2>/dev/null || true
}

_mcp_logs_rag() {
    _mcp_weaviate_compose logs -f
}

_mcp_ingest() {
    _mcp_ensure_binary
    
    # Ensure Weaviate is running
    if ! _mcp_is_weaviate_running; then
        _mcp_log "Starting Weaviate first..."
        _mcp_start_weaviate || _mcp_die "Failed to start Weaviate"
    fi
    
    local weaviate_url
    weaviate_url=$(_mcp_get_weaviate_url)
    
    local clear_flag=""
    if [[ "$1" == "--clear" ]]; then
        clear_flag="--clear"
        shift
    fi
    
    _mcp_log "Ingesting MechCrate documentation..."
    "$MX_INGEST_BIN" --weaviate-url "$weaviate_url" --mech-crate-root "$MECH_CRATE_ROOT" $clear_flag
    success "Documentation ingested"
}

_mcp_run() {
    _mcp_ensure_binary
    
    # Auto-start Weaviate if not running
    if ! _mcp_is_weaviate_running; then
        _mcp_log "Auto-starting Weaviate..."
        _mcp_start_weaviate || _mcp_die "Failed to start Weaviate"
    fi
    
    local weaviate_url
    weaviate_url=$(_mcp_get_weaviate_url)
    
    _mcp_log "Starting MCP server with Weaviate at ${weaviate_url}..."
    exec "$MX_MCP_BIN" --weaviate-url "$weaviate_url" "$@"
}

_mcp_config() {
    _mcp_ensure_binary
    
    local mcp_bin="$MX_MCP_BIN"
    local mech_root="$MECH_CRATE_ROOT"
    local weaviate_url
    weaviate_url=$(_mcp_get_weaviate_url)
    
    # Create the wrapper script
    local wrapper_script="${MX_MCP_STATE_DIR}/mx-mcp-wrapper.sh"
    _mcp_ensure_state_dir
    
    cat > "$wrapper_script" << EOF
#!/bin/bash
# MechCrate MCP Server Wrapper
# Auto-starts Weaviate and runs the MCP server

set -e

MECH_CRATE_ROOT="$mech_root"
source "\${MECH_CRATE_ROOT}/bin/lib/common.sh"
source "\${MECH_CRATE_ROOT}/bin/lib/mcp.sh"

# Start Weaviate if not running
if ! _mcp_is_weaviate_running; then
    _mcp_start_weaviate >/dev/null 2>&1 || true
fi

weaviate_url=\$(_mcp_get_weaviate_url)

exec "$mcp_bin" --weaviate-url "\$weaviate_url" "\$@"
EOF
    chmod +x "$wrapper_script"
    
    echo ""
    echo -e "${BOLD}MCP Client Configuration${NC}"
    echo ""
    echo "Add this to your MCP client configuration:"
    echo ""
    echo -e "${CYAN}Claude Desktop (~/.claude/claude_desktop_config.json):${NC}"
    echo ""
    cat << EOF
{
  "mcpServers": {
    "mechcrate": {
      "command": "$wrapper_script",
      "env": {
        "MECH_CRATE_ROOT": "$mech_root"
      }
    }
  }
}
EOF
    echo ""
    echo -e "${CYAN}Cursor IDE (mcp.json in workspace or ~/.cursor/mcp.json):${NC}"
    echo ""
    cat << EOF
{
  "mcpServers": {
    "mechcrate": {
      "command": "$wrapper_script",
      "env": {
        "MECH_CRATE_ROOT": "$mech_root"
      }
    }
  }
}
EOF
    echo ""
    echo -e "${CYAN}Alternative (direct, requires Weaviate pre-started):${NC}"
    echo ""
    cat << EOF
{
  "mcpServers": {
    "mechcrate": {
      "command": "$mcp_bin",
      "args": ["--weaviate-url", "$weaviate_url"],
      "env": {
        "MECH_CRATE_ROOT": "$mech_root"
      }
    }
  }
}
EOF
    echo ""
    info "Wrapper script created at: $wrapper_script"
    info "The wrapper auto-starts Weaviate when the MCP server starts."
}

_mcp_test() {
    _mcp_ensure_binary
    
    # Ensure Weaviate is running
    if ! _mcp_is_weaviate_running; then
        _mcp_log "Starting Weaviate first..."
        _mcp_start_weaviate || _mcp_die "Failed to start Weaviate"
    fi
    
    local weaviate_url
    weaviate_url=$(_mcp_get_weaviate_url)
    
    echo ""
    _mcp_log "Testing MCP server with Weaviate at ${weaviate_url}..."
    echo ""
    
    # Test initialization
    echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | \
        timeout 5 "$MX_MCP_BIN" --weaviate-url "$weaviate_url" 2>/dev/null | head -1 | jq .
    
    echo ""
    success "MCP server responds correctly"
}

_mcp_info() {
    local http_port
    http_port=$(_mcp_get_http_port)
    local weaviate_url
    weaviate_url=$(_mcp_get_weaviate_url)
    
    echo -e "${BOLD}MechCrate MCP Server Info${NC}"
    echo ""
    echo -e "  ${CYAN}MCP Binary${NC}     : $MX_MCP_BIN"
    echo -e "  ${CYAN}Ingest Binary${NC}  : $MX_INGEST_BIN"
    echo -e "  ${CYAN}State Dir${NC}      : $MX_MCP_STATE_DIR"
    echo -e "  ${CYAN}Compose Dir${NC}    : $MX_MCP_DIR"
    echo ""
    echo -e "  ${CYAN}Weaviate URL${NC}   : $weaviate_url"
    echo -e "  ${CYAN}HTTP Port${NC}      : $http_port"
    echo -e "  ${CYAN}HTTP Range${NC}     : $MX_MCP_HTTP_PORT_RANGE"
    echo -e "  ${CYAN}gRPC Range${NC}     : $MX_MCP_GRPC_PORT_RANGE"
    echo ""
    
    if [[ -f "$MX_MCP_BIN" ]]; then
        echo -e "  ${GREEN}●${NC} MCP binary built"
    else
        echo -e "  ${YELLOW}○${NC} MCP binary not built (run: mx mcp build)"
    fi
    
    if _mcp_is_weaviate_running; then
        echo -e "  ${GREEN}●${NC} Weaviate running"
    else
        echo -e "  ${RED}○${NC} Weaviate not running"
    fi
    echo ""
}

_mcp_help() {
    echo -e "${BOLD}mx mcp${NC} - MechCrate MCP Server Management"
    echo ""
    echo -e "${BOLD}USAGE:${NC}"
    echo "    mx mcp <command>"
    echo ""
    echo -e "${BOLD}COMMANDS:${NC}"
    echo "    build             Build the MCP server binary"
    echo "    start             Start the Weaviate RAG backend"
    echo "    stop              Stop the Weaviate RAG backend"
    echo "    status            Show Weaviate container status"
    echo "    logs              Tail Weaviate logs"
    echo "    ingest            Ingest documentation into Weaviate"
    echo "    ingest --clear    Clear existing docs and re-ingest"
    echo "    config            Show MCP client configuration"
    echo "    run               Run MCP server (auto-starts Weaviate)"
    echo "    test              Test MCP server response"
    echo "    info              Show MCP server information"
    echo ""
    echo -e "${BOLD}SETUP:${NC}"
    echo "    1. mx mcp build                 # Build the server"
    echo "    2. mx mcp start                 # Start Weaviate"
    echo "    3. mx mcp ingest                # Load documentation"
    echo "    4. mx mcp config                # Get client config"
    echo "    5. Add config to your MCP client"
    echo ""
    echo -e "${BOLD}PORT ALLOCATION:${NC}"
    echo "    Weaviate ports are dynamically allocated to avoid conflicts:"
    echo "    - HTTP: ${MX_MCP_HTTP_PORT_RANGE}"
    echo "    - gRPC: ${MX_MCP_GRPC_PORT_RANGE}"
    echo ""
    echo "    Override with environment variables:"
    echo "    MX_MCP_HTTP_PORT_RANGE=9080-9179 mx mcp start"
    echo ""
    echo -e "${BOLD}HOW IT WORKS:${NC}"
    echo "    The MCP server enables LLMs to:"
    echo "    - Create and manage MechCrate projects"
    echo "    - Add services using recipes"
    echo "    - Run make commands (dev, up, down, logs, etc.)"
    echo "    - Query documentation via RAG"
    echo ""
    echo "    When started via 'mx mcp run' or the wrapper script,"
    echo "    Weaviate is automatically started if not running."
    echo ""
    echo -e "${BOLD}TOOLS AVAILABLE:${NC}"
    echo "    40+ tools covering all mx operations including:"
    echo "    - mx_new, mx_add_service, mx_recipes_list"
    echo "    - mx_router_install, mx_router_up, mx_router_inspect"
    echo "    - make_dev, make_up, make_down, make_logs, make_shell"
    echo "    - project_analyze, service_info"
    echo "    - rag_search, rag_search_category"
    echo ""
}

# ─────────────────────────────────────────────────────────────────────────────
# Main MCP Command Handler
# ─────────────────────────────────────────────────────────────────────────────

mcp_cmd() {
    local subcommand="${1:-help}"
    shift 2>/dev/null || true
    
    case "$subcommand" in
        build)
            _mcp_build "$@"
            ;;
        start|up)
            _mcp_start_rag "$@"
            ;;
        stop|down)
            _mcp_stop_rag "$@"
            ;;
        status|ps)
            _mcp_status_rag "$@"
            ;;
        logs)
            _mcp_logs_rag "$@"
            ;;
        ingest)
            _mcp_ingest "$@"
            ;;
        config|configure)
            _mcp_config "$@"
            ;;
        run)
            _mcp_run "$@"
            ;;
        test)
            _mcp_test "$@"
            ;;
        info|inspect)
            _mcp_info "$@"
            ;;
        help|--help|-h)
            _mcp_help
            ;;
        *)
            error "Unknown mcp command: $subcommand. Run 'mx mcp help' for usage."
            ;;
    esac
}
