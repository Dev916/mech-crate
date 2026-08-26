#!/bin/bash
# Execute a command in a running container
# Usage: ./scripts/exec.sh <service> <command>

set -e

if [ -z "$1" ] || [ -z "$2" ]; then
    echo "Usage: $0 <service> <command>"
    exit 1
fi

# Check if we have previous run context
if ! ls tmp/up/*.txt 1>/dev/null 2>&1; then
    echo "No services running. Start services first."
    exit 1
fi

files=$(cat tmp/up/*.txt)

docker compose $files exec "$1" $2
