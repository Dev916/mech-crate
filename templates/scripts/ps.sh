#!/bin/bash
# List running services
# Usage: ./scripts/ps.sh

set -e

# Check if we have previous run context
if ls tmp/up/*.txt 1>/dev/null 2>&1; then
    files=$(cat tmp/up/*.txt)
    docker compose $files ps
else
    # Fallback to showing all containers with our project
    docker ps --filter "label=com.docker.compose.project" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
fi
