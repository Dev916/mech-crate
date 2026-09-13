#!/bin/bash
# Execute a command in a running container
# Usage: ./scripts/exec.sh <service> <command>

set -e

source ./scripts/.bashrc

if [ -z "$1" ] || [ -z "$2" ]; then
    echo "Usage: $0 <service> <command>"
    exit 1
fi

# One container runs the command: a list has no meaning here (bd:mech-crate-3kq).
mech_require_single_service "$1" "make exec" || exit 1

# Check if we have previous run context
if ! ls tmp/up/*.txt 1>/dev/null 2>&1; then
    echo "No services running. Start services first."
    exit 1
fi

files=$(cat tmp/up/*.txt)

docker compose -p "$COMPOSE_PROJECT_NAME" $files exec "$1" $2
