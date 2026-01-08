#!/bin/bash
# Check project health and dependencies
# Usage: ./scripts/doctor.sh

set -e

source ./scripts/.bashrc

print_info "Checking project health..."
echo ""

has_errors=false

# Check Docker
if command -v docker &> /dev/null; then
    print_success "Docker: $(docker --version | cut -d' ' -f3 | tr -d ',')"
else
    print_error "Docker: not found"
    has_errors=true
fi

# Check Docker Compose
if docker compose version &> /dev/null; then
    print_success "Docker Compose: $(docker compose version --short)"
else
    print_warn "Docker Compose: not found"
fi

# Check Make
if command -v make &> /dev/null; then
    print_success "Make: $(make --version | head -1)"
else
    print_error "Make: not found"
    has_errors=true
fi

# Check project structure
echo ""
print_info "Checking project structure..."

[[ -f "Makefile" ]] && print_success "Makefile exists" || print_warn "Makefile missing"
[[ -d "make" ]] && print_success "make/ directory exists" || print_warn "make/ directory missing"
[[ -d "scripts" ]] && print_success "scripts/ directory exists" || print_warn "scripts/ directory missing"
[[ -d "docker/compose" ]] && print_success "docker/compose/ exists" || print_warn "docker/compose/ missing"
[[ -d "docker/.config" ]] && print_success "docker/.config/ exists" || print_warn "docker/.config/ missing"

# Check for secrets file
if [[ -f "docker/.config/.env.secrets" ]]; then
    print_success "Secrets file exists"
else
    print_warn "Secrets file missing - copy from .env.secrets.template"
fi

# Check Docker network
NETWORK_NAME="${NETWORK_NAME:-mech-network}"
if docker network inspect "$NETWORK_NAME" &>/dev/null; then
    print_success "Docker network '$NETWORK_NAME' exists"
else
    print_warn "Docker network '$NETWORK_NAME' not found - run 'make init'"
fi

# List available services
echo ""
print_info "Available services:"
for yml in docker/compose/*.yml; do
    if [[ -f "$yml" && ! "$(basename "$yml")" =~ \.dev\.yml$ ]]; then
        echo "    - $(basename "$yml" .yml)"
    fi
done

echo ""
if [[ "$has_errors" == "true" ]]; then
    print_error "Some dependencies are missing. Please install them first."
    exit 1
else
    print_success "All checks passed!"
fi
