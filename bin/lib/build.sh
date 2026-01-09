#!/bin/bash
#
# MechCrate Build Library
# Build service images with dev/prod modes
#

# ─────────────────────────────────────────────────────────────────────────────
# Build Command Handler
# ─────────────────────────────────────────────────────────────────────────────
build_cmd() {
    local service=""
    local tag="latest"
    local mode="dev"
    local push=0
    local extra_args=""
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -s|--service)
                service="$2"
                shift 2
                ;;
            -t|--tag)
                tag="$2"
                shift 2
                ;;
            --prod|--production)
                mode="prod"
                shift
                ;;
            --dev|--development)
                mode="dev"
                shift
                ;;
            --push)
                push=1
                shift
                ;;
            --platform)
                extra_args="--platform=$2"
                shift 2
                ;;
            -h|--help)
                show_build_help
                return 0
                ;;
            *)
                # First positional arg is service if not set
                if [ -z "$service" ]; then
                    service="$1"
                else
                    extra_args="$extra_args $1"
                fi
                shift
                ;;
        esac
    done
    
    # Validate we're in a project
    if ! is_mech_crate_project; then
        error "Not in a MechCrate project. Run 'mx new <name>' first."
    fi
    
    # Service is required
    if [ -z "$service" ]; then
        echo ""
        warn "No service specified"
        echo ""
        show_available_services
        echo ""
        echo "Usage: mx build <service> [options]"
        echo "       mx build --help"
        return 1
    fi
    
    # Validate service exists
    if [ ! -d "docker/dockerfiles/$service" ]; then
        echo ""
        error "Service '$service' not found"
        echo ""
        show_available_services
        return 1
    fi
    
    # Run the build
    if [ "$mode" = "prod" ]; then
        info "Building production image for $service..."
        make _build service="$service" tag="$tag" mode=prod push="$push"
    else
        info "Building development image for $service..."
        make _build service="$service" tag="$tag" mode=dev push="$push"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Help
# ─────────────────────────────────────────────────────────────────────────────
show_build_help() {
    echo ""
    echo -e "${BOLD}mx build${NC} - Build service Docker images"
    echo ""
    echo -e "${BOLD}USAGE${NC}"
    echo "    mx build <service> [options]"
    echo ""
    echo -e "${BOLD}ARGUMENTS${NC}"
    echo "    <service>    Service name to build"
    echo ""
    echo -e "${BOLD}OPTIONS${NC}"
    echo "    --prod, --production     Build production-optimized image"
    echo "    --dev, --development     Build development image (default)"
    echo "    -t, --tag <tag>          Image tag (default: latest)"
    echo "    --push                   Push image to registry after build"
    echo "    --platform <platform>    Target platform (e.g., linux/amd64)"
    echo "    -h, --help               Show this help"
    echo ""
    echo -e "${BOLD}EXAMPLES${NC}"
    echo "    mx build api                    # Dev build of 'api' service"
    echo "    mx build api --prod             # Production build"
    echo "    mx build api -t v1.0.0 --prod   # Production with tag"
    echo "    mx build api --prod --push      # Production build & push"
    echo ""
    echo -e "${BOLD}BUILD MODES${NC}"
    echo ""
    echo "  ${GREEN}Development (--dev)${NC}"
    echo "    - Uses app Dockerfile with 'development' target"
    echo "    - Includes dev tools, debugging support"
    echo "    - Optimized for fast iteration"
    echo ""
    echo "  ${GREEN}Production (--prod)${NC}"
    echo "    - Uses app.prod Dockerfile if available"
    echo "    - Minimal image size, no dev tools"
    echo "    - Security hardening, non-root user"
    echo "    - Optimized for deployment"
    echo ""
}

# Show available services to build
show_available_services() {
    echo -e "${BOLD}Available services:${NC}"
    if [ -d "docker/dockerfiles" ]; then
        for dir in docker/dockerfiles/*/; do
            if [ -d "$dir" ]; then
                local service=$(basename "$dir")
                local has_prod=""
                if [ -f "$dir/app.prod" ]; then
                    has_prod=" ${GREEN}(prod available)${NC}"
                fi
                echo -e "  - $service$has_prod"
            fi
        done
    else
        echo "  (no services found)"
    fi
}
