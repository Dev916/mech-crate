#!/bin/bash
# Start services in production mode (no dev overrides)
# Usage: ./scripts/up.sh [service]

set -e

source ./scripts/.bashrc

# Get compose files without dev overrides
files=$(compose_context_files "${1:-app}" "false")

if [ -z "$files" ]; then
    echo "No service configuration found for '${1:-app}'"
    echo "Available services:"
    ls -1 docker/compose/*.yml 2>/dev/null | xargs -n1 basename | sed 's/.yml$//' | grep -v '.dev$' | sed 's/^/  - /'
    exit 1
fi

run_service_in_context "$files" "$1"
