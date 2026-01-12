#!/bin/bash
# Start services in production mode (no dev overrides)
# Usage: ./scripts/up.sh [service]

set -e

source ./scripts/.bashrc

# Ensure project is initialized (network + secrets)
./scripts/init.sh >/dev/null 2>&1 || true

# Get compose files without dev overrides
files=$(compose_context_files "${1:-}" "false")

if [ -z "$files" ]; then
    if [[ -n "${1:-}" ]]; then
        echo "No service configuration found for '$1'"
    else
        echo "No services found (no compose files in docker/compose/)"
    fi
    echo ""
    echo "Add a service first:"
    echo "  mx add api --recipe=nuxt"
    echo ""
    echo "Available services:"
    ls -1 docker/compose/*.yml 2>/dev/null | xargs -n1 basename | sed 's/.yml$//' | grep -v '.dev$' | sed 's/^/  - /' || true
    exit 1
fi

run_service_in_context "$files" "$1"
