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

# Check compose project name
#
# Every script here pins `-p "$COMPOSE_PROJECT_NAME"` (see scripts/.bashrc), so
# this project's containers live in their own namespace. Migration hazard: a
# project whose containers were started BEFORE that pin landed ran under the
# name compose derives from the compose file's parent directory — "compose",
# shared by every mx project on the machine. Those containers are now orphaned:
# `make down`/`make ps` under the pinned name cannot see them.
#
# One `docker ps` surfaces it. Only the legacy default namespace is inspected —
# containers under any other project name belong to another stack and are none
# of this project's business. Ownership is settled by compose's own
# `project.working_dir` label, so an orphan of THIS project is never confused
# with another project that is also sitting in the shared default namespace.
echo ""
print_info "Checking compose project name..."
print_success "Compose project name: $COMPOSE_PROJECT_NAME"
legacy_project="$(mech_compose_project_name "$(pwd)/docker/compose")"
if [[ "$legacy_project" == "$COMPOSE_PROJECT_NAME" ]]; then
    : # nothing to migrate from
elif command -v docker &> /dev/null && docker ps --format '{{.ID}}' &>/dev/null; then
    declared_services=""
    for yml in docker/compose/*.yml; do
        if [[ -f "$yml" && ! "$(basename "$yml")" =~ \.dev\.yml$ ]]; then
            declared_services+=" $(basename "$yml" .yml)"
        fi
    done

    orphans=""
    neighbours=""
    while IFS='|' read -r proj svc name wdir; do
        [[ "$proj" == "$legacy_project" ]] || continue
        [[ " $declared_services " == *" $svc "* ]] || continue
        if [[ "$wdir" == "$(pwd)/docker/compose" ]]; then
            orphans+="    - $name (service '$svc')\n"
        else
            neighbours+="    - $name (service '$svc', from $wdir)\n"
        fi
    done < <(docker ps --format '{{.Label "com.docker.compose.project"}}|{{.Label "com.docker.compose.service"}}|{{.Names}}|{{.Label "com.docker.compose.project.working_dir"}}' 2>/dev/null)

    if [[ -n "$orphans" ]]; then
        print_warn "These containers are THIS project's, but still running under the old"
        print_warn "shared project name '$legacy_project' - 'make down' won't see them:"
        echo -en "$orphans"
        print_warn "Remove them once with: docker rm -f <name>   then 'make dev' again."
    else
        print_success "No orphans left under the old '$legacy_project' project name"
    fi
    if [[ -n "$neighbours" ]]; then
        print_info "Another stack is using the shared default project name '$legacy_project':"
        echo -en "$neighbours"
        print_info "Left alone - pinning '$COMPOSE_PROJECT_NAME' is what keeps this project out of it."
    fi
else
    print_warn "Docker not reachable - skipped the compose project name check"
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
