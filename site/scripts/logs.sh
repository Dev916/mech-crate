#!/bin/bash
# Tail service logs
# Usage: ./scripts/logs.sh [service]

set -e

# Check if we have previous run context
if ! ls tmp/up/*.txt 1>/dev/null 2>&1; then
    echo "No services running. Start services first with 'make dev' or 'make up'"
    exit 1
fi

files=$(cat tmp/up/*.txt)

docker compose $files logs -f --tail=1000 $1
