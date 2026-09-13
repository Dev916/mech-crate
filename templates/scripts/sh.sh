#!/bin/bash
# Shell into a running service
# Usage: ./scripts/sh.sh <service>

set -e

source ./scripts/.bashrc

if [ -z "$1" ]; then
    echo "Usage: $0 <service>"
    exit 1
fi

# One shell, one container: a list has no meaning here (bd:mech-crate-3kq).
mech_require_single_service "$1" "make sh" || exit 1

files=$(compose_context_files "$1" "true")

docker compose -p "$COMPOSE_PROJECT_NAME" $files exec "$1" sh
