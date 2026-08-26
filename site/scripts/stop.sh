#!/bin/bash
# Stop services (without removing)
# Usage: ./scripts/stop.sh [service]

set -e

service=$1

if ! ls tmp/up/*.txt 1>/dev/null 2>&1; then
    echo "No previous runs found. Doing normal stop..."
    docker compose stop -t0
    exit 0
fi

files=$(cat tmp/up/*.txt)

docker compose $files stop $service
