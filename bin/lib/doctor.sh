#!/bin/bash
#
# MechCrate Doctor Command
# Check project health and dependencies
#

# Doctor - check project health
doctor() {
    # Try to find project root (don't error if not found)
    local project_root
    project_root=$(find_project_root 2>/dev/null) || true
    
    if [[ -n "$project_root" ]]; then
        cd "$project_root"
        info "Checking MechCrate project health..."
    else
        # Check global dependencies only
        info "Checking global dependencies..."
    fi
    
    local has_errors=false
    
    # Check Docker
    if command -v docker &> /dev/null; then
        success "Docker: $(docker --version | cut -d' ' -f3 | tr -d ',')"
    else
        error "Docker: not found"
        has_errors=true
    fi
    
    # Check Docker Compose
    if docker compose version &> /dev/null; then
        success "Docker Compose: $(docker compose version --short)"
    else
        warn "Docker Compose: not found (using docker-compose?)"
    fi
    
    # Check Make
    if command -v make &> /dev/null; then
        success "Make: $(make --version | head -1)"
    else
        error "Make: not found"
        has_errors=true
    fi
    
    # Project-specific checks (only if we found a project root)
    if [[ -n "$project_root" ]]; then
        echo ""
        info "Checking project structure..."
        
        [[ -f "Makefile" ]] && success "Makefile exists" || warn "Makefile missing"
        [[ -d "make" ]] && success "make/ directory exists" || warn "make/ directory missing"
        [[ -d "scripts" ]] && success "scripts/ directory exists" || warn "scripts/ directory missing"
        [[ -d "docker/compose" ]] && success "docker/compose/ exists" || warn "docker/compose/ missing"
        [[ -d "docker/.config" ]] && success "docker/.config/ exists" || warn "docker/.config/ missing"
        
        # Check for secrets file
        if [[ -f "docker/.config/.env.secrets" ]]; then
            success "Secrets file exists"
        else
            warn "Secrets file missing - copy from .env.secrets.template"
        fi
        
        echo ""
        info "Services found:"
        for yml in docker/compose/*.yml; do
            if [[ -f "$yml" && ! "$(basename "$yml")" =~ \.dev\.yml$ ]]; then
                echo "    - $(basename "$yml" .yml)"
            fi
        done
    fi
    
    echo ""
    if [[ "$has_errors" == "true" ]]; then
        error "Some dependencies are missing. Please install them first."
    else
        success "All checks passed!"
        echo -e "${CYAN}🦝 Crate Raccoon says: Looking good!${NC}"
    fi
}
