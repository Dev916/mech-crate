#!/bin/bash
# Start services in development mode
# Usage: ./scripts/dev.sh [service]

set -e

source ./scripts/.bashrc

# Stop existing services first
./scripts/down.sh "$1"

# Get compose files with dev overrides
files=$(compose_context_files "${1:-app}" "true")

if [ -z "$files" ]; then
    echo "No service configuration found for '${1:-app}'"
    echo "Available services:"
    ls -1 docker/compose/*.yml 2>/dev/null | xargs -n1 basename | sed 's/.yml$//' | grep -v '.dev$' | sed 's/^/  - /'
    exit 1
fi

run_service_in_context "$files" "$1"
