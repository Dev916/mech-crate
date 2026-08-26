#!/bin/bash
# Stop and remove services
# Usage: ./scripts/down.sh [service]

set -e

# Check if we have previous run context
if ! ls tmp/up/*.txt 1>/dev/null 2>&1; then
    echo "No previous runs found. Nothing to stop."
    exit 0
fi

files=$(cat tmp/up/*.txt)

if [ -n "$1" ]; then
    echo "docker compose $files stop $1"
    docker compose $files stop $1
    docker compose $files rm -f $1
else
    echo "docker compose $files down -t0"
    docker compose $files down -t0
    rm -rf tmp/up/*.txt
fi
