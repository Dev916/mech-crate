#!/bin/bash
# List running services
# Usage: ./scripts/ps.sh

set -e

source ./scripts/.bashrc

# Check if we have previous run context
if ls tmp/up/*.txt 1>/dev/null 2>&1; then
    files=$(cat tmp/up/*.txt)
    docker compose -p "$COMPOSE_PROJECT_NAME" $files ps
else
    # Fallback: containers of THIS project only (the label filter is what keeps
    # another mx stack's containers out of the listing).
    docker ps --filter "label=com.docker.compose.project=$COMPOSE_PROJECT_NAME" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
fi
