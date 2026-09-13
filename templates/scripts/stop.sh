#!/bin/bash
# Stop services (without removing)
# Usage: ./scripts/stop.sh [service]

set -e

source ./scripts/.bashrc

service=$1

if ! ls tmp/up/*.txt 1>/dev/null 2>&1; then
    echo "No previous runs found. Doing normal stop..."
    docker compose -p "$COMPOSE_PROJECT_NAME" stop -t0
    exit 0
fi

files=$(cat tmp/up/*.txt)

docker compose -p "$COMPOSE_PROJECT_NAME" $files stop $service
